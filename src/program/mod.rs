mod binops;
mod compile_context;
mod error;
mod exp_desc;
mod parlist;
#[cfg(test)]
mod tests;

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use binops::Binop;
use compile_context::GotoLabel;
use parlist::Parlist;

use crate::{
    byte_code::ByteCode,
    ext::Unescape,
    parser::{Parser, Token, TokenType},
    Closure,
};

use super::value::Value;

pub use self::error::Error;
use self::{compile_context::CompileContext, exp_desc::ExpDesc};

macro_rules! make_deconstruct {
    ($($name:ident($token:pat$(,)?)),+$(,)?) => {
        [$($name @ Token {
            tokens: _,
            token_type: $token,
        },)+]
    };
}

#[derive(Debug, Default)]
pub struct Program {
    pub(super) constants: Vec<Value>,
    pub(super) byte_codes: Vec<ByteCode>,
    pub(super) functions: Vec<Value>,
}

impl Program {
    pub fn parse(program: &str) -> Result<Self, Error> {
        let chunk = Parser::parse(program)?;

        let mut program = Program::default();
        let mut compile_context = CompileContext::default();

        program.chunk(&chunk, &mut compile_context)?;

        Ok(program)
    }

    fn push_constant(&mut self, value: impl Into<Value>) -> Result<u8, Error> {
        let value = value.into();
        u8::try_from(
            self.constants
                .iter()
                .position(|v| v == &value)
                .unwrap_or_else(|| {
                    self.constants.push(value);
                    self.constants.len() - 1
                }),
        )
        .map_err(Error::from)
    }

    fn push_function(&mut self, value: impl Into<Value>) -> Result<u8, Error> {
        let value @ Value::Closure(_) = value.into() else {
            unreachable!("Should never be called with anything other than a closure.");
        };

        let new_position = self.functions.len();
        self.functions.push(value);

        u8::try_from(new_position).map_err(Error::from)
    }

    fn invert_last_test(&mut self) {
        let start_of_block = self.byte_codes.len();
        if let ByteCode::Test(_, test) = &mut self.byte_codes[start_of_block - 2] {
            *test ^= 1;
        } else {
            unreachable!("When inverting a test, the penultimate bytecode must be a `Test`.");
        }
    }

