mod binops;
mod exp_desc;
mod helper_types;
mod proto;
mod unops;

use alloc::{boxed::Box, vec, vec::Vec};
use exp_desc::ExpDesc;
use helper_types::{FunctionNameList, ParList, TableFields, TableKey};

use crate::{
    bytecode::{Bytecode, OpCode},
    function::Function,
    parser::{Parser, Token, TokenType},
    program::{Error, Local},
};

pub use proto::Proto;

macro_rules! make_deconstruct {
    ($($name:ident($token:pat$(,)?)),+$(,)?) => {
        [$($name @ Token {
            tokens: _,
            token_type: $token,
        },)+]
    };
}

type ExpList<'a> = Vec<ExpDesc<'a>>;
type NameList<'a> = Vec<Box<str>>;

#[derive(Debug, Default)]
pub struct CompileContext<'a> {
    pub proto: Proto,
    pub stack_top: u8,
    pub previous_context: Option<&'a CompileContext<'a>>,
    pub var_args: Option<bool>,
    pub locals: Vec<Box<str>>,
    pub breaks: Option<Vec<usize>>,
    pub gotos: Vec<GotoLabel<'a>>,
    pub labels: Vec<GotoLabel<'a>>,
    pub jumps_to_block: Vec<usize>,
    pub jumps_to_end: Vec<usize>,
}

impl<'a> CompileContext<'a> {
    pub fn with_var_args(mut self, var_args: bool) -> Self {
        self.var_args = Some(var_args);
        self
    }

    pub fn parse(program: &str) -> Result<Proto, Error> {
        let chunk = Parser::parse(program)?;

        let mut compile_context = CompileContext::default().with_var_args(true);
        compile_context.chunk(&chunk)?;

        Ok(compile_context.proto)
    }

