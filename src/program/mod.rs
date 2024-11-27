mod byte_code;
mod compile_context;
mod error;
mod exp_desc;
#[cfg(test)]
mod tests;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    ext::Unescape,
    parser::{Parser, Token, TokenType},
};

use super::value::Value;

pub use self::{byte_code::ByteCode, error::Error};
use self::{compile_context::CompileContext, exp_desc::ExpDesc};

macro_rules! make_deconstruct {
    ($($name:ident => $token:pat),+$(,)?) => {
        [$($name @ Token {
            tokens: _,
            token_type: $token,
        },)+]
    };
}

#[derive(Debug)]
pub struct Program {
    pub(super) constants: Vec<Value>,
    pub(super) byte_codes: Vec<ByteCode>,
}

impl Program {
    pub fn parse(program: &str) -> Result<Self, Error> {
        let chunk = Parser::parse(program)?;

        let mut program = Program {
            constants: Vec::new(),
            byte_codes: Vec::new(),
        };
        let mut compile_context = CompileContext {
            stack_top: 0,
            locals: Vec::new(),
        };

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

    // Non-terminals
    fn chunk(
        &mut self,
        chunk: &Token<'_>,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match chunk.tokens.as_slice() {
            make_deconstruct!(block=> TokenType::Block) => self.block(block, compile_context),
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

    fn block(
        &mut self,
        block: &Token<'_>,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match block.tokens.as_slice() {
            make_deconstruct!(
                block_stat => TokenType::BlockStat,
                block_retstat=> TokenType::BlockRetstat
            ) => self
                .block_stat(block_stat, compile_context)
                .and_then(|()| self.block_retstat(block_retstat, compile_context)),
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

    fn block_stat(
        &mut self,
        block: &Token<'_>,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(stat=> TokenType::Stat, blockstat=> TokenType::BlockStat) => self
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

    fn block_retstat(
        &mut self,
        block_retstat: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match block_retstat.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(_retstat=> TokenType::Retstat) => Err(Error::Unimplemented),
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

    fn stat(
        &mut self,
        stat: &Token<'_>,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match stat.tokens.as_slice() {
            make_deconstruct!(_semicolon=> TokenType::SemiColon) => Ok(()),
            make_deconstruct!(
                varlist=> TokenType::Varlist,
                _assing=> TokenType::Assign,
                explist=> TokenType::Explist
            ) => {
                let exp_descs = self.varlist(varlist, compile_context)?;
                self.explist(explist, compile_context, Some(&exp_descs))
            }
            make_deconstruct!(functioncall=> TokenType::Functioncall) => {
                self.functioncall(functioncall, compile_context)
            }
            make_deconstruct!(_label=> TokenType::Label) => Err(Error::Unimplemented),
            make_deconstruct!(_break=> TokenType::Break) => Err(Error::Unimplemented),
            make_deconstruct!(_goto=> TokenType::Goto, _name=> TokenType::Name(_)) => {
                Err(Error::Unimplemented)
            }
            make_deconstruct!(
                _do=> TokenType::Do,
                _block=> TokenType::Block,
                _end=> TokenType::End
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _while=> TokenType::While,
                _exp=> TokenType::Exp,
                _do=> TokenType::Do,
                _block=> TokenType::Block,
                _end=> TokenType::End
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _repeat=> TokenType::Repeat,
                _block=> TokenType::Block,
                _until=> TokenType::Until,
                _exp=> TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _if=> TokenType::If,
                _exp=> TokenType::Exp,
                _then=> TokenType::Then,
                _block=> TokenType::Block,
                _stat_elseif=> TokenType::StatElseif,
                _stat_else=> TokenType::StatElse,
                _end=> TokenType::End
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _for=> TokenType::For,
                _name=> TokenType::Name(_),
                _assign=> TokenType::Assign,
                _exp1=> TokenType::Exp,
                _comma=> TokenType::Comma,
                _exp2=> TokenType::Exp,
                _stat_forexp=> TokenType::StatForexp,
                _do=> TokenType::Do,
                _block=> TokenType::Block,
                _end=> TokenType::End
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _for=> TokenType::For,
                _namelist=> TokenType::Namelist,
                _in=> TokenType::In,
                _explist=> TokenType::Explist,
                _do=> TokenType::Do,
                _block=> TokenType::Block,
                _end=> TokenType::End
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _function=> TokenType::Function,
                _funcname=> TokenType::Funcname,
                _funcbody=> TokenType::Funcbody
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _local=> TokenType::Local,
                _function=> TokenType::Function,
                _name=> TokenType::Name(_),
                _funcbody=> TokenType::Funcbody
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _local=> TokenType::Local,
                attnamelist=> TokenType::Attnamelist,
                stat_attexplist=> TokenType::StatAttexplist
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

    fn stat_elseif(
        &mut self,
        stat_elseif: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match stat_elseif.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _elseif=> TokenType::Elseif,
                _exp=> TokenType::Exp,
                _then=> TokenType::Then,
                _block=> TokenType::Block,
                _stat_elseif=> TokenType::StatElseif
            ) => Err(Error::Unimplemented),
            _ => {
                unreachable!(
                    "StatElseif did not match any of the productions. Had {:#?}.",
                    stat_elseif
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn stat_else(
        &mut self,
        stat_else: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match stat_else.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _else=> TokenType::Else,
                _block=> TokenType::Block) => Err(Error::Unimplemented),
            _ => {
                unreachable!(
                    "StatElse did not match any of the productions. Had {:#?}.",
                    stat_else
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn stat_forexp(
        &mut self,
        stat_else: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match stat_else.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma=> TokenType::Comma,
                _exp=> TokenType::Exp) => Err(Error::Unimplemented),
            _ => {
                unreachable!(
                    "StatForexp did not match any of the productions. Had {:#?}.",
                    stat_else
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn stat_attexplist(
        &mut self,
        stat_attexplist: &Token<'_>,
        compile_context: &mut CompileContext,
        exp_descs: &[ExpDesc],
    ) -> Result<(), Error> {
        match stat_attexplist.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _assign=> TokenType::Assign,
                explist=> TokenType::Explist
            ) => self.explist(explist, compile_context, Some(exp_descs)),
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
        compile_context: &mut CompileContext,
    ) -> Result<(Vec<Value>, Vec<ExpDesc<'a>>), Error> {
        match attnamelist.tokens.as_slice() {
            make_deconstruct!(
                _name=> TokenType::Name(name),
                _attrib=> TokenType::Attrib,
                attnamelist_cont=> TokenType::AttnamelistCont
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
        compile_context: &mut CompileContext,
        new_locals: &mut Vec<Value>,
        exp_descs: &mut Vec<ExpDesc<'a>>,
    ) -> Result<(), Error> {
        match attnamelist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma=> TokenType::Comma,
                _name=> TokenType::Name(name),
                _attrib=> TokenType::Attrib,
                attnamelist_cont=> TokenType::AttnamelistCont
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
                _less=> TokenType::Less,
                _name=> TokenType::Name(_),
                _greater=> TokenType::Greater
            ) => Err(Error::Unimplemented),
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

    fn retstat(&mut self, retstat: &Token, _compile_context: &CompileContext) -> Result<(), Error> {
        match retstat.tokens.as_slice() {
            make_deconstruct!(
                _return=> TokenType::Return,
                _retstat_explist=> TokenType::RetstatExplist,
                _retstat_end=> TokenType::RetstatEnd
            ) => Err(Error::Unimplemented),
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

    fn retstat_explist(
        &mut self,
        retstat_explist: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match retstat_explist.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(_explist=> TokenType::Explist) => Err(Error::Unimplemented),
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
            make_deconstruct!(_semicolon=> TokenType::SemiColon) => Ok(()),
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

    fn label(&mut self, label: &Token, _compile_context: &CompileContext) -> Result<(), Error> {
        match label.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _doublecolon1=> TokenType::DoubleColon,
                _name=> TokenType::Name(_),
                _doublecolon2=> TokenType::DoubleColon
            ) => Err(Error::Unimplemented),
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
                _name=> TokenType::Name(_),
                _funcname_cont=> TokenType::FuncnameCont,
                _funcname_end=> TokenType::FuncnameEnd
            ) => Err(Error::Unimplemented),
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
                _dot=> TokenType::Dot,
                _name=> TokenType::Name(_),
                _funcname_cont=> TokenType::FuncnameCont
            ) => Err(Error::Unimplemented),
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
            make_deconstruct!(_colon=> TokenType::Colon, _name=> TokenType::Name(_)) => {
                Err(Error::Unimplemented)
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
        compile_context: &mut CompileContext,
    ) -> Result<Vec<ExpDesc<'a>>, Error> {
        match varlist.tokens.as_slice() {
            make_deconstruct!(var=> TokenType::Var, varlist_cont=> TokenType::VarlistCont) => {
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
        compile_context: &mut CompileContext,
        exp_descs: &mut Vec<ExpDesc<'a>>,
    ) -> Result<(), Error> {
        match varlist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma=> TokenType::Comma,
                var=> TokenType::Var,
                varlist_cont=> TokenType::VarlistCont
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
        compile_context: &mut CompileContext,
    ) -> Result<ExpDesc<'a>, Error> {
        match var.tokens.as_slice() {
            make_deconstruct!(_name=> TokenType::Name(name)) => self.name(name, compile_context),
            make_deconstruct!(
                var=> TokenType::Var,
                _lsquare=> TokenType::LSquare,
                exp=> TokenType::Exp,
                _rsquare=> TokenType::RSquare
            ) => {
                let var_exp_desc = self.var(var, compile_context)?;
                let (_, top) = compile_context.reserve_stack_top();
                let exp_exp_desc = self.exp(exp, compile_context, &top)?;
                compile_context.stack_top -= 1;
                match var_exp_desc {
                    ExpDesc::Local(table) => Ok(ExpDesc::TableLocal(table, Box::new(exp_exp_desc))),
                    ExpDesc::Global(table) => {
                        Ok(ExpDesc::TableGlobal(table, Box::new(exp_exp_desc)))
                    }
                    _ => {
                        log::error!("Only local table access is available.");
                        Err(Error::Unimplemented)
                    }
                }
            }
            make_deconstruct!(
                _functioncall=> TokenType::Functioncall,
                _lsquare=> TokenType::LSquare,
                _exp=> TokenType::Exp,
                _rsquare=> TokenType::RSquare
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lparen=> TokenType::LParen,
                _exp1=> TokenType::Exp,
                _rparen=> TokenType::RParen,
                _lsquare=> TokenType::LSquare,
                _exp2=> TokenType::Exp,
                _rsquare=> TokenType::RSquare
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                var=> TokenType::Var,
                _dot=> TokenType::Dot,
                _name=> TokenType::Name(name)
            ) => {
                let var_exp_desc = self.var(var, compile_context)?;

                let name_exp_desc = self.string(name);
                match var_exp_desc {
                    ExpDesc::Local(table) => {
                        Ok(ExpDesc::TableLocal(table, Box::new(name_exp_desc)))
                    }
                    ExpDesc::Global(table) => {
                        Ok(ExpDesc::TableGlobal(table, Box::new(name_exp_desc)))
                    }
                    _ => {
                        log::error!("Only local table access is available.");
                        Err(Error::Unimplemented)
                    }
                }
            }
            make_deconstruct!(
                _functioncall=> TokenType::Functioncall,
                _dot=> TokenType::Dot,
                _name=> TokenType::Name(_)
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lparen=> TokenType::LParen,
                _exp=> TokenType::Exp,
                _rparen=> TokenType::RParen,
                _dot=> TokenType::Dot,
                _name=> TokenType::Name(_)
            ) => Err(Error::Unimplemented),
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
                _name=> TokenType::Name(_),
                namelist_cont=> TokenType::NamelistCont
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
                _comma=> TokenType::Comma,
                _name=> TokenType::Name(_),
                namelist_cont=> TokenType::NamelistCont
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
        compile_context: &mut CompileContext,
        maybe_exp_descs: Option<&[ExpDesc<'a>]>,
    ) -> Result<(), Error> {
        match explist.tokens.as_slice() {
            make_deconstruct!(
                exp => TokenType::Exp,
                explist_cont => TokenType::ExplistCont
            ) => {
                let (exp_desc, tail) = maybe_exp_descs.map_or_else(
                    || (compile_context.reserve_stack_top().1, None),
                    |exp_descs| (exp_descs[0].clone(), Some(&exp_descs[1..])),
                );

                self.exp(exp, compile_context, &exp_desc)?.discharge(
                    &exp_desc,
                    self,
                    compile_context,
                )?;

                self.explist_cont(explist_cont, compile_context, tail)
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
        compile_context: &mut CompileContext,
        maybe_exp_descs: Option<&[ExpDesc<'a>]>,
    ) -> Result<(), Error> {
        match explist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma => TokenType::Comma,
                exp => TokenType::Exp,
                explist_cont => TokenType::ExplistCont
            ) => {
                let (exp_desc, tail) = maybe_exp_descs.map_or_else(
                    || (compile_context.reserve_stack_top().1, None),
                    |exp_descs| (exp_descs[0].clone(), Some(&exp_descs[1..])),
                );

                self.exp(exp, compile_context, &exp_desc)?.discharge(
                    &exp_desc,
                    self,
                    compile_context,
                )?;

                self.explist_cont(explist_cont, compile_context, tail)
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
        compile_context: &mut CompileContext,
        exp_desc: &ExpDesc<'a>,
    ) -> Result<ExpDesc<'a>, Error> {
        match exp.tokens.as_slice() {
            make_deconstruct!(_nil => TokenType::Nil) => Ok(self.nil()),
            make_deconstruct!(_false => TokenType::False) => Ok(self.boolean(false)),
            make_deconstruct!(_true => TokenType::True) => Ok(self.boolean(true)),
            make_deconstruct!(
                _string => TokenType::String(string)
            ) => Ok(self.string(string)),
            make_deconstruct!(_integer => TokenType::Integer(integer)) => {
                Ok(self.integer(*integer))
            }
            make_deconstruct!(_float => TokenType::Float(float)) => Ok(self.float(*float)),
            make_deconstruct!(_functiondef => TokenType::Functiondef) => Err(Error::Unimplemented),
            make_deconstruct!(var => TokenType::Var) => self.var(var, compile_context),
            make_deconstruct!(functioncall => TokenType::Functioncall) => {
                self.functioncall(functioncall, compile_context)?;
                // TODO verify what needs to be returned here
                Ok(ExpDesc::Nil)
            }
            make_deconstruct!(
                _lparen => TokenType::LParen,
                exp => TokenType::Exp,
                _rparen => TokenType::RParen
            ) => self.exp(exp, compile_context, exp_desc),
            make_deconstruct!(
                tableconstructor => TokenType::Tableconstructor
            ) => self.tableconstructor(tableconstructor, compile_context, exp_desc),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Or,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::And,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Less,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Greater,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Leq,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Geq,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Eq,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Neq,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::BitOr,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::BitXor,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::BitAnd,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::ShiftL,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::ShiftR,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Concat,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                lhs => TokenType::Exp,
                _op => TokenType::Add,
                rhs => TokenType::Exp
            ) => {
                let (lhs_dst, lhs_top) = match exp_desc {
                    ExpDesc::Local(dst) => (u8::try_from(*dst)?, exp_desc.clone()),
                    ExpDesc::Global(_) => compile_context.reserve_stack_top(),
                    _ => todo!("add: see what other cases are needed"),
                };
                let lhs = self.exp(lhs, compile_context, &lhs_top)?;

                let (rhs_dst, rhs_top) = compile_context.reserve_stack_top();
                let rhs = self.exp(rhs, compile_context, &rhs_top)?;

                compile_context.stack_top -= 2;

                match (lhs, rhs) {
                    (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Integer(lhs_i + rhs_i))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_f + rhs_f))
                    }
                    (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_i as f64 + rhs_f))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Float(lhs_f + rhs_i as f64))
                    }
                    (ExpDesc::Nil, _) => Err(Error::NilArithmetic),
                    (ExpDesc::Boolean(_), _) => Err(Error::BoolArithmetic),
                    (ExpDesc::String(_), _) => Err(Error::StringArithmetic),
                    (ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _), _) => {
                        Err(Error::TableArithmetic)
                    }
                    (_, ExpDesc::Nil) => Err(Error::NilArithmetic),
                    (_, ExpDesc::Boolean(_)) => Err(Error::BoolArithmetic),
                    (_, ExpDesc::String(_)) => Err(Error::StringArithmetic),
                    (_, ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _)) => {
                        Err(Error::TableArithmetic)
                    }
                    (lhs, rhs) => {
                        lhs.discharge(&lhs_top, self, compile_context)?;
                        rhs.discharge(&rhs_top, self, compile_context)?;

                        Ok(ExpDesc::Binop(
                            ByteCode::Add,
                            usize::from(lhs_dst),
                            usize::from(rhs_dst),
                        ))
                    }
                }
            }
            make_deconstruct!(
                lhs => TokenType::Exp,
                _op => TokenType::Sub,
                rhs => TokenType::Exp
            ) => {
                let (lhs_dst, lhs_top) = match exp_desc {
                    ExpDesc::Local(dst) => (u8::try_from(*dst)?, exp_desc.clone()),
                    ExpDesc::Global(_) => compile_context.reserve_stack_top(),
                    _ => todo!("sub: see what other cases are needed"),
                };
                let lhs = self.exp(lhs, compile_context, &lhs_top)?;

                let (rhs_dst, rhs_top) = compile_context.reserve_stack_top();
                let rhs = self.exp(rhs, compile_context, &rhs_top)?;

                compile_context.stack_top -= 2;

                match (lhs, rhs) {
                    (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Integer(lhs_i - rhs_i))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_f - rhs_f))
                    }
                    (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_i as f64 - rhs_f))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Float(lhs_f - rhs_i as f64))
                    }
                    (ExpDesc::Nil, _) => Err(Error::NilArithmetic),
                    (ExpDesc::Boolean(_), _) => Err(Error::BoolArithmetic),
                    (ExpDesc::String(_), _) => Err(Error::StringArithmetic),
                    (ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _), _) => {
                        Err(Error::TableArithmetic)
                    }
                    (_, ExpDesc::Nil) => Err(Error::NilArithmetic),
                    (_, ExpDesc::Boolean(_)) => Err(Error::BoolArithmetic),
                    (_, ExpDesc::String(_)) => Err(Error::StringArithmetic),
                    (_, ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _)) => {
                        Err(Error::TableArithmetic)
                    }
                    (lhs, rhs) => {
                        lhs.discharge(&lhs_top, self, compile_context)?;
                        rhs.discharge(&rhs_top, self, compile_context)?;

                        Ok(ExpDesc::Binop(
                            ByteCode::Sub,
                            usize::from(lhs_dst),
                            usize::from(rhs_dst),
                        ))
                    }
                }
            }
            make_deconstruct!(
                lhs => TokenType::Exp,
                _op => TokenType::Mul,
                rhs => TokenType::Exp
            ) => {
                let (lhs_dst, lhs_top) = match exp_desc {
                    ExpDesc::Local(dst) => (u8::try_from(*dst)?, exp_desc.clone()),
                    ExpDesc::Global(_) => compile_context.reserve_stack_top(),
                    _ => todo!("mul: see what other cases are needed"),
                };
                let lhs = self.exp(lhs, compile_context, &lhs_top)?;

                let (rhs_dst, rhs_top) = compile_context.reserve_stack_top();
                let rhs = self.exp(rhs, compile_context, &rhs_top)?;

                compile_context.stack_top -= 2;

                match (lhs, rhs) {
                    (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Integer(lhs_i * rhs_i))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_f * rhs_f))
                    }
                    (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_i as f64 * rhs_f))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Float(lhs_f * rhs_i as f64))
                    }
                    (ExpDesc::Nil, _) => Err(Error::NilArithmetic),
                    (ExpDesc::Boolean(_), _) => Err(Error::BoolArithmetic),
                    (ExpDesc::String(_), _) => Err(Error::StringArithmetic),
                    (ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _), _) => {
                        Err(Error::TableArithmetic)
                    }
                    (_, ExpDesc::Nil) => Err(Error::NilArithmetic),
                    (_, ExpDesc::Boolean(_)) => Err(Error::BoolArithmetic),
                    (_, ExpDesc::String(_)) => Err(Error::StringArithmetic),
                    (_, ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _)) => {
                        Err(Error::TableArithmetic)
                    }
                    (lhs, rhs) => {
                        lhs.discharge(&lhs_top, self, compile_context)?;
                        rhs.discharge(&rhs_top, self, compile_context)?;

                        Ok(ExpDesc::Binop(
                            ByteCode::Mul,
                            usize::from(lhs_dst),
                            usize::from(rhs_dst),
                        ))
                    }
                }
            }
            make_deconstruct!(
                lhs => TokenType::Exp,
                _op => TokenType::Div,
                rhs => TokenType::Exp
            ) => {
                let (lhs_dst, lhs_top) = match exp_desc {
                    ExpDesc::Local(dst) => (u8::try_from(*dst)?, exp_desc.clone()),
                    ExpDesc::Global(_) => compile_context.reserve_stack_top(),
                    _ => todo!("mul: see what other cases are needed"),
                };
                let lhs = self.exp(lhs, compile_context, &lhs_top)?;

                let (rhs_dst, rhs_top) = compile_context.reserve_stack_top();
                let rhs = self.exp(rhs, compile_context, &rhs_top)?;

                compile_context.stack_top -= 2;

                match (lhs, rhs) {
                    (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Float(lhs_i as f64 / rhs_i as f64))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_f / rhs_f))
                    }
                    (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
                        Ok(ExpDesc::Float(lhs_i as f64 / rhs_f))
                    }
                    (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
                        Ok(ExpDesc::Float(lhs_f / rhs_i as f64))
                    }
                    (ExpDesc::Nil, _) => Err(Error::NilArithmetic),
                    (ExpDesc::Boolean(_), _) => Err(Error::BoolArithmetic),
                    (ExpDesc::String(_), _) => Err(Error::StringArithmetic),
                    (ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _), _) => {
                        Err(Error::TableArithmetic)
                    }
                    (_, ExpDesc::Nil) => Err(Error::NilArithmetic),
                    (_, ExpDesc::Boolean(_)) => Err(Error::BoolArithmetic),
                    (_, ExpDesc::String(_)) => Err(Error::StringArithmetic),
                    (_, ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _)) => {
                        Err(Error::TableArithmetic)
                    }
                    (lhs, rhs) => {
                        lhs.discharge(&lhs_top, self, compile_context)?;
                        rhs.discharge(&rhs_top, self, compile_context)?;

                        Ok(ExpDesc::Binop(
                            ByteCode::Div,
                            usize::from(lhs_dst),
                            usize::from(rhs_dst),
                        ))
                    }
                }
            }
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Idiv,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Mod,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Pow,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _op => TokenType::Not,
                rhs => TokenType::Exp
            ) => {
                let (stack_top, top) = match exp_desc {
                    ExpDesc::Local(top) => (*top, exp_desc.clone()),
                    _ => {
                        let (stack_top, top) = compile_context.reserve_stack_top();
                        (usize::from(stack_top), top)
                    }
                };
                let exp_exp_desc = self.exp(rhs, compile_context, &top)?;

                match exp_exp_desc {
                    ExpDesc::Local(src) => Ok(ExpDesc::Unop(ByteCode::Not, src)),
                    global @ ExpDesc::Global(_) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::Not, stack_top))
                    }
                    ExpDesc::Nil => Ok(ExpDesc::Boolean(true)),
                    ExpDesc::Boolean(boolean) => Ok(ExpDesc::Boolean(!boolean)),
                    other => {
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                }
            }
            make_deconstruct!(
                _op => TokenType::Len,
                rhs => TokenType::Exp
            ) => {
                let (stack_top, top) = match exp_desc {
                    ExpDesc::Local(top) => (*top, exp_desc.clone()),
                    _ => {
                        let (stack_top, top) = compile_context.reserve_stack_top();
                        (usize::from(stack_top), top)
                    }
                };
                let exp_exp_desc = self.exp(rhs, compile_context, &top)?;

                match exp_exp_desc {
                    ExpDesc::Local(src) => Ok(ExpDesc::Unop(ByteCode::Len, src)),
                    global @ ExpDesc::Global(_) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::Len, stack_top))
                    }
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        Ok(ExpDesc::Integer(i64::try_from(string.len())?))
                    }
                    other => {
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                }
            }
            make_deconstruct!(
                _op => TokenType::Sub,
                rhs => TokenType::Exp
            ) => {
                let (stack_top, top) = match exp_desc {
                    ExpDesc::Local(top) => (*top, exp_desc.clone()),
                    _ => {
                        let (stack_top, top) = compile_context.reserve_stack_top();
                        (usize::from(stack_top), top)
                    }
                };
                let exp_exp_desc = self.exp(rhs, compile_context, &top)?;

                match exp_exp_desc {
                    ExpDesc::Local(src) => Ok(ExpDesc::Unop(ByteCode::Neg, src)),
                    global @ ExpDesc::Global(_) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::Neg, stack_top))
                    }
                    ExpDesc::Integer(integer) => Ok(ExpDesc::Integer(-integer)),
                    ExpDesc::Float(float) => Ok(ExpDesc::Float(-float)),
                    other => {
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                }
            }
            make_deconstruct!(
                _op => TokenType::BitXor,
                rhs => TokenType::Exp
            ) => {
                let (stack_top, top) = match exp_desc {
                    ExpDesc::Local(top) => (*top, exp_desc.clone()),
                    _ => {
                        let (stack_top, top) = compile_context.reserve_stack_top();
                        (usize::from(stack_top), top)
                    }
                };
                let exp_exp_desc = self.exp(rhs, compile_context, &top)?;

                match exp_exp_desc {
                    ExpDesc::Local(src) => Ok(ExpDesc::Unop(ByteCode::BitNot, src)),
                    global @ ExpDesc::Global(_) => {
                        global.discharge(&top, self, compile_context)?;
                        Ok(ExpDesc::Unop(ByteCode::BitNot, stack_top))
                    }
                    ExpDesc::Integer(integer) => Ok(ExpDesc::Integer(!integer)),
                    other => {
                        other.discharge(&top, self, compile_context)?;
                        Ok(top)
                    }
                }
            }
            _ => {
                unreachable!(
                    "Exp did not match any of the productions. Had {:#?}.",
                    exp.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn functioncall(
        &mut self,
        functioncall: &Token<'_>,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let func_index = compile_context.stack_top;
        match functioncall.tokens.as_slice() {
            make_deconstruct!(
                var => TokenType::Var,
                args => TokenType::Args
            ) => {
                let top = self.var(var, compile_context)?;
                top.discharge(
                    &compile_context.reserve_stack_top().1,
                    self,
                    compile_context,
                )?;

                self.args(args, compile_context)?;

                self.byte_codes.push(ByteCode::Call(func_index, 1));
                compile_context.stack_top = func_index;

                Ok(())
            }
            make_deconstruct!(
                _functioncall => TokenType::Functioncall,
                _args => TokenType::Args
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lparen => TokenType::LParen,
                _exp => TokenType::Exp,
                _rparen => TokenType::RParen,
                _args => TokenType::Args
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _var => TokenType::Var,
                _colon => TokenType::Colon,
                _name => TokenType::Name(_),
                _args => TokenType::Args
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _functioncall => TokenType::Functioncall,
                _colon => TokenType::Colon,
                _name => TokenType::Name(_),
                _args => TokenType::Args
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lparen => TokenType::LParen,
                _exp => TokenType::Exp,
                _rparen => TokenType::RParen,
                _colon => TokenType::Colon,
                _name => TokenType::Name(_),
                _args => TokenType::Args
            ) => Err(Error::Unimplemented),
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

    fn args(
        &mut self,
        args: &Token<'_>,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match args.tokens.as_slice() {
            make_deconstruct!(
                _lparen => TokenType::LParen,
                args_explist => TokenType::ArgsExplist,
                _rparen =>TokenType::RParen
            ) => self.args_explist(args_explist, compile_context),
            make_deconstruct!(
                tableconstructor => TokenType::Tableconstructor
            ) => {
                let (_, top) = compile_context.reserve_stack_top();
                self.tableconstructor(tableconstructor, compile_context, &top)?;
                // Already on top of the stack, no need to move
                Ok(())
            }
            make_deconstruct!(
                _string => TokenType::String(string)
            ) => self.string(string).discharge(
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

    fn args_explist(
        &mut self,
        args_explist: &Token<'_>,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match args_explist.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                explist => TokenType::Explist
            ) => self.explist(explist, compile_context, None),
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

    fn functiondef(
        &mut self,
        functiondef: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match functiondef.tokens.as_slice() {
            make_deconstruct!(
                _function => TokenType::Function,
                _funcbody => TokenType::Funcbody
            ) => Err(Error::Unimplemented),
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

    fn funcbody(
        &mut self,
        funcbody: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match funcbody.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::LParen,
            }, Token {
                tokens: _,
                token_type: TokenType::FuncbodyParlist,
            }, Token {
                tokens: _,
                token_type: TokenType::RParen,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::End,
            }] => Err(Error::Unimplemented),
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

    fn funcbody_parlist(
        &mut self,
        funcbody_parlist: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match funcbody_parlist.tokens.as_slice() {
            [] => Ok(()),
            [Token {
                tokens: _,
                token_type: TokenType::Parlist,
            }] => Err(Error::Unimplemented),
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

    fn parlist(&mut self, parlist: &Token, _compile_context: &CompileContext) -> Result<(), Error> {
        match parlist.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::Namelist,
            }, Token {
                tokens: _,
                token_type: TokenType::ParlistCont,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Dots,
            }] => Err(Error::Unimplemented),
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
        parlist_cont: &Token,
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match parlist_cont.tokens.as_slice() {
            [] => Ok(()),
            [Token {
                tokens: _,
                token_type: TokenType::Comma,
            }, Token {
                tokens: _,
                token_type: TokenType::Dots,
            }] => Err(Error::Unimplemented),
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
        compile_context: &mut CompileContext,
        exp_desc: &ExpDesc<'a>,
    ) -> Result<ExpDesc<'a>, Error> {
        match tableconstructor.tokens.as_slice() {
            make_deconstruct!(
                _lcurly => TokenType::LCurly,
                tableconstructor_fieldlist => TokenType::TableconstructorFieldlist,
                _rcurly => TokenType::RCurly
            ) => {
                let dst = match exp_desc {
                    ExpDesc::Local(dst) => u8::try_from(*dst)?,
                    ExpDesc::Global(_) => {
                        let dst = compile_context.stack_top;
                        compile_context.stack_top += 1;
                        dst
                    }
                    _ => {
                        log::error!("Only table creation on stack is supported.");
                        return Err(Error::Unimplemented);
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
                self.byte_codes.push(ByteCode::SetList(dst, array_items));

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

    fn tableconstructor_fieldlist(
        &mut self,
        tableconstructor_fieldlist: &Token<'_>,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match tableconstructor_fieldlist.tokens.as_slice() {
            [] => Ok((0, 0)),
            make_deconstruct!(
                fieldlist => TokenType::Fieldlist
            ) => self.fieldlist(fieldlist, compile_context, table),
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

    fn fieldlist(
        &mut self,
        fieldlist: &Token<'_>,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match fieldlist.tokens.as_slice() {
            make_deconstruct!(
                field => TokenType::Field,
                fieldlist_cont => TokenType::FieldlistCont
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

    fn fieldlist_cont(
        &mut self,
        fieldlist_cont: &Token<'_>,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match fieldlist_cont.tokens.as_slice() {
            [] => Ok((0, 0)),
            make_deconstruct!(
                fieldsep => TokenType::Fieldsep,
                field => TokenType::Field,
                fieldlist_cont => TokenType::FieldlistCont
            ) => self
                .fieldsep(fieldsep)
                .and_then(|()| self.field(field, compile_context, table))
                .and_then(|(array_item, table_item)| {
                    self.fieldlist_cont(fieldlist_cont, compile_context, table)
                        .map(|(array_len, table_len)| {
                            (array_len + array_item, table_len + table_item)
                        })
                }),
            make_deconstruct!(
                fieldsep => TokenType::Fieldsep
            ) => self.fieldsep(fieldsep).map(|()| (0, 0)),
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

    fn field(
        &mut self,
        field: &Token<'_>,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match field.tokens.as_slice() {
            make_deconstruct!(
                _lsquare => TokenType::LSquare,
                key => TokenType::Exp,
                _rsquare => TokenType::RSquare,
                _assing => TokenType::Assign,
                exp => TokenType::Exp
            ) => {
                let (dst, key_top) = compile_context.reserve_stack_top();
                self.exp(key, compile_context, &key_top)?.discharge(
                    &key_top,
                    self,
                    compile_context,
                )?;
                let (_, exp_top) = compile_context.reserve_stack_top();
                self.exp(exp, compile_context, &exp_top)?.discharge(
                    &exp_top,
                    self,
                    compile_context,
                )?;

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
                    _ => Err(Error::Unimplemented),
                }
            }
            make_deconstruct!(
                _name => TokenType::Name(name),
                _assign => TokenType::Assign,
                exp => TokenType::Exp
            ) => {
                let (dst, top) = compile_context.reserve_stack_top();
                self.exp(exp, compile_context, &top)?
                    .discharge(&top, self, compile_context)?;

                let constant = self.push_constant(*name)?;
                self.byte_codes
                    .push(ByteCode::SetField(table, constant, dst));

                compile_context.stack_top = dst;
                Ok((0, 1))
            }
            make_deconstruct!(
                exp => TokenType::Exp
            ) => {
                let (_, top) = compile_context.reserve_stack_top();
                self.exp(exp, compile_context, &top)?
                    .discharge(&top, self, compile_context)?;

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
    fn fieldsep(&mut self, fieldsep: &Token) -> Result<(), Error> {
        match fieldsep.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::Comma,
            }] => Ok(()),
            [Token {
                tokens: _,
                token_type: TokenType::SemiColon,
            }] => Ok(()),
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