    fn make_if<'a>(
        &mut self,
        compile_context: &mut CompileContext<'a>,
        exp: &Token<'a>,
        block: &Token<'a>,
        stat_if: &Token<'a>,
    ) -> Result<(), Error> {
        assert!(
            compile_context.jumps_to_block.is_empty() && compile_context.jumps_to_end.is_empty(),
            "At the start of an `if`, compile context jumps must be empty."
        );

        let (top_index, top) = compile_context.reserve_stack_top();
        let Some(cond) = self.exp(exp, compile_context, &top)? else {
            unreachable!("Condition of if must always exist");
        };

        cond.discharge(
            &ExpDesc::IfCondition(usize::from(top_index), false),
            self,
            compile_context,
        )?;

        // Finish use of condition
        compile_context.stack_top = top_index;

        let start_of_block = self.byte_codes.len() - 1;
        for jump in compile_context.jumps_to_block.drain(..) {
            self.byte_codes[jump] =
                ByteCode::Jmp(i16::try_from(start_of_block - jump).map_err(|_| Error::LongJump)?);
        }

        // If the last test of an `if` is an `or` it's test flipped
        if compile_context.last_expdesc_was_or {
            self.invert_last_test();
        }
        compile_context.last_expdesc_was_or = false;
        compile_context.last_expdesc_was_relational = false;

        let locals = compile_context.locals.len();
        self.block(block, compile_context)?;
        compile_context.stack_top -= u8::try_from(compile_context.locals.len() - locals)
            .inspect_err(|_| log::error!("Failed to rewind stack top after `if`s block."))?;
        compile_context.locals.truncate(locals);

        let end_of_block = self.byte_codes.len();
        // Update jump to have the correct number

        let jump_end = self.byte_codes.len();
        self.byte_codes.push(ByteCode::Jmp(0));

        let jumps = core::mem::take(&mut compile_context.jumps_to_end);

        self.stat_if(stat_if, compile_context)?;
        let end_of_if = self.byte_codes.len();

        let fix_up = if end_of_block == (end_of_if - 1) {
            // Last else, so no need to jump
            self.byte_codes.pop();
            end_of_block - 1
        } else {
            self.byte_codes[jump_end] = ByteCode::Jmp(
                i16::try_from(end_of_if - (jump_end + 1)).map_err(|_| Error::LongJump)?,
            );
            end_of_block
        };

        for jump in jumps {
            self.byte_codes[jump] =
                ByteCode::Jmp(i16::try_from(fix_up - jump).map_err(|_| Error::LongJump)?);
        }

        Ok(())
    }

    // Non-terminals
    fn chunk<'a>(
        &mut self,
        chunk: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match chunk.tokens.as_slice() {
            make_deconstruct!(block(TokenType::Block)) => {
                self.block(block, compile_context)?;
                if compile_context.gotos.is_empty() {
                    Ok(())
                } else {
                    for goto in compile_context.gotos.iter() {
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

    fn block<'a>(
        &mut self,
        block: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match block.tokens.as_slice() {
            make_deconstruct!(
                block_stat(TokenType::BlockStat),
                block_retstat(TokenType::BlockRetstat)
            ) => {
                let gotos = compile_context.gotos.len();
                let labels = compile_context.labels.len();

                self.block_stat(block_stat, compile_context)?;
                self.block_retstat(block_retstat, compile_context)?;

                let unmatched = compile_context
                    .gotos
                    .drain(gotos..)
                    .filter_map(|goto| {
                        if let Some(label) = compile_context
                            .labels
                            .iter()
                            .rev()
                            .find(|label| label.name == goto.name)
                        {
                            if label.bytecode != self.byte_codes.len() && label.nvar > goto.nvar {
                                return Some(Err(Error::GotoIntoScope));
                            }
                            let Ok(label_i) = isize::try_from(label.bytecode) else {
                                return Some(Err(Error::IntCoversion));
                            };
                            let Ok(goto_i) = isize::try_from(goto.bytecode) else {
                                return Some(Err(Error::IntCoversion));
                            };
                            let Ok(jump) = i16::try_from((label_i - 1) - goto_i) else {
                                return Some(Err(Error::LongJump));
                            };
                            self.byte_codes[goto.bytecode] = ByteCode::Jmp(jump);
                            None
                        } else {
                            Some(Ok(goto))
                        }
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                compile_context.gotos.extend(unmatched);

                compile_context.labels.truncate(labels);

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

    fn block_stat<'a>(
        &mut self,
        block: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(stat(TokenType::Stat), blockstat(TokenType::BlockStat)) => self
                .stat(stat, compile_context)
                .and_then(|()| self.block_stat(blockstat, compile_context)),
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

    fn block_retstat<'a>(
        &mut self,
        block_retstat: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match block_retstat.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(retstat(TokenType::Retstat)) => {
                self.retstat(retstat, compile_context)
            }
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

    fn stat<'a>(
        &mut self,
        stat: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match stat.tokens.as_slice() {
            make_deconstruct!(_semicolon(TokenType::SemiColon)) => Ok(()),
            make_deconstruct!(
                varlist(TokenType::Varlist),
                _assing(TokenType::Assign),
                explist(TokenType::Explist)
            ) => {
                let exp_descs = self.varlist(varlist, compile_context)?;
                self.explist(explist, compile_context, Some(&exp_descs))
            }
            make_deconstruct!(functioncall(TokenType::Functioncall)) => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                self.functioncall(functioncall, compile_context, &stack_top)?;
                compile_context.stack_top -= 1;

                Ok(())
            }
            make_deconstruct!(label(TokenType::Label)) => self.label(label, compile_context),
            make_deconstruct!(_break(TokenType::Break)) => match compile_context.breaks.as_mut() {
                Some(breaks) => {
                    let bytecode = self.byte_codes.len();
                    breaks.push(bytecode);
                    self.byte_codes.push(ByteCode::Jmp(0));
                    Ok(())
                }
                None => Err(Error::BreakOutsideLoop),
            },
            make_deconstruct!(_goto(TokenType::Goto), _name(TokenType::Name(name))) => {
                let bytecode = self.byte_codes.len();
                self.byte_codes.push(ByteCode::Jmp(0));

                compile_context.push_goto(GotoLabel {
                    name,
                    bytecode,
                    nvar: compile_context.locals.len(),
                });

                Ok(())
            }
            make_deconstruct!(
                _do(TokenType::Do),
                block(TokenType::Block),
                _end(TokenType::End)
            ) => {
                let locals = compile_context.locals.len();

                self.block(block, compile_context)?;

                compile_context.locals.truncate(locals);

                Ok(())
            }
            make_deconstruct!(
                _while(TokenType::While),
                exp(TokenType::Exp),
                _do(TokenType::Do),
                block(TokenType::Block),
                _end(TokenType::End)
            ) => {
                let mut jump_cache = core::mem::take(&mut compile_context.jumps_to_end);

                let (top_index, top) = compile_context.reserve_stack_top();
                let Some(cond) = self.exp(exp, compile_context, &top)? else {
                    unreachable!("Condition of while must always exist)")
                };
                cond.discharge(
                    &ExpDesc::IfCondition(usize::from(top_index), false),
                    self,
                    compile_context,
                )?;

                // Finish use of condition
                compile_context.stack_top = top_index;

                let locals = compile_context.locals.len();
                let cache_breaks = compile_context.breaks.replace(Vec::with_capacity(16));

                self.block(block, compile_context)?;
                compile_context.stack_top -= u8::try_from(compile_context.locals.len() - locals)
                    .inspect_err(|_| {
                        log::error!("Failed to rewind stack top after `while`s block.")
                    })?;

                core::mem::swap(&mut compile_context.jumps_to_end, &mut jump_cache);
                assert_eq!(
                    jump_cache.len(),
                    1,
                    "While loops should only have one conditional jump."
                );
                let jump_end = self.byte_codes.len();

                self.byte_codes[jump_cache[0]] = ByteCode::Jmp(
                    i16::try_from(jump_end - jump_cache[0]).map_err(|_| Error::LongJump)?,
                );

                self.byte_codes.push(ByteCode::Jmp(
                    i16::try_from(isize::try_from(jump_cache[0])? - isize::try_from(jump_end)? - 2)
                        .map_err(|_| Error::LongJump)?,
                ));

                let Some(breaks) = compile_context.breaks.take() else {
                    unreachable!(
                        "Compile Context breaks should only ever be None outside of loops."
                    );
                };
                for break_bytecode in breaks {
                    self.byte_codes[break_bytecode] = ByteCode::Jmp(
                        i16::try_from(jump_end - break_bytecode).map_err(|_| Error::LongJump)?,
                    );
                }

                compile_context.locals.truncate(locals);
                compile_context.breaks = cache_breaks;

                Ok(())
            }
            make_deconstruct!(
                _repeat(TokenType::Repeat),
                block(TokenType::Block),
                _until(TokenType::Until),
                exp(TokenType::Exp)
            ) => {
                let mut jump_cache = core::mem::take(&mut compile_context.jumps_to_end);

                let locals = compile_context.locals.len();
                let stack_top = compile_context.stack_top;
                let repeat_start = self.byte_codes.len();

                self.block(block, compile_context)?;

                let (top_index, top) = compile_context.reserve_stack_top();
                let Some(cond) = self.exp(exp, compile_context, &top)? else {
                    unreachable!("Condition of repeat must always exist");
                };
                cond.discharge(
                    &ExpDesc::IfCondition(usize::from(top_index), false),
                    self,
                    compile_context,
                )?;

                compile_context.locals.truncate(locals);
                compile_context.stack_top = stack_top;

                core::mem::swap(&mut compile_context.jumps_to_end, &mut jump_cache);
                assert_eq!(
                    jump_cache.len(),
                    1,
                    "Repeat should only ever have 1 conditional jump."
                );

                let repeat_end = self.byte_codes.len();
                self.byte_codes[jump_cache[0]] = ByteCode::Jmp(
                    i16::try_from(isize::try_from(repeat_start)? - isize::try_from(repeat_end)?)
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
            ) => self.make_if(compile_context, exp, block, stat_if),
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
                let locals = compile_context.locals.len();

                // Names can't start with `?`, so using it for internal symbols
                compile_context.locals.push("?start".into());
                compile_context.locals.push("?end".into());
                compile_context.locals.push("?step".into());
                compile_context.locals.push((*name).into());

                let (for_stack, start_stack) = compile_context.reserve_stack_top();
                let Some(start) = self.exp(start, compile_context, &start_stack)? else {
                    unreachable!("Start of for must always exist.");
                };
                start.discharge(&start_stack, self, compile_context)?;

                let (_, end_stack) = compile_context.reserve_stack_top();
                let Some(end) = self.exp(end, compile_context, &end_stack)? else {
                    unreachable!("End of for must always exist.");
                };
                end.discharge(&end_stack, self, compile_context)?;

                let step = self.stat_forexp(stat_forexp, compile_context)?;
                let (_, step_stack) = compile_context.reserve_stack_top();
                step.discharge(&step_stack, self, compile_context)?;

                // Reserve 1 slot for counter
                compile_context.stack_top += 1;

                let counter_bytecode = self.byte_codes.len();
                self.byte_codes.push(ByteCode::ForPrepare(for_stack, 0));

                self.block(block, compile_context)?;

                let end_bytecode = self.byte_codes.len();
                self.byte_codes.push(ByteCode::ForLoop(
                    for_stack,
                    u16::try_from(end_bytecode - counter_bytecode)?,
                ));
                self.byte_codes[counter_bytecode] = ByteCode::ForPrepare(
                    for_stack,
                    u16::try_from(end_bytecode - (counter_bytecode + 1))?,
                );

                compile_context.stack_top = for_stack;
                compile_context.locals.truncate(locals);

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
                _funcname(TokenType::Funcname),
                _funcbody(TokenType::Funcbody)
            ) => unimplemented!("stat production"),
            make_deconstruct!(
                _local(TokenType::Local),
                _function(TokenType::Function),
                _name(TokenType::Name(name)),
                funcbody(TokenType::Funcbody)
            ) => {
                compile_context.locals.push((*name).into());
                let (_, function_body) = compile_context.reserve_stack_top();
                self.funcbody(funcbody, compile_context, &function_body)
            }
            make_deconstruct!(
                _local(TokenType::Local),
                attnamelist(TokenType::Attnamelist),
                stat_attexplist(TokenType::StatAttexplist)
            ) => {
                let (new_locals, exp_descs) = self.attnamelist(attnamelist, compile_context)?;
                self.stat_attexplist(stat_attexplist, compile_context, &exp_descs)?;
                // Adding the new names into `locals` to prevent
                // referencing the new name when you could be trying to shadow a
                // global or another local
                compile_context.locals.extend(new_locals);
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

    fn stat_if<'a>(
        &mut self,
        stat_if: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match stat_if.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _elseif(TokenType::Elseif),
                exp(TokenType::Exp),
                _then(TokenType::Then),
                block(TokenType::Block),
                stat_if(TokenType::StatIf)
            ) => self.make_if(compile_context, exp, block, stat_if),
            make_deconstruct!(_else(TokenType::Else), block(TokenType::Block)) => {
                let locals = compile_context.locals.len();
                self.block(block, compile_context)?;
                compile_context.stack_top -= u8::try_from(compile_context.locals.len() - locals)
                    .inspect_err(|_| {
                        log::error!("Failed to rewind stack top after `else`s block.")
                    })?;
                compile_context.locals.truncate(locals);

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

    fn stat_forexp<'a>(
        &mut self,
        stat_forexp: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<ExpDesc<'a>, Error> {
        match stat_forexp.tokens.as_slice() {
            [] => Ok(ExpDesc::Integer(1)),
            make_deconstruct!(_comma(TokenType::Comma), exp(TokenType::Exp)) => {
                let (_, top) = compile_context.reserve_stack_top();
                let Some(exp_desc) = self.exp(exp, compile_context, &top)? else {
                    unreachable!("Forexp must always exist.");
                };
                compile_context.stack_top -= 1;

                Ok(exp_desc)
            }
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

    fn stat_attexplist<'a>(
        &mut self,
        stat_attexplist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        exp_descs: &[ExpDesc<'a>],
    ) -> Result<(), Error> {
        match stat_attexplist.tokens.as_slice() {
            [] => {
                for exp_desc in exp_descs {
                    { ExpDesc::Nil }.discharge(exp_desc, self, compile_context)?;
                }
                Ok(())
            }
            make_deconstruct!(_assign(TokenType::Assign), explist(TokenType::Explist)) => {
                self.explist(explist, compile_context, Some(exp_descs))
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

    fn attnamelist<'a>(
        &mut self,
        attnamelist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(Vec<Value>, Vec<ExpDesc<'a>>), Error> {
        match attnamelist.tokens.as_slice() {
            make_deconstruct!(
                _name(TokenType::Name(name)),
                _attrib(TokenType::Attrib),
                attnamelist_cont(TokenType::AttnamelistCont)
            ) => {
                let mut new_locals = [(*name).into()].to_vec();

                let (_, stack_top) = compile_context.reserve_stack_top();
                let mut exp_descs = [stack_top].to_vec();

                self.attnamelist_cont(
                    attnamelist_cont,
                    compile_context,
                    &mut new_locals,
                    &mut exp_descs,
                )?;

                Ok((new_locals, exp_descs))
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

    fn attnamelist_cont<'a>(
        &mut self,
        attnamelist_cont: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        new_locals: &mut Vec<Value>,
        exp_descs: &mut Vec<ExpDesc<'a>>,
    ) -> Result<(), Error> {
        match attnamelist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                _name(TokenType::Name(name)),
                _attrib(TokenType::Attrib),
                attnamelist_cont(TokenType::AttnamelistCont)
            ) => {
                new_locals.push((*name).into());

                let (_, stack_top) = compile_context.reserve_stack_top();
                exp_descs.push(stack_top);

                self.attnamelist_cont(attnamelist_cont, compile_context, new_locals, exp_descs)
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

    fn attrib(&mut self, attrib: &Token, _compile_context: &CompileContext) -> Result<(), Error> {
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

    fn retstat<'a>(
        &mut self,
        retstat: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match retstat.tokens.as_slice() {
            make_deconstruct!(
                _return(TokenType::Return),
                retstat_explist(TokenType::RetstatExplist),
                retstat_end(TokenType::RetstatEnd)
            ) => {
                let stack_top = compile_context.stack_top;
                self.retstat_explist(retstat_explist, compile_context)?;
                let ret_count = compile_context.stack_top - stack_top;

                match ret_count {
                    0 => self.byte_codes.push(ByteCode::ZeroReturn),
                    1 => self
                        .byte_codes
                        .push(ByteCode::OneReturn(compile_context.stack_top - 1)),
                    _ => unimplemented!("Variadic return"),
                }

                compile_context.stack_top = stack_top;

                self.retstat_end(retstat_end, compile_context)?;

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

    fn retstat_explist<'a>(
        &mut self,
        retstat_explist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match retstat_explist.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(explist(TokenType::Explist)) => {
                self.explist(explist, compile_context, None)
            }
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

    fn retstat_end(
        &mut self,
        retstat_end: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
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

    fn label<'a>(
        &mut self,
        label: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match label.tokens.as_slice() {
            make_deconstruct!(
                _doublecolon1(TokenType::DoubleColon),
                _name(TokenType::Name(name)),
                _doublecolon2(TokenType::DoubleColon)
            ) => compile_context.push_label(GotoLabel {
                name,
                bytecode: self.byte_codes.len(),
                nvar: compile_context.locals.len(),
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

    fn funcname(
        &mut self,
        funcname: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match funcname.tokens.as_slice() {
            make_deconstruct!(
                _name(TokenType::Name(_)),
                _funcname_cont(TokenType::FuncnameCont),
                _funcname_end(TokenType::FuncnameEnd)
            ) => unimplemented!("funcname production"),
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
        &mut self,
        attrib: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match attrib.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _dot(TokenType::Dot),
                _name(TokenType::Name(_)),
                _funcname_cont(TokenType::FuncnameCont)
            ) => unimplemented!("funcname_cont production"),
            _ => {
                unreachable!(
                    "FuncnameCont did not match any of the productions. Had {:#?}.",
                    attrib
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
        funcname_end: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match funcname_end.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(_colon(TokenType::Colon), _name(TokenType::Name(_))) => {
                unimplemented!("funcname_end production")
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

    fn varlist<'a>(
        &mut self,
        varlist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<Vec<ExpDesc<'a>>, Error> {
        match varlist.tokens.as_slice() {
            make_deconstruct!(var(TokenType::Var), varlist_cont(TokenType::VarlistCont)) => {
                let mut exp_descs = Vec::new();

                let var_exp_desc = self.var(var, compile_context)?;
                exp_descs.push(var_exp_desc);

                self.varlist_cont(varlist_cont, compile_context, &mut exp_descs)?;

                Ok(exp_descs)
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

    fn varlist_cont<'a>(
        &mut self,
        varlist_cont: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        exp_descs: &mut Vec<ExpDesc<'a>>,
    ) -> Result<(), Error> {
        match varlist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                var(TokenType::Var),
                varlist_cont(TokenType::VarlistCont)
            ) => {
                let var_exp_desc = self.var(var, compile_context)?;
                exp_descs.push(var_exp_desc);

                self.varlist_cont(varlist_cont, compile_context, exp_descs)
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

    fn var<'a>(
        &mut self,
        var: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<ExpDesc<'a>, Error> {
        match var.tokens.as_slice() {
            make_deconstruct!(_name(TokenType::Name(name))) => self.name(name, compile_context),
            make_deconstruct!(
                prefixexp(TokenType::Prefixexp),
                _lsquare(TokenType::LSquare),
                exp(TokenType::Exp),
                _rsquare(TokenType::RSquare)
            ) => {
                let (_, prefix_top) = compile_context.reserve_stack_top();
                let Some(prefixexp_exp_desc) =
                    self.prefixexp(prefixexp, compile_context, &prefix_top)?
                else {
                    unreachable!("Prefixexp of var must always exist.");
                };

                let (_, top) = compile_context.reserve_stack_top();
                let Some(exp_exp_desc) = self.exp(exp, compile_context, &top)? else {
                    unreachable!("Exp of var must always exist.");
                };

                compile_context.stack_top -= 2;
                match prefixexp_exp_desc {
                    ExpDesc::Local(table) => Ok(ExpDesc::TableLocal(table, Box::new(exp_exp_desc))),
                    ExpDesc::Global(table) => {
                        Ok(ExpDesc::TableGlobal(table, Box::new(exp_exp_desc)))
                    }
                    _ => {
                        unimplemented!("Only local table access is available.");
                    }
                }
            }
            make_deconstruct!(
                prefixexp(TokenType::Prefixexp),
                _dot(TokenType::Dot),
                _name(TokenType::Name(name))
            ) => {
                let (_, prefix_top) = compile_context.reserve_stack_top();
                let Some(prefixexp_exp_desc) =
                    self.prefixexp(prefixexp, compile_context, &prefix_top)?
                else {
                    unreachable!("Prefix exp of var must always exist");
                };
                let name_exp_desc = self.string(name);
                compile_context.stack_top -= 1;

                match prefixexp_exp_desc {
                    ExpDesc::Local(table) => {
                        Ok(ExpDesc::TableLocal(table, Box::new(name_exp_desc)))
                    }
                    ExpDesc::Global(table) => {
                        Ok(ExpDesc::TableGlobal(table, Box::new(name_exp_desc)))
                    }
                    _ => {
                        unimplemented!("Only local table access is available.");
                    }
                }
            }

            _ => {
                unreachable!(
                    "Var did not match any of the productions. Had {:#?}.",
                    var.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn namelist(
        &mut self,
        namelist: &Token,
        compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match namelist.tokens.as_slice() {
            make_deconstruct!(
                _name(TokenType::Name(_)),
                namelist_cont(TokenType::NamelistCont)
            ) => self.namelist_cont(namelist_cont, compile_context),
            _ => {
                unreachable!(
                    "Namelist did not match any of the productions. Had {:#?}.",
                    namelist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn namelist_cont(
        &mut self,
        namelist_cont: &Token,
        compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match namelist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                _name(TokenType::Name(_)),
                namelist_cont(TokenType::NamelistCont)
            ) => self.namelist_cont(namelist_cont, compile_context),
            _ => {
                unreachable!(
                    "NamelistCont did not match any of the productions. Had {:#?}.",
                    namelist_cont
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn explist<'a>(
        &mut self,
        explist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        maybe_exp_descs: Option<&[ExpDesc<'a>]>,
    ) -> Result<(), Error> {
        let exp_top = compile_context.stack_top;
        match explist.tokens.as_slice() {
            make_deconstruct!(exp(TokenType::Exp), explist_cont(TokenType::ExplistCont)) => {
                let (exp_desc, tail) = maybe_exp_descs.map_or_else(
                    || (compile_context.reserve_stack_top().1, None),
                    |exp_descs| (exp_descs[0].clone(), Some(&exp_descs[1..])),
                );

                if let Some(exp) = self.exp(exp, compile_context, &exp_desc)? {
                    exp.discharge(&exp_desc, self, compile_context)?;

                    if compile_context.last_expdesc_was_relational {
                        self.byte_codes.push(ByteCode::LoadFalseSkip(exp_top));

                        let after_false_skip = self.byte_codes.len() - 2;
                        for jump in compile_context.jump_to_false.drain(..) {
                            self.byte_codes[jump] =
                                ByteCode::Jmp(i16::try_from(after_false_skip - jump)?);
                        }

                        match &mut self.byte_codes[after_false_skip - 1] {
                        ByteCode::EqualConstant(_, _, test) | ByteCode::LessThan(_, _, test) | ByteCode::LessEqual(_, _, test) | ByteCode::GreaterThanInteger(_, _, test) | ByteCode::GreaterEqualInteger(_, _, test) => {
                            *test ^= 1
                        }
                        other => unreachable!(
                            "The second to last bytecode should always be a relational comparison. Was {:?}", other
                        ),
                    }
                        self.byte_codes[after_false_skip] = ByteCode::Jmp(1);

                        self.byte_codes.push(ByteCode::LoadTrue(exp_top));
                    }
                };

                self.explist_cont(
                    explist_cont,
                    compile_context,
                    tail.filter(|slice| !slice.is_empty()),
                )
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

    fn explist_cont<'a>(
        &mut self,
        explist_cont: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        maybe_exp_descs: Option<&[ExpDesc<'a>]>,
    ) -> Result<(), Error> {
        let exp_top = compile_context.stack_top;
        match explist_cont.tokens.as_slice() {
            [] => {
                if let Some(exp_descs) = maybe_exp_descs {
                    for exp_desc in exp_descs {
                        { ExpDesc::Nil }.discharge(exp_desc, self, compile_context)?;
                    }
                }
                Ok(())
            }
            make_deconstruct!(
                _comma(TokenType::Comma),
                exp(TokenType::Exp),
                explist_cont(TokenType::ExplistCont)
            ) => {
                let (exp_desc, tail) = maybe_exp_descs.map_or_else(
                    || (compile_context.reserve_stack_top().1, None),
                    |exp_descs| (exp_descs[0].clone(), Some(&exp_descs[1..])),
                );

                if let Some(exp) = self.exp(exp, compile_context, &exp_desc)? {
                    exp.discharge(&exp_desc, self, compile_context)?;

                    if compile_context.last_expdesc_was_relational {
                        self.byte_codes.push(ByteCode::LoadFalseSkip(exp_top));

                        let after_false_skip = self.byte_codes.len() - 2;
                        for jump in compile_context.jump_to_false.drain(..) {
                            self.byte_codes[jump] =
                                ByteCode::Jmp(i16::try_from(after_false_skip - jump)?);
                        }

                        match &mut self.byte_codes[after_false_skip - 1] {
                        ByteCode::EqualConstant(_, _, test) | ByteCode::LessThan(_, _, test) | ByteCode::LessEqual(_, _, test) | ByteCode::GreaterThanInteger(_, _, test) | ByteCode::GreaterEqualInteger(_, _, test) => {
                            *test ^= 1
                        }
                        other => unreachable!(
                            "The second to last bytecode should always be a relational comparison. Was {:?}", other
                        ),
                    }
                        self.byte_codes[after_false_skip] = ByteCode::Jmp(1);

                        self.byte_codes.push(ByteCode::LoadTrue(exp_top));
                    }
                };

                self.explist_cont(
                    explist_cont,
                    compile_context,
                    tail.filter(|slice| !slice.is_empty()),
                )
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

    #[must_use = "ExpDesc might be constant values that need to be discharged"]
    fn exp<'a>(
        &mut self,
        exp: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        exp_desc: &ExpDesc<'a>,
    ) -> Result<Option<ExpDesc<'a>>, Error> {
        match exp.tokens.as_slice() {
            make_deconstruct!(_nil(TokenType::Nil)) => Ok(Some(self.nil())),
            make_deconstruct!(_false(TokenType::False)) => Ok(Some(self.boolean(false))),
            make_deconstruct!(_true(TokenType::True)) => Ok(Some(self.boolean(true))),
            make_deconstruct!(_string(TokenType::String(string))) => Ok(Some(self.string(string))),
            make_deconstruct!(_integer(TokenType::Integer(integer))) => {
                Ok(Some(self.integer(*integer)))
            }
            make_deconstruct!(_float(TokenType::Float(float))) => Ok(Some(self.float(*float))),
            make_deconstruct!(_dots(TokenType::Dots)) => unimplemented!("exp production"),
            make_deconstruct!(functiondef(TokenType::Functiondef)) => self
                .functiondef(functiondef, compile_context, exp_desc)
                .map(|_| None),
            make_deconstruct!(prefixexp(TokenType::Prefixexp)) => {
                self.prefixexp(prefixexp, compile_context, exp_desc)
            }
            make_deconstruct!(tableconstructor(TokenType::Tableconstructor)) => self
                .tableconstructor(tableconstructor, compile_context, exp_desc)
                .map(Some),
            make_deconstruct!(
                lhs(TokenType::Exp),
                op(TokenType::Binop),
                rhs(TokenType::Exp)
            ) => {
                let op = self.binop(op)?;

                let (lhs_dst, lhs_top) = match exp_desc {
                    ExpDesc::Local(dst) => (u8::try_from(*dst)?, exp_desc.clone()),
                    ExpDesc::Global(_) => compile_context.reserve_stack_top(),
                    _ => todo!("binop: see what other cases are needed"),
                };
                let Some(lhs) = self.exp(lhs, compile_context, &lhs_top)? else {
                    unreachable!("Left operand of binary operator must always exist.");
                };

                let (rhs_dst, rhs_top) = compile_context.reserve_stack_top();
                let Some(rhs) = self.exp(rhs, compile_context, &rhs_top)? else {
                    unreachable!("Right operand of binary operator must always exist.");
                };

                compile_context.stack_top -= if matches!(exp_desc, ExpDesc::Local(_)) {
                    1
                } else {
                    2
                };

                let func = match op {
                    TokenType::Add => binops::binop_add,
                    TokenType::Sub => binops::binop_sub,
                    TokenType::Mul => binops::binop_mul,
                    TokenType::Mod => binops::binop_mod,
                    TokenType::Pow => binops::binop_pow,
                    TokenType::Div => binops::binop_div,
                    TokenType::Idiv => binops::binop_idiv,
                    TokenType::BitAnd => binops::binop_bitand,
                    TokenType::BitOr => binops::binop_bitor,
                    TokenType::BitXor => binops::binop_bitxor,
                    TokenType::ShiftL => binops::binop_shiftl,
                    TokenType::ShiftR => binops::binop_shiftr,
                    TokenType::Or => binops::binop_or,
                    TokenType::And => binops::binop_and,
                    TokenType::Less => binops::binop_lt,
                    TokenType::Greater => binops::binop_gt,
                    TokenType::Leq => binops::binop_le,
                    TokenType::Geq => binops::binop_ge,
                    TokenType::Eq => binops::binop_eq,
                    TokenType::Neq => binops::binop_ne,
                    TokenType::Concat => binops::binop_concat,
                    other => unreachable!("{:?} is not a binary operator", other),
                };
                func(
                    self,
                    compile_context,
                    Binop {
                        expdesc: &lhs,
                        top: &lhs_top,
                        dst: lhs_dst,
                    },
                    Binop {
                        expdesc: &rhs,
                        top: &rhs_top,
                        dst: rhs_dst,
                    },
                )
                .map(Some)
            }
            make_deconstruct!(op(TokenType::Unop), rhs(TokenType::Exp)) => {
                let op = self.unop(op)?;

                let (stack_top, top) = match exp_desc {
                    ExpDesc::Local(top) => (*top, exp_desc.clone()),
                    _ => {
                        let (stack_top, top) = compile_context.reserve_stack_top();
                        (usize::from(stack_top), top)
                    }
                };
                let Some(exp_exp_desc) = self.exp(rhs, compile_context, &top)? else {
                    unreachable!("Operand of unary operator must always exist");
                };

                match (exp_exp_desc, op) {
                    (ExpDesc::Local(src), TokenType::Not) => Ok(ExpDesc::Unop(ByteCode::Not, src)),
                    (global @ ExpDesc::Global(_), TokenType::Not) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::Not, stack_top))
                    }
                    (ExpDesc::Nil, TokenType::Not) => Ok(ExpDesc::Boolean(true)),
                    (ExpDesc::Boolean(boolean), TokenType::Not) => Ok(ExpDesc::Boolean(!boolean)),
                    (other, TokenType::Not) => {
                        // TODO check what to do here
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                    (ExpDesc::Local(src), TokenType::Len) => Ok(ExpDesc::Unop(ByteCode::Len, src)),
                    (global @ ExpDesc::Global(_), TokenType::Len) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::Len, stack_top))
                    }
                    (ExpDesc::String(string), TokenType::Len) => {
                        let string = string.unescape()?;
                        Ok(ExpDesc::Integer(i64::try_from(string.len())?))
                    }
                    (other, TokenType::Len) => {
                        // TODO check what to do here
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                    (ExpDesc::Local(src), TokenType::Sub) => Ok(ExpDesc::Unop(ByteCode::Neg, src)),
                    (global @ ExpDesc::Global(_), TokenType::Sub) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::Neg, stack_top))
                    }
                    (ExpDesc::Integer(integer), TokenType::Sub) => Ok(ExpDesc::Integer(-integer)),
                    (ExpDesc::Float(float), TokenType::Sub) => Ok(ExpDesc::Float(-float)),
                    (other, TokenType::Sub) => {
                        // TODO check what to do here
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                    (ExpDesc::Local(src), TokenType::BitXor) => {
                        Ok(ExpDesc::Unop(ByteCode::BitNot, src))
                    }
                    (global @ ExpDesc::Global(_), TokenType::BitXor) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::BitNot, stack_top))
                    }
                    (ExpDesc::Integer(integer), TokenType::BitXor) => {
                        Ok(ExpDesc::Integer(!integer))
                    }
                    (other, TokenType::BitXor) => {
                        // TODO check what to do here
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                    _ => {
                        unimplemented!("Could't process Unop.");
                    }
                }
                .map(Some)
            }
            _ => {
                unreachable!(
                    "Exp did not match any of the productions. Had {:#?}.",
                    exp.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn prefixexp<'a>(
        &mut self,
        prefixexp: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        exp_desc: &ExpDesc<'a>,
    ) -> Result<Option<ExpDesc<'a>>, Error> {
        match prefixexp.tokens.as_slice() {
            make_deconstruct!(var(TokenType::Var)) => self.var(var, compile_context).map(Some),
            make_deconstruct!(functioncall(TokenType::Functioncall)) => self
                .functioncall(functioncall, compile_context, exp_desc)
                .map(|()| None),
            make_deconstruct!(
                _lparen(TokenType::LParen),
                exp(TokenType::Exp),
                _rparen(TokenType::RParen)
            ) => self.exp(exp, compile_context, exp_desc),
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

    fn functioncall<'a>(
        &mut self,
        functioncall: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        function_discharge_location: &ExpDesc<'a>,
    ) -> Result<(), Error> {
        assert!(!compile_context.last_expdesc_was_or);
        assert!(!compile_context.last_expdesc_was_relational);

        match functioncall.tokens.as_slice() {
            make_deconstruct!(prefixexp(TokenType::Prefixexp), args(TokenType::Args)) => {
                if let prefixexp_top @ ExpDesc::Local(func_index_usize) =
                    function_discharge_location
                {
                    let func_index =
                        u8::try_from(*func_index_usize).map_err(|_| Error::StackOverflow)?;
                    let Some(top) = self.prefixexp(prefixexp, compile_context, prefixexp_top)?
                    else {
                        unreachable!("Function must always exist.");
                    };
                    top.discharge(prefixexp_top, self, compile_context)?;

                    let stack_before_args = compile_context.stack_top;
                    self.args(args, compile_context)?;

                    self.byte_codes.push(ByteCode::Call(
                        func_index,
                        (compile_context.stack_top - 1) - func_index,
                    ));
                    compile_context.stack_top = stack_before_args;

                    compile_context.last_expdesc_was_or = false;
                    compile_context.last_expdesc_was_relational = false;

                    Ok(())
                } else {
                    unimplemented!("functioncall can only be discharged on local for now")
                }
            }
            make_deconstruct!(
                _prefixexp(TokenType::Prefixexp),
                _colon(TokenType::Colon),
                _name(TokenType::Name(_)),
                _args(TokenType::Args)
            ) => unimplemented!("functioncall production"),
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

    fn args<'a>(
        &mut self,
        args: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match args.tokens.as_slice() {
            make_deconstruct!(
                _lparen(TokenType::LParen),
                args_explist(TokenType::ArgsExplist),
                _rparen(TokenType::RParen)
            ) => {
                let count_end_jumps = compile_context.jumps_to_end.len();
                let count_block_jumps = compile_context.jumps_to_block.len();
                let count_false_jumps = compile_context.jump_to_false.len();

                let res = self.args_explist(args_explist, compile_context);

                assert_eq!(compile_context.jumps_to_end.len(), count_end_jumps);
                assert_eq!(compile_context.jumps_to_block.len(), count_block_jumps);
                assert_eq!(compile_context.jump_to_false.len(), count_false_jumps);

                res
            }
            make_deconstruct!(tableconstructor(TokenType::Tableconstructor)) => {
                let (_, top) = compile_context.reserve_stack_top();
                self.tableconstructor(tableconstructor, compile_context, &top)?;
                // Already on top of the stack, no need to move
                Ok(())
            }
            make_deconstruct!(_string(TokenType::String(string))) => self.string(string).discharge(
                &compile_context.reserve_stack_top().1,
                self,
                compile_context,
            ),
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

    fn args_explist<'a>(
        &mut self,
        args_explist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match args_explist.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(explist(TokenType::Explist)) => {
                self.explist(explist, compile_context, None)
            }
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

    fn functiondef<'a>(
        &mut self,
        functiondef: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        exp_desc: &ExpDesc<'a>,
    ) -> Result<(), Error> {
        match functiondef.tokens.as_slice() {
            make_deconstruct!(
                _function(TokenType::Function),
                funcbody(TokenType::Funcbody)
            ) => self.funcbody(funcbody, compile_context, exp_desc),
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

    fn funcbody<'a>(
        &mut self,
        funcbody: &Token<'a>,
        _compile_context: &mut CompileContext<'a>,
        exp_desc: &ExpDesc<'a>,
    ) -> Result<(), Error> {
        match funcbody.tokens.as_slice() {
            make_deconstruct!(
                _lparen(TokenType::LParen),
                funcbody_parlist(TokenType::FuncbodyParlist),
                _rparen(TokenType::RParen),
                block(TokenType::Block),
                _end(TokenType::End),
            ) => {
                if let ExpDesc::Local(top_index) = exp_desc {
                    let top_index = u8::try_from(*top_index)?;

                    let parlist = self.funcbody_parlist(funcbody_parlist)?;
                    let parlist_name_count = parlist.names.len();

                    let mut func_program = Program::default();
                    let mut func_compile_context = CompileContext::default();

                    func_compile_context.locals.extend(parlist.names);
                    func_compile_context.stack_top = u8::try_from(parlist_name_count)?;

                    func_program.block(block, &mut func_compile_context)?;

                    func_program.byte_codes.push(ByteCode::ZeroReturn);

                    let closure_position = self.push_function(Rc::new(Closure::new(
                        func_program,
                        parlist_name_count,
                        parlist.variadic_args,
                    )))?;
                    self.byte_codes
                        .push(ByteCode::Closure(top_index, closure_position));

                    Ok(())
                } else {
                    unimplemented!("Can only create function bodies into locals for now");
                }
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

    fn funcbody_parlist(&mut self, funcbody_parlist: &Token<'_>) -> Result<Parlist, Error> {
        match funcbody_parlist.tokens.as_slice() {
            [] => Ok(Parlist::default()),
            make_deconstruct!(parlist(TokenType::Parlist)) => {
                let mut func_parlist = Parlist::default();
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

    fn parlist(&mut self, parlist: &Token<'_>, func_parlist: &mut Parlist) -> Result<(), Error> {
        match parlist.tokens.as_slice() {
            make_deconstruct!(
                _name(TokenType::Name(name)),
                parlist_cont(TokenType::ParlistCont)
            ) => {
                func_parlist.names.push((*name).into());
                self.parlist_cont(parlist_cont, func_parlist)
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

    fn parlist_cont(
        &mut self,
        parlist_cont: &Token<'_>,
        func_parlist: &mut Parlist,
    ) -> Result<(), Error> {
        match parlist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma(TokenType::Comma),
                _name(TokenType::Name(name)),
                parlist_cont(TokenType::ParlistCont)
            ) => {
                func_parlist.names.push((*name).into());
                self.parlist_cont(parlist_cont, func_parlist)
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

    fn tableconstructor<'a>(
        &mut self,
        tableconstructor: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        exp_desc: &ExpDesc<'a>,
    ) -> Result<ExpDesc<'a>, Error> {
        match tableconstructor.tokens.as_slice() {
            make_deconstruct!(
                _lcurly(TokenType::LCurly),
                tableconstructor_fieldlist(TokenType::TableconstructorFieldlist),
                _rcurly(TokenType::RCurly)
            ) => {
                let dst = match exp_desc {
                    ExpDesc::Local(dst) => u8::try_from(*dst)?,
                    ExpDesc::Global(_) => {
                        let dst = compile_context.stack_top;
                        compile_context.stack_top += 1;
                        dst
                    }
                    _ => {
                        unimplemented!("Only table creation on stack is supported.");
                    }
                };

                let table_initialization_bytecode_position = self.byte_codes.len();
                self.byte_codes.push(ByteCode::NewTable(0, 0, 0));

                let (array_items, table_items) = self.tableconstructor_fieldlist(
                    tableconstructor_fieldlist,
                    compile_context,
                    dst,
                )?;

                self.byte_codes[table_initialization_bytecode_position] =
                    ByteCode::NewTable(dst, array_items, table_items);
                if array_items > 0 {
                    self.byte_codes.push(ByteCode::SetList(dst, array_items));
                }

                // Clear the list values
                compile_context.stack_top -= array_items;
                if let ExpDesc::Global(_) = exp_desc {
                    compile_context.stack_top -= 1;
                }

                Ok(ExpDesc::Local(usize::from(dst)))
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

    fn tableconstructor_fieldlist<'a>(
        &mut self,
        tableconstructor_fieldlist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match tableconstructor_fieldlist.tokens.as_slice() {
            [] => Ok((0, 0)),
            make_deconstruct!(fieldlist(TokenType::Fieldlist)) => {
                self.fieldlist(fieldlist, compile_context, table)
            }
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

    fn fieldlist<'a>(
        &mut self,
        fieldlist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match fieldlist.tokens.as_slice() {
            make_deconstruct!(
                field(TokenType::Field),
                fieldlist_cont(TokenType::FieldlistCont)
            ) => {
                let (array_item, table_item) = self.field(field, compile_context, table)?;
                let (array_len, table_len) =
                    self.fieldlist_cont(fieldlist_cont, compile_context, table)?;

                Ok((array_len + array_item, table_len + table_item))
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

    fn fieldlist_cont<'a>(
        &mut self,
        fieldlist_cont: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match fieldlist_cont.tokens.as_slice() {
            [] => Ok((0, 0)),
            make_deconstruct!(
                fieldsep(TokenType::Fieldsep),
                field(TokenType::Field),
                fieldlist_cont(TokenType::FieldlistCont)
            ) => self
                .fieldsep(fieldsep)
                .and_then(|()| self.field(field, compile_context, table))
                .and_then(|(array_item, table_item)| {
                    self.fieldlist_cont(fieldlist_cont, compile_context, table)
                        .map(|(array_len, table_len)| {
                            (array_len + array_item, table_len + table_item)
                        })
                }),
            make_deconstruct!(fieldsep(TokenType::Fieldsep)) => {
                self.fieldsep(fieldsep).map(|()| (0, 0))
            }
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

    fn field<'a>(
        &mut self,
        field: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match field.tokens.as_slice() {
            make_deconstruct!(
                _lsquare(TokenType::LSquare),
                key(TokenType::Exp),
                _rsquare(TokenType::RSquare),
                _assing(TokenType::Assign),
                exp(TokenType::Exp)
            ) => {
                let (dst, key_top) = compile_context.reserve_stack_top();
                let Some(key) = self.exp(key, compile_context, &key_top)? else {
                    unreachable!("Field key must always exist.");
                };
                key.discharge(&key_top, self, compile_context)?;
                let (_, exp_top) = compile_context.reserve_stack_top();
                let Some(exp) = self.exp(exp, compile_context, &exp_top)? else {
                    unreachable!("Exp of keyd field must always exist.");
                };
                exp.discharge(&exp_top, self, compile_context)?;

                match (key_top, exp_top) {
                    (ExpDesc::Local(key), ExpDesc::Local(value)) => {
                        self.byte_codes.push(ByteCode::SetTable(
                            table,
                            u8::try_from(key)?,
                            u8::try_from(value)?,
                        ));

                        compile_context.stack_top = dst;

                        Ok((0, 1))
                    }
                    _ => unimplemented!("field production"),
                }
            }
            make_deconstruct!(
                _name(TokenType::Name(name)),
                _assign(TokenType::Assign),
                exp(TokenType::Exp)
            ) => {
                let (dst, top) = compile_context.reserve_stack_top();
                let Some(exp) = self.exp(exp, compile_context, &top)? else {
                    unreachable!("Exp of named field must always exist.");
                };
                exp.discharge(&top, self, compile_context)?;

                let constant = self.push_constant(*name)?;
                self.byte_codes
                    .push(ByteCode::SetField(table, constant, dst));

                compile_context.stack_top = dst;
                Ok((0, 1))
            }
            make_deconstruct!(exp(TokenType::Exp)) => {
                let (_, top) = compile_context.reserve_stack_top();
                let Some(exp) = self.exp(exp, compile_context, &top)? else {
                    unreachable!("Exp of field must always exist.");
                };
                exp.discharge(&top, self, compile_context)?;

                Ok((1, 0))
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

    fn binop<'a>(&mut self, binop: &Token<'a>) -> Result<TokenType<'a>, Error> {
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

    fn unop<'a>(&mut self, unop: &Token<'a>) -> Result<TokenType<'a>, Error> {
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
    fn nil<'a>(&mut self) -> ExpDesc<'a> {
        ExpDesc::Nil
    }

    #[inline(always)]
    fn boolean<'a>(&mut self, boolean: bool) -> ExpDesc<'a> {
        ExpDesc::Boolean(boolean)
    }

    #[inline(always)]
    fn integer<'a>(&mut self, integer: i64) -> ExpDesc<'a> {
        ExpDesc::Integer(integer)
    }

    #[inline(always)]
    fn float<'a>(&mut self, float: f64) -> ExpDesc<'a> {
        ExpDesc::Float(float)
    }

    #[inline(always)]
    fn string<'a>(&mut self, string: &'a str) -> ExpDesc<'a> {
        ExpDesc::String(string)
    }

    #[inline(always)]
    fn name<'a>(
        &mut self,
        name: &'a str,
        compile_context: &CompileContext,
    ) -> Result<ExpDesc<'a>, Error> {
        if let Some(i) = compile_context
            .locals
            .iter()
            .rposition(|v| v == &Value::from(name))
        {
            Ok(ExpDesc::Local(i))
        } else {
            self.push_constant(name)
                .map(|i| ExpDesc::Global(usize::from(i)))
        }
    }
}