    // Non-terminals
    pub fn chunk(&mut self, chunk: &Token<'a>) -> Result<(), Error> {
        match chunk.tokens.as_slice() {
            make_deconstruct!(block(TokenType::Block)) => {
                self.proto.push_upvalue("_ENV");

                self.block(block)?;

                self.proto.byte_codes.push(Bytecode::zero_return());
                self.fix_up_last_return(0)?;

                self.close_locals(0);

                if self.gotos.is_empty() {
                    Ok(())
                } else {
                    for goto in self.gotos.iter() {
                        log::error!(
                            target: "no_deps_lua::parser",
                            "Goto `{}` did not point to a label.",
                            goto.name
                        );
                    }
                    Err(Error::UnmatchedGoto)
                }
            }
            _ => {
                unreachable!(
                    "Chunk did not match any of the productions. Had {:#?}.",
                    chunk
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn block(&mut self, block: &Token<'a>) -> Result<(), Error> {
        match block.tokens.as_slice() {
            make_deconstruct!(
                block_stat(TokenType::BlockStat),
                block_retstat(TokenType::BlockRetstat)
            ) => {
                let gotos = self.gotos.len();
                let labels = self.labels.len();
                let locals = self.locals.len();

                if self.var_args.unwrap_or(false) {
                    self.proto
                        .byte_codes
                        .push(Bytecode::variadic_arguments_prepare(u8::try_from(locals)?));
                }

                self.block_stat(block_stat)?;
                self.block_retstat(block_retstat)?;

                let unmatched = self
                    .gotos
                    .drain(gotos..)
                    .filter_map(|goto| {
                        if let Some(label) = self
                            .labels
                            .iter()
                            .rev()
                            .find(|label| label.name == goto.name)
                        {
                            if label.bytecode != self.proto.byte_codes.len()
                                && label.nvar > goto.nvar
                            {
                                return Some(Err(Error::GotoIntoScope));
                            }
                            let Ok(label_i) = isize::try_from(label.bytecode) else {
                                return Some(Err(Error::IntCoversion));
                            };
                            let Ok(goto_i) = isize::try_from(goto.bytecode) else {
                                return Some(Err(Error::IntCoversion));
                            };
                            let Ok(jump) = i32::try_from((label_i - 1) - goto_i) else {
                                return Some(Err(Error::LongJump));
                            };
                            self.proto.byte_codes[goto.bytecode] = Bytecode::jump(jump);
                            None
                        } else {
                            Some(Ok(goto))
                        }
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                self.gotos.extend(unmatched);
                self.labels.truncate(labels);

                Ok(())
            }
            _ => {
                unreachable!(
                    "Block did not match any production. Had {:#?}.",
                    block
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn block_stat(&mut self, block: &Token<'a>) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(stat(TokenType::Stat), blockstat(TokenType::BlockStat)) => {
                self.stat(stat).and_then(|()| self.block_stat(blockstat))
            }
            _ => {
                unreachable!(
                    "BlockStat did not match any production. Had {:#?}.",
                    block
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn block_retstat(&mut self, block_retstat: &Token<'a>) -> Result<(), Error> {
        match block_retstat.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(retstat(TokenType::Retstat)) => self.retstat(retstat),
            _ => {
                unreachable!(
                    "BlockRetstat did not match any production. Had {:#?}.",
                    block_retstat
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn stat(&mut self, stat: &Token<'a>) -> Result<(), Error> {
        match stat.tokens.as_slice() {
            make_deconstruct!(_semicolon(TokenType::SemiColon)) => Ok(()),
            make_deconstruct!(
                varlist(TokenType::Varlist),
                _assing(TokenType::Assign),
                explist(TokenType::Explist)
            ) => {
                let varlist = self.varlist(varlist)?;
                let explist = self.explist(explist)?;

                ExpDesc::ExpList(varlist).discharge(&ExpDesc::ExpList(explist), self)
            }
            make_deconstruct!(functioncall(TokenType::Functioncall)) => {
                let function_call = self.functioncall(functioncall)?;

                let (_, stack_top) = self.reserve_stack_top();
                stack_top.discharge(&function_call, self)?;
                self.stack_top -= 1;

                Ok(())
            }
            make_deconstruct!(label(TokenType::Label)) => self.label(label),
            make_deconstruct!(_break(TokenType::Break)) => match self.breaks.as_mut() {
                Some(breaks) => {
                    let bytecode = self.proto.byte_codes.len();
                    breaks.push(bytecode);
                    self.proto.byte_codes.push(Bytecode::jump(0));
                    Ok(())
                }
                None => Err(Error::BreakOutsideLoop),
            },
            make_deconstruct!(_goto(TokenType::Goto), _name(TokenType::Name(name))) => {
                let bytecode = self.proto.byte_codes.len();
                self.proto.byte_codes.push(Bytecode::jump(0));

                self.push_goto(GotoLabel {
                    name,
                    bytecode,
                    nvar: self.locals.len(),
                });

                Ok(())
            }
            make_deconstruct!(
                _do(TokenType::Do),
                block(TokenType::Block),
                _end(TokenType::End)
            ) => {
                let locals = self.locals.len();
                let cache_var_args = self.var_args.take();

                self.block(block)?;

                self.var_args = cache_var_args;
                self.close_locals(locals);

                Ok(())
            }
            make_deconstruct!(
                _while(TokenType::While),
                exp(TokenType::Exp),
                _do(TokenType::Do),
                block(TokenType::Block),
                _end(TokenType::End)
            ) => {
                let jump_to_block_count = self.jumps_to_block.len();
                let jump_to_end_count = self.jumps_to_end.len();
                let locals = self.locals.len();
                let mut cache_break = self.breaks.replace(Vec::with_capacity(16));

                let start_of_cond = self.proto.byte_codes.len();
                let cond = self.exp(exp)?;
                ExpDesc::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(&cond, self)?;

                let end_of_cond = self.proto.byte_codes.len();
                for jump in self.jumps_to_block.drain(jump_to_block_count..) {
                    self.proto.byte_codes[jump] = Bytecode::jump(
                        i32::try_from(end_of_cond - jump).map_err(|_| Error::LongJump)?,
                    );
                }

                let cache_var_args = self.var_args.take();

                self.block(block)?;

                self.var_args = cache_var_args;
                self.close_locals(locals);
                self.stack_top -= u8::try_from(self.locals.len() - locals).inspect_err(|_| {
                    log::error!("Failed to rewind stack top after `while`s block.")
                })?;

                let end_of_block = self.proto.byte_codes.len();
                for jump in self.jumps_to_end.drain(jump_to_end_count..) {
                    self.proto.byte_codes[jump] = Bytecode::jump(
                        i32::try_from(end_of_block - jump).map_err(|_| Error::LongJump)?,
                    );
                }

                core::mem::swap(&mut self.breaks, &mut cache_break);
                let Some(breaks) = cache_break else {
                    unreachable!(
                        "Compile Context breaks should only ever be None outside of loops."
                    );
                };
                for break_bytecode in breaks {
                    self.proto.byte_codes[break_bytecode] = Bytecode::jump(
                        i32::try_from(end_of_block - break_bytecode)
                            .map_err(|_| Error::LongJump)?,
                    );
                }

                self.proto.byte_codes.push(Bytecode::jump(
                    i32::try_from(start_of_cond)
                        .and_then(|lhs| i32::try_from(end_of_block + 1).map(|rhs| (lhs, rhs)))
                        .map(|(lhs, rhs)| lhs - rhs)
                        .map_err(|_| Error::LongJump)?,
                ));

                Ok(())
            }
            make_deconstruct!(
                _repeat(TokenType::Repeat),
                block(TokenType::Block),
                _until(TokenType::Until),
                exp(TokenType::Exp)
            ) => {
                let mut jump_cache = core::mem::take(&mut self.jumps_to_end);

                let locals = self.locals.len();
                let repeat_start = self.proto.byte_codes.len();

                let cache_var_args = self.var_args.take();
                self.block(block)?;
                self.var_args = cache_var_args;

                let cond = self.exp(exp)?;

                ExpDesc::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(&cond, self)?;
                self.close_locals(locals);

                core::mem::swap(&mut self.jumps_to_end, &mut jump_cache);
                assert_eq!(
                    jump_cache.len(),
                    1,
                    "Repeat should only ever have 1 conditional jump."
                );

                let repeat_end = self.proto.byte_codes.len();
                self.proto.byte_codes[jump_cache[0]] = Bytecode::jump(
                    i32::try_from(isize::try_from(repeat_start)? - isize::try_from(repeat_end)?)
                        .map_err(|_| Error::LongJump)?,
                );

                Ok(())
            }
            make_deconstruct!(
                _if(TokenType::If),
                exp(TokenType::Exp),
                _then(TokenType::Then),
                block(TokenType::Block),
                stat_if(TokenType::StatIf),
                _end(TokenType::End)
            ) => self.make_if(exp, block, stat_if),
            make_deconstruct!(
                _for(TokenType::For),
                _name(TokenType::Name(name)),
                _assign(TokenType::Assign),
                start(TokenType::Exp),
                _comma(TokenType::Comma),
                end(TokenType::Exp),
                stat_forexp(TokenType::StatForexp),
                _do(TokenType::Do),
                block(TokenType::Block),
                _end(TokenType::End)
            ) => {
                let locals = self.locals.len();

                let start = self.exp(start)?;
                let (for_stack, start_stack) = self.reserve_stack_top();
                start_stack.discharge(&start, self)?;

                let end = self.exp(end)?;
                let (_, end_stack) = self.reserve_stack_top();
                end_stack.discharge(&end, self)?;

                let step = self.stat_forexp(stat_forexp)?;
                let (_, step_stack) = self.reserve_stack_top();
                step_stack.discharge(&step, self)?;

                // Names can't start with `?`, so using it for internal symbols
                for for_var in ["?for_start", "?for_end", "?for_step"] {
                    self.open_local(for_var);
                }

                // Reserve 1 slot for counter
                self.stack_top += 1;

                let counter_bytecode = self.proto.byte_codes.len();
                self.proto
                    .byte_codes
                    .push(Bytecode::for_prepare(for_stack, 0));

                self.open_local(name);

                let cache_var_args = self.var_args.take();
                self.block(block)?;
                self.var_args = cache_var_args;

                // Close just the for variable
                self.close_locals(locals + 3);

                let end_bytecode = self.proto.byte_codes.len();
                self.proto.byte_codes.push(Bytecode::for_loop(
                    for_stack,
                    u32::try_from(end_bytecode - counter_bytecode)?,
                ));
                self.proto.byte_codes[counter_bytecode] = Bytecode::for_prepare(
                    for_stack,
                    u32::try_from(end_bytecode - (counter_bytecode + 1))?,
                );

                self.stack_top = for_stack;
                // Close for states
                self.close_locals(locals);

                Ok(())
            }
            make_deconstruct!(
                _for(TokenType::For),
                _namelist(TokenType::Namelist),
                _in(TokenType::In),
                _explist(TokenType::Explist),
                _do(TokenType::Do),
                _block(TokenType::Block),
                _end(TokenType::End)
            ) => unimplemented!("stat production"),
            make_deconstruct!(
                _function(TokenType::Function),
                funcname(TokenType::Funcname),
                funcbody(TokenType::Funcbody)
            ) => {
                let function_name = self.funcname(funcname)?;
                let funcbody = self.funcbody(funcbody, function_name.has_method)?;

                let [head @ .., tail] = function_name.names.as_slice() else {
                    unreachable!("Function name should never be empty.");
                };

                let mut stacks_used = 0;

                let final_dst = if head.is_empty() {
                    // This is the case where the function is defined as
                    // function f() ... end
                    let constant = self.proto.push_constant(*tail)?;
                    ExpDesc::Global(usize::try_from(constant)?)
                } else {
                    let (stack_loc, stack_top) = self.reserve_stack_top();
                    let mut used_stack_top = false;

                    let mut table_loc = if let Some(local) = self.find_name(head[0]) {
                        u8::try_from(local)?
                    } else {
                        used_stack_top = true;
                        let constant = self.proto.push_constant(head[0])?;
                        self.proto.byte_codes.push(Bytecode::get_uptable(
                            stack_loc,
                            0,
                            u8::try_from(constant)?,
                        ));
                        stack_loc
                    };

                    for table_key in &head[1..] {
                        stack_top.discharge(
                            &ExpDesc::TableAccess {
                                table: Box::new(ExpDesc::Local(usize::from(table_loc))),
                                key: Box::new(self.name(table_key)),
                                record: true,
                            },
                            self,
                        )?;

                        used_stack_top = true;
                        table_loc = stack_loc;
                    }

                    if used_stack_top {
                        stacks_used += 1;
                    } else {
                        self.stack_top -= 1;
                    }

                    ExpDesc::TableAccess {
                        table: Box::new(ExpDesc::Local(usize::from(table_loc))),
                        key: Box::new(self.name(tail)),
                        record: true,
                    }
                };

                let (_, funcbody_stack) = self.reserve_stack_top();
                stacks_used += 1;

                funcbody_stack.discharge(&funcbody, self)?;

                final_dst.discharge(&funcbody_stack, self)?;

                self.stack_top -= stacks_used;

                Ok(())
            }
            make_deconstruct!(
                _local(TokenType::Local),
                _function(TokenType::Function),
                _name(TokenType::Name(name)),
                funcbody(TokenType::Funcbody)
            ) => {
                let funcbody = self.funcbody(funcbody, false)?;

                let (_, function_body) = self.reserve_stack_top();
                function_body.discharge(&funcbody, self)?;

                self.open_local(name);

                Ok(())
            }
            make_deconstruct!(
                _local(TokenType::Local),
                attnamelist(TokenType::Attnamelist),
                stat_attexplist(TokenType::StatAttexplist)
            ) => {
                let namelist = self.attnamelist(attnamelist)?;
                let explist = self.stat_attexplist(stat_attexplist)?;

                let dst_locals = namelist
                    .iter()
                    .map(|_| {
                        let (_, loc) = self.reserve_stack_top();
                        loc
                    })
                    .collect::<Vec<_>>();

                ExpDesc::ExpList(dst_locals).discharge(&ExpDesc::ExpList(explist), self)?;

                // Adding the new names into `locals` to prevent
                // referencing the new name when you could be trying to shadow a
                // global or another local
                for local in namelist {
                    self.open_local(local.as_ref());
                }
                Ok(())
            }
            _ => {
                unreachable!(
                    "Stat did not match any of the productions. Had {:#?}.",
                    stat.tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn stat_if(&mut self, stat_if: &Token<'a>) -> Result<(), Error> {
        match stat_if.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _elseif(TokenType::Elseif),
                exp(TokenType::Exp),
                _then(TokenType::Then),
                block(TokenType::Block),
                stat_if(TokenType::StatIf)
            ) => self.make_if(exp, block, stat_if),
            make_deconstruct!(_else(TokenType::Else), block(TokenType::Block)) => {
                let locals = self.locals.len();
                let cache_var_args = self.var_args.take();

                self.block(block)?;

                self.var_args = cache_var_args;
                self.stack_top -= u8::try_from(self.locals.len() - locals).inspect_err(|_| {
                    log::error!("Failed to rewind stack top after `else`s block.")
                })?;
                self.close_locals(locals);

                Ok(())
            }
            _ => {
                unreachable!(
                    "StatIf did not match any of the productions. Had {:#?}.",
                    stat_if
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn stat_forexp(&mut self, stat_forexp: &Token<'a>) -> Result<ExpDesc<'a>, Error> {
        match stat_forexp.tokens.as_slice() {
            [] => Ok(ExpDesc::Integer(1)),
            make_deconstruct!(_comma(TokenType::Comma), exp(TokenType::Exp)) => self.exp(exp),
            _ => {
                unreachable!(
                    "StatForexp did not match any of the productions. Had {:#?}.",
                    stat_forexp
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn stat_attexplist(&mut self, stat_attexplist: &Token<'a>) -> Result<ExpList<'a>, Error> {
        match stat_attexplist.tokens.as_slice() {
            [] => Ok(ExpList::new()),
            make_deconstruct!(_assign(TokenType::Assign), explist(TokenType::Explist)) => {
                self.explist(explist)
            }
            _ => {
                unreachable!(
                    "StatAttexplist did not match any of the productions. Had {:#?}.",
                    stat_attexplist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn attnamelist(&mut self, attnamelist: &Token<'_>) -> Result<NameList, Error> {
        match attnamelist.tokens.as_slice() {
            make_deconstruct!(
                _name(TokenType::Name(name)),
                _attrib(TokenType::Attrib),
                attnamelist_cont(TokenType::AttnamelistCont)
            ) => {
                let mut namelist = NameList::default();
                namelist.push((*name).into());

                Self::attnamelist_cont(attnamelist_cont, &mut namelist)?;

                Ok(namelist)
            }
            _ => {
                unreachable!(
                    "Attnamelist did not match any of the productions. Had {:#?}.",
                    attnamelist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn attnamelist_cont(
        attnamelist_cont: &Token<'_>,
        namelist: &mut NameList,
    ) -> Result<(), Error> {
        match attnamelist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                _name(TokenType::Name(name)),
                attrib(TokenType::Attrib),
                attnamelist_cont(TokenType::AttnamelistCont)
            ) => {
                namelist.push((*name).into());

                Self::attrib(attrib)?;

                Self::attnamelist_cont(attnamelist_cont, namelist)
            }
            _ => {
                unreachable!(
                    "AttnamelistCont did not match any of the productions. Had {:#?}.",
                    attnamelist_cont
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn attrib(attrib: &Token) -> Result<(), Error> {
        match attrib.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _less(TokenType::Less),
                _name(TokenType::Name(_)),
                _greater(TokenType::Greater)
            ) => unimplemented!("attrib production"),
            _ => {
                unreachable!(
                    "Attrib did not match any of the productions. Had {:#?}.",
                    attrib
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn retstat(&mut self, retstat: &Token<'a>) -> Result<(), Error> {
        match retstat.tokens.as_slice() {
            make_deconstruct!(
                _return(TokenType::Return),
                retstat_explist(TokenType::RetstatExplist),
                retstat_end(TokenType::RetstatEnd)
            ) => {
                let explist = self.retstat_explist(retstat_explist)?;

                match explist.len() {
                    0 => self.proto.byte_codes.push(Bytecode::zero_return()),
                    1 => {
                        let Some(last) = explist.last() else {
                            unreachable!(
                                "Return list should only have 1 exp, but had {}.",
                                explist.len()
                            );
                        };

                        let (stack_loc, stack_top) = self.reserve_stack_top();
                        if let ExpDesc::Name(_) = last {
                            let dst = last.get_local_or_discharge_at_location(self, stack_loc)?;

                            self.proto.byte_codes.push(Bytecode::one_return(dst))
                        } else {
                            stack_top.discharge(last, self)?;

                            match last {
                                ExpDesc::FunctionCall(_, _) => {
                                    let Some(call) = self.proto.byte_codes.pop() else {
                                        unreachable!("Last should always be a function call");
                                    };
                                    assert_eq!(call.get_opcode(), OpCode::Call);
                                    let (func_index, inputs, _, _) = call.decode_abck();

                                    self.proto
                                        .byte_codes
                                        .push(Bytecode::tail_call(func_index, inputs, 0));
                                    self.proto
                                        .byte_codes
                                        .push(Bytecode::return_bytecode(stack_loc, 0, 0));
                                }
                                ExpDesc::Closure(_) => self
                                    .proto
                                    .byte_codes
                                    .push(Bytecode::return_bytecode(stack_loc, 2, 0)),
                                _ => {
                                    self.proto.byte_codes.push(Bytecode::one_return(stack_loc));
                                }
                            }
                        };
                        self.stack_top -= 1;
                    }
                    _ => {
                        let return_start = self.stack_top;
                        for exp in explist.iter() {
                            let (_, stack_top) = self.reserve_stack_top();
                            stack_top.discharge(exp, self)?;
                        }
                        self.stack_top -= u8::try_from(explist.len())?;

                        self.proto.byte_codes.push(Bytecode::return_bytecode(
                            return_start,
                            u8::try_from(explist.len())? + 1,
                            0,
                        ));
                    }
                }

                self.retstat_end(retstat_end)?;

                Ok(())
            }
            _ => {
                unreachable!(
                    "Retstat did not match any of the productions. Had {:#?}.",
                    retstat
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn retstat_explist(&mut self, retstat_explist: &Token<'a>) -> Result<ExpList<'a>, Error> {
        match retstat_explist.tokens.as_slice() {
            [] => Ok(ExpList::new()),
            make_deconstruct!(explist(TokenType::Explist)) => self.explist(explist),
            _ => {
                unreachable!(
                    "RetstatExplist did not match any of the productions. Had {:#?}.",
                    retstat_explist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn retstat_end(&mut self, retstat_end: &Token) -> Result<(), Error> {
        match retstat_end.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(_semicolon(TokenType::SemiColon)) => Ok(()),
            _ => {
                unreachable!(
                    "RetstatEnd did not match any of the productions. Had {:#?}.",
                    retstat_end
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn label(&mut self, label: &Token<'a>) -> Result<(), Error> {
        match label.tokens.as_slice() {
            make_deconstruct!(
                _doublecolon1(TokenType::DoubleColon),
                _name(TokenType::Name(name)),
                _doublecolon2(TokenType::DoubleColon)
            ) => self.push_label(GotoLabel {
                name,
                bytecode: self.proto.byte_codes.len(),
                nvar: self.locals.len(),
            }),
            _ => {
                unreachable!(
                    "Label did not match any of the productions. Had {:#?}.",
                    label
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn funcname(&mut self, funcname: &Token<'a>) -> Result<FunctionNameList<'a>, Error> {
        match funcname.tokens.as_slice() {
            make_deconstruct!(
                _name(TokenType::Name(name)),
                funcname_cont(TokenType::FuncnameCont),
                funcname_end(TokenType::FuncnameEnd)
            ) => {
                let mut func_namelist = FunctionNameList::default();
                func_namelist.names.push(name);

                Self::funcname_cont(funcname_cont, &mut func_namelist)?;
                self.funcname_end(funcname_end, &mut func_namelist)?;

                Ok(func_namelist)
            }
            _ => {
                unreachable!(
                    "Funcname did not match any of the productions. Had {:#?}.",
                    funcname
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn funcname_cont(
        funcname_cont: &Token<'a>,
        func_namelist: &mut FunctionNameList<'a>,
    ) -> Result<(), Error> {
        match funcname_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _dot(TokenType::Dot),
                _name(TokenType::Name(name)),
                funcname_cont(TokenType::FuncnameCont)
            ) => {
                func_namelist.names.push(name);

                Self::funcname_cont(funcname_cont, func_namelist)?;

                Ok(())
            }
            _ => {
                unreachable!(
                    "FuncnameCont did not match any of the productions. Had {:#?}.",
                    funcname_cont
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn funcname_end(
        &mut self,
        funcname_end: &Token<'a>,
        func_namelist: &mut FunctionNameList<'a>,
    ) -> Result<(), Error> {
        match funcname_end.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(_colon(TokenType::Colon), _name(TokenType::Name(name))) => {
                func_namelist.names.push(name);
                func_namelist.has_method = true;
                Ok(())
            }
            _ => {
                unreachable!(
                    "FuncnameEnd did not match any of the productions. Had {:#?}.",
                    funcname_end
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn varlist(&mut self, varlist: &Token<'a>) -> Result<ExpList<'a>, Error> {
        match varlist.tokens.as_slice() {
            make_deconstruct!(var(TokenType::Var), varlist_cont(TokenType::VarlistCont)) => {
                let mut varlist = ExpList::new();

                let var = self.var(var)?;
                varlist.push(var);

                self.varlist_cont(varlist_cont, &mut varlist)?;

                Ok(varlist)
            }
            _ => {
                unreachable!(
                    "Varlist did not match any of the productions. Had {:#?}.",
                    varlist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn varlist_cont(
        &mut self,
        varlist_cont: &Token<'a>,
        varlist: &mut Vec<ExpDesc<'a>>,
    ) -> Result<(), Error> {
        match varlist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                var(TokenType::Var),
                varlist_cont(TokenType::VarlistCont)
            ) => {
                let var_exp_desc = self.var(var)?;
                varlist.push(var_exp_desc);
                self.varlist_cont(varlist_cont, varlist)
            }
            _ => {
                unreachable!(
                    "VarlistCont did not match any of the productions. Had {:#?}.",
                    varlist_cont
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn var(&mut self, var: &Token<'a>) -> Result<ExpDesc<'a>, Error> {
        match var.tokens.as_slice() {
            make_deconstruct!(_name(TokenType::Name(name))) => Ok(self.name(name)),
            make_deconstruct!(
                prefixexp(TokenType::Prefixexp),
                _lsquare(TokenType::LSquare),
                exp(TokenType::Exp),
                _rsquare(TokenType::RSquare)
            ) => {
                let table = self.prefixexp(prefixexp)?;
                let key = self.exp(exp)?;

                Ok(ExpDesc::TableAccess {
                    table: Box::new(table),
                    key: Box::new(key),
                    record: false,
                })
            }
            make_deconstruct!(
                prefixexp(TokenType::Prefixexp),
                _dot(TokenType::Dot),
                _name(TokenType::Name(name))
            ) => {
                let table = self.prefixexp(prefixexp)?;
                let key = self.name(name);

                Ok(ExpDesc::TableAccess {
                    table: Box::new(table),
                    key: Box::new(key),
                    record: true,
                })
            }

            _ => {
                unreachable!(
                    "Var did not match any of the productions. Had {:#?}.",
                    var.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn explist(&mut self, explist: &Token<'a>) -> Result<ExpList<'a>, Error> {
        match explist.tokens.as_slice() {
            make_deconstruct!(exp(TokenType::Exp), explist_cont(TokenType::ExplistCont)) => {
                let mut explist = ExpList::new();

                let exp = self.exp(exp)?;
                explist.push(exp);
                self.explist_cont(explist_cont, &mut explist)?;

                Ok(explist)
            }
            _ => {
                unreachable!(
                    "Explist did not match any of the productions. Had {:#?}.",
                    explist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn explist_cont(
        &mut self,
        explist_cont: &Token<'a>,
        explist: &mut Vec<ExpDesc<'a>>,
    ) -> Result<(), Error> {
        match explist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                exp(TokenType::Exp),
                explist_cont(TokenType::ExplistCont)
            ) => {
                let exp = self.exp(exp)?;
                explist.push(exp);
                self.explist_cont(explist_cont, explist)
            }
            _ => {
                unreachable!(
                    "ExplistCont did not match any of the productions. Had {:#?}.",
                    explist_cont
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn exp(&mut self, exp: &Token<'a>) -> Result<ExpDesc<'a>, Error> {
        match exp.tokens.as_slice() {
            make_deconstruct!(_nil(TokenType::Nil)) => Ok(self.nil()),
            make_deconstruct!(_false(TokenType::False)) => Ok(self.boolean(false)),
            make_deconstruct!(_true(TokenType::True)) => Ok(self.boolean(true)),
            make_deconstruct!(_string(TokenType::String(string))) => Ok(self.string(string)),
            make_deconstruct!(_integer(TokenType::Integer(integer))) => Ok(self.integer(*integer)),
            make_deconstruct!(_float(TokenType::Float(float))) => Ok(self.float(*float)),
            make_deconstruct!(_dots(TokenType::Dots)) => Ok(ExpDesc::VariadicArguments),
            make_deconstruct!(functiondef(TokenType::Functiondef)) => self.functiondef(functiondef),
            make_deconstruct!(prefixexp(TokenType::Prefixexp)) => self.prefixexp(prefixexp),
            make_deconstruct!(tableconstructor(TokenType::Tableconstructor)) => {
                self.tableconstructor(tableconstructor)
            }
            make_deconstruct!(
                lhs(TokenType::Exp),
                op(TokenType::Binop),
                rhs(TokenType::Exp)
            ) => {
                let op = self.binop(op)?;
                let lhs = self.exp(lhs)?;
                let rhs = self.exp(rhs)?;

                let binop = op.try_into()?;
                Ok(ExpDesc::Binop(binop, Box::new(lhs), Box::new(rhs)))
            }
            make_deconstruct!(op(TokenType::Unop), rhs(TokenType::Exp)) => {
                let op = self.unop(op)?;
                let rhs = self.exp(rhs)?;

                let func = match op {
                    TokenType::Not => unops::unop_not,
                    TokenType::Len => unops::unop_len,
                    TokenType::Sub => unops::unop_neg,
                    TokenType::BitXor => unops::unop_bitnot,
                    other => unreachable!("{:?} is not a unary operator", other),
                };
                func(&rhs)
            }
            _ => {
                unreachable!(
                    "Exp did not match any of the productions. Had {:#?}.",
                    exp.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn prefixexp(&mut self, prefixexp: &Token<'a>) -> Result<ExpDesc<'a>, Error> {
        match prefixexp.tokens.as_slice() {
            make_deconstruct!(var(TokenType::Var)) => self.var(var),
            make_deconstruct!(functioncall(TokenType::Functioncall)) => {
                self.functioncall(functioncall)
            }
            make_deconstruct!(
                _lparen(TokenType::LParen),
                exp(TokenType::Exp),
                _rparen(TokenType::RParen)
            ) => self.exp(exp),
            _ => {
                unreachable!(
                    "Prefixexp did not match any of the productions. Had {:#?}.",
                    prefixexp
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn functioncall(&mut self, functioncall: &Token<'a>) -> Result<ExpDesc<'a>, Error> {
        match functioncall.tokens.as_slice() {
            make_deconstruct!(prefixexp(TokenType::Prefixexp), args(TokenType::Args)) => {
                let prefix = self.prefixexp(prefixexp)?;
                let args = self.args(args)?;

                Ok(ExpDesc::FunctionCall(Box::new(prefix), args))
            }
            make_deconstruct!(
                prefixexp(TokenType::Prefixexp),
                _colon(TokenType::Colon),
                _name(TokenType::Name(name)),
                args(TokenType::Args)
            ) => {
                let prefix = self.prefixexp(prefixexp)?;
                let name = self.name(name);
                let args = self.args(args)?;

                Ok(ExpDesc::MethodCall(Box::new(prefix), Box::new(name), args))
            }
            _ => {
                unreachable!(
                    "Functioncall did not match any of the productions. Had {:#?}.",
                    functioncall
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn args(&mut self, args: &Token<'a>) -> Result<ExpList<'a>, Error> {
        match args.tokens.as_slice() {
            make_deconstruct!(
                _lparen(TokenType::LParen),
                args_explist(TokenType::ArgsExplist),
                _rparen(TokenType::RParen)
            ) => self.args_explist(args_explist),
            make_deconstruct!(tableconstructor(TokenType::Tableconstructor)) => {
                let table = self.tableconstructor(tableconstructor)?;
                Ok(vec![table])
            }
            make_deconstruct!(_string(TokenType::String(string))) => Ok(vec![self.string(string)]),
            _ => {
                unreachable!(
                    "Args did not match any of the productions. Had {:#?}.",
                    args.tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn args_explist(&mut self, args_explist: &Token<'a>) -> Result<ExpList<'a>, Error> {
        match args_explist.tokens.as_slice() {
            [] => Ok(ExpList::new()),
            make_deconstruct!(explist(TokenType::Explist)) => self.explist(explist),
            _ => {
                unreachable!(
                    "ArgsExplist did not match any of the productions. Had {:#?}.",
                    args_explist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn functiondef(&mut self, functiondef: &Token<'a>) -> Result<ExpDesc<'a>, Error> {
        match functiondef.tokens.as_slice() {
            make_deconstruct!(
                _function(TokenType::Function),
                funcbody(TokenType::Funcbody)
            ) => self.funcbody(funcbody, false),
            _ => {
                unreachable!(
                    "Functiondef did not match any of the productions. Had {:#?}.",
                    functiondef
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn funcbody(&mut self, funcbody: &Token<'a>, needs_self: bool) -> Result<ExpDesc<'a>, Error> {
        match funcbody.tokens.as_slice() {
            make_deconstruct!(
                _lparen(TokenType::LParen),
                funcbody_parlist(TokenType::FuncbodyParlist),
                _rparen(TokenType::RParen),
                block(TokenType::Block),
                _end(TokenType::End),
            ) => {
                let parlist = self.funcbody_parlist(funcbody_parlist)?;
                let parlist_name_count = parlist.names.len();

                let mut func_compile_context =
                    CompileContext::default().with_var_args(parlist.variadic_args);
                func_compile_context.previous_context = Some(self);

                if needs_self {
                    func_compile_context.open_local("self");
                }
                for name in parlist.names {
                    func_compile_context.open_local(name.as_ref());
                }

                func_compile_context.stack_top =
                    u8::try_from(parlist_name_count)? + (needs_self as u8);

                func_compile_context.block(block)?;

                func_compile_context
                    .proto
                    .byte_codes
                    .push(Bytecode::zero_return());
                if parlist.variadic_args {
                    func_compile_context.fix_up_last_return(u8::try_from(parlist_name_count)?)?;
                }

                func_compile_context.close_locals(0);

                let closure_position = self.proto.push_function(Function::new(
                    func_compile_context.proto.into(),
                    parlist_name_count + (needs_self as usize),
                    parlist.variadic_args,
                ));

                Ok(ExpDesc::Closure(closure_position))
            }
            _ => {
                unreachable!(
                    "Funcbody did not match any of the productions. Had {:#?}.",
                    funcbody
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn funcbody_parlist(&mut self, funcbody_parlist: &Token<'_>) -> Result<ParList, Error> {
        match funcbody_parlist.tokens.as_slice() {
            [] => Ok(ParList::default()),
            make_deconstruct!(parlist(TokenType::Parlist)) => {
                let mut func_parlist = ParList::default();
                self.parlist(parlist, &mut func_parlist)?;
                Ok(func_parlist)
            }
            _ => {
                unreachable!(
                    "FuncbodyParlist did not match any of the productions. Had {:#?}.",
                    funcbody_parlist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn parlist(&mut self, parlist: &Token<'_>, func_parlist: &mut ParList) -> Result<(), Error> {
        match parlist.tokens.as_slice() {
            make_deconstruct!(
                _name(TokenType::Name(name)),
                parlist_cont(TokenType::ParlistCont)
            ) => {
                func_parlist.names.push((*name).into());
                Self::parlist_cont(parlist_cont, func_parlist)
            }
            make_deconstruct!(_dots(TokenType::Dots)) => {
                func_parlist.variadic_args = true;
                Ok(())
            }
            _ => {
                unreachable!(
                    "Parlist did not match any of the productions. Had {:#?}.",
                    parlist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn parlist_cont(parlist_cont: &Token<'_>, func_parlist: &mut ParList) -> Result<(), Error> {
        match parlist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                _name(TokenType::Name(name)),
                parlist_cont(TokenType::ParlistCont)
            ) => {
                func_parlist.names.push((*name).into());
                Self::parlist_cont(parlist_cont, func_parlist)
            }
            make_deconstruct!(_comma(TokenType::Comma), _dots(TokenType::Dots)) => {
                func_parlist.variadic_args = true;
                Ok(())
            }
            _ => {
                unreachable!(
                    "ParlistCont did not match any of the productions. Had {:#?}.",
                    parlist_cont
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn tableconstructor(&mut self, tableconstructor: &Token<'a>) -> Result<ExpDesc<'a>, Error> {
        match tableconstructor.tokens.as_slice() {
            make_deconstruct!(
                _lcurly(TokenType::LCurly),
                tableconstructor_fieldlist(TokenType::TableconstructorFieldlist),
                _rcurly(TokenType::RCurly)
            ) => {
                let fields = self.tableconstructor_fieldlist(tableconstructor_fieldlist)?;

                Ok(ExpDesc::Table(fields))
            }
            _ => {
                unreachable!(
                    "Tableconstructor did not match any of the productions. Had {:#?}.",
                    tableconstructor
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn tableconstructor_fieldlist(
        &mut self,
        tableconstructor_fieldlist: &Token<'a>,
    ) -> Result<TableFields<'a>, Error> {
        match tableconstructor_fieldlist.tokens.as_slice() {
            [] => Ok(TableFields::default()),
            make_deconstruct!(fieldlist(TokenType::Fieldlist)) => self.fieldlist(fieldlist),
            _ => {
                unreachable!(
                    "TableconstructorFieldlist did not match any of the productions. Had {:#?}.",
                    tableconstructor_fieldlist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn fieldlist(&mut self, fieldlist: &Token<'a>) -> Result<TableFields<'a>, Error> {
        match fieldlist.tokens.as_slice() {
            make_deconstruct!(
                field(TokenType::Field),
                fieldlist_cont(TokenType::FieldlistCont)
            ) => {
                let mut fields = TableFields::default();

                self.field(field, &mut fields)?;
                self.fieldlist_cont(fieldlist_cont, &mut fields)?;

                Ok(fields)
            }
            _ => {
                unreachable!(
                    "Fieldlist did not match any of the productions. Had {:#?}.",
                    fieldlist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn fieldlist_cont(
        &mut self,
        fieldlist_cont: &Token<'a>,
        fields: &mut TableFields<'a>,
    ) -> Result<(), Error> {
        match fieldlist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                fieldsep(TokenType::Fieldsep),
                field(TokenType::Field),
                fieldlist_cont(TokenType::FieldlistCont)
            ) => {
                self.fieldsep(fieldsep)?;
                self.field(field, fields)?;
                self.fieldlist_cont(fieldlist_cont, fields)?;

                Ok(())
            }
            make_deconstruct!(fieldsep(TokenType::Fieldsep)) => self.fieldsep(fieldsep),
            _ => {
                unreachable!(
                    "FieldlistCont did not match any of the productions. Had {:#?}.",
                    fieldlist_cont
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn field(&mut self, field: &Token<'a>, fields: &mut TableFields<'a>) -> Result<(), Error> {
        match field.tokens.as_slice() {
            make_deconstruct!(
                _lsquare(TokenType::LSquare),
                key(TokenType::Exp),
                _rsquare(TokenType::RSquare),
                _assing(TokenType::Assign),
                exp(TokenType::Exp)
            ) => {
                let key = self.exp(key)?;
                let exp = self.exp(exp)?;

                fields.push((TableKey::General(Box::new(key)), exp));

                Ok(())
            }
            make_deconstruct!(
                _name(TokenType::Name(name)),
                _assign(TokenType::Assign),
                exp(TokenType::Exp)
            ) => {
                let constant = self.name(name);
                let exp = self.exp(exp)?;
                fields.push((TableKey::Record(Box::new(constant)), exp));

                Ok(())
            }
            make_deconstruct!(exp(TokenType::Exp)) => {
                let exp = self.exp(exp)?;

                fields.push((TableKey::Array, exp));

                Ok(())
            }
            _ => {
                unreachable!(
                    "Field did not match any of the productions. Had {:#?}.",
                    field
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    /// Test against `Comma` and `SemiColon` to garantee
    /// integrity of AST
    fn fieldsep(&mut self, fieldsep: &Token<'_>) -> Result<(), Error> {
        match fieldsep.tokens.as_slice() {
            make_deconstruct!(_comma(TokenType::Comma)) => Ok(()),
            make_deconstruct!(_semicolon(TokenType::SemiColon)) => Ok(()),
            _ => {
                unreachable!(
                    "Fieldsep did not match any of the productions. Had {:#?}.",
                    fieldsep
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn binop(&mut self, binop: &Token<'a>) -> Result<TokenType<'a>, Error> {
        match binop.tokens.as_slice() {
            make_deconstruct!(
                _binop(
                    token @ (TokenType::Or
                    | TokenType::And
                    | TokenType::Less
                    | TokenType::Greater
                    | TokenType::Leq
                    | TokenType::Geq
                    | TokenType::Eq
                    | TokenType::Neq
                    | TokenType::BitOr
                    | TokenType::BitXor
                    | TokenType::BitAnd
                    | TokenType::ShiftL
                    | TokenType::ShiftR
                    | TokenType::Concat
                    | TokenType::Add
                    | TokenType::Sub
                    | TokenType::Mul
                    | TokenType::Div
                    | TokenType::Idiv
                    | TokenType::Mod
                    | TokenType::Pow),
                )
            ) => Ok(*token),
            _ => {
                unreachable!(
                    "Binop did not match any of the productions. Had {:#?}.",
                    binop
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn unop(&mut self, unop: &Token<'a>) -> Result<TokenType<'a>, Error> {
        match unop.tokens.as_slice() {
            make_deconstruct!(
                _binop(
                    token @ (TokenType::BitXor | TokenType::Sub | TokenType::Len | TokenType::Not),
                )
            ) => Ok(*token),
            _ => {
                unreachable!(
                    "Unop did not match any of the productions. Had {:#?}.",
                    unop.tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    // Terminals
    #[inline(always)]
    fn nil(&mut self) -> ExpDesc<'a> {
        ExpDesc::Nil
    }

    #[inline(always)]
    fn boolean(&mut self, boolean: bool) -> ExpDesc<'a> {
        ExpDesc::Boolean(boolean)
    }

    #[inline(always)]
    fn integer(&mut self, integer: i64) -> ExpDesc<'a> {
        ExpDesc::Integer(integer)
    }

    #[inline(always)]
    fn float(&mut self, float: f64) -> ExpDesc<'a> {
        ExpDesc::Float(float)
    }

    #[inline(always)]
    fn string(&mut self, string: &'a str) -> ExpDesc<'a> {
        ExpDesc::String(string)
    }

    #[inline(always)]
    fn name(&mut self, name: &'a str) -> ExpDesc<'a> {
        ExpDesc::Name(name)
    }

    pub fn find_name(&self, name: &'a str) -> Option<usize> {
        self.locals.iter().rposition(|local| local.as_ref() == name)
    }

    pub fn exists_in_upvalue(&self, name: &'a str) -> bool {
        if self
            .locals
            .iter()
            .any(|local| local.as_ref() == name || local.as_ref() == "_ENV")
        {
            true
        } else {
            self.previous_context
                .filter(|context| context.exists_in_upvalue(name))
                .is_some()
        }
    }

    fn make_if(
        &mut self,
        exp: &Token<'a>,
        block: &Token<'a>,
        stat_if: &Token<'a>,
    ) -> Result<(), Error> {
        let jump_to_block_count = self.jumps_to_block.len();
        let jump_to_end_count = self.jumps_to_end.len();

        let cond = self.exp(exp)?;
        ExpDesc::Condition {
            jump_to_end: true,
            if_condition: false,
        }
        .discharge(&cond, self)?;

        let start_of_block = self.proto.byte_codes.len() - 1;
        for jump in self.jumps_to_block.drain(jump_to_block_count..) {
            self.proto.byte_codes[jump] =
                Bytecode::jump(i32::try_from(start_of_block - jump).map_err(|_| Error::LongJump)?);
        }

        let locals = self.locals.len();
        let cache_var_args = self.var_args.take();

        self.block(block)?;

        self.var_args = cache_var_args;
        self.stack_top -= u8::try_from(self.locals.len() - locals)
            .inspect_err(|_| log::error!("Failed to rewind stack top after `if`s block."))?;
        self.close_locals(locals);

        let jump_out_of_if = self.proto.byte_codes.len();
        self.proto.byte_codes.push(Bytecode::jump(0));

        let start_of_block = self.proto.byte_codes.len() - 1;
        for jump in self.jumps_to_block.drain(jump_to_block_count..) {
            self.proto.byte_codes[jump] =
                Bytecode::jump(i32::try_from(start_of_block - jump).map_err(|_| Error::LongJump)?);
        }

        self.stat_if(stat_if)?;

        let after_elses = self.proto.byte_codes.len();
        let offset = if after_elses != jump_out_of_if + 1 {
            self.proto.byte_codes[jump_out_of_if] = Bytecode::jump(
                i32::try_from(after_elses - jump_out_of_if - 1).map_err(|_| Error::LongJump)?,
            );
            0
        } else {
            self.proto.byte_codes.pop();
            1
        };

        for jump in self.jumps_to_end.drain(jump_to_end_count..) {
            self.proto.byte_codes[jump] = Bytecode::jump(
                i32::try_from(start_of_block - jump - offset).map_err(|_| Error::LongJump)?,
            );
        }

        Ok(())
    }

    fn fix_up_last_return(&mut self, fixed_arguments: u8) -> Result<(), Error> {
        if self
            .proto
            .byte_codes
            .pop()
            .filter(|bytecode| bytecode.get_opcode() == OpCode::ZeroReturn)
            .is_none()
        {
            unreachable!("Bytecode at the end of a function body should always be `ZeroReturn`.");
        };

        let locals = u8::try_from(self.locals.len())?;
        if self
            .proto
            .byte_codes
            .last()
            .filter(|bytecode| bytecode.get_opcode() == OpCode::TailCall)
            .is_some()
        {
            self.proto
                .byte_codes
                .push(Bytecode::return_bytecode(locals, 0, 0));
        } else {
            self.proto
                .byte_codes
                .push(Bytecode::return_bytecode(locals, 1, fixed_arguments + 1));
        }

        Ok(())
    }

    fn open_local(&mut self, name: &str) {
        self.locals.push(name.into());
        self.proto.locals.push(Local::new_no_end(
            name.into(),
            self.proto.byte_codes.len() + 1,
        ));
    }

    fn close_locals(&mut self, first_local_of_scope: usize) {
        let scope_end = self.proto.byte_codes.len() + 1;
        let mut closed_on_this_call = Vec::new();
        for local in self.locals.drain(first_local_of_scope..).rev() {
            let Some((i, local)) =
                self.proto
                    .locals
                    .iter_mut()
                    .enumerate()
                    .rev()
                    .find(|(i, proto_local)| {
                        proto_local.name() == local.as_ref() && !closed_on_this_call.contains(i)
                    })
            else {
                unreachable!(
                    "The local '{}' on the compile context must exist on the proto function.",
                    local
                );
            };
            closed_on_this_call.push(i);
            local.update_scope_end(scope_end);
        }
    }

    pub fn reserve_stack_top(&mut self) -> (u8, ExpDesc<'a>) {
        let top = self.stack_top;
        self.stack_top += 1;
        (top, ExpDesc::Local(usize::from(top)))
    }

    fn push_goto(&mut self, goto_label: GotoLabel<'a>) {
        self.gotos.push(goto_label);
    }

    fn push_label(&mut self, goto_label: GotoLabel<'a>) -> Result<(), Error> {
        if self
            .labels
            .iter()
            .any(|label| label.name == goto_label.name)
        {
            Err(Error::LabelRedefinition)
        } else {
            self.labels.push(goto_label);
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct GotoLabel<'a> {
    pub name: &'a str,
    pub bytecode: usize,
    pub nvar: usize,
}
