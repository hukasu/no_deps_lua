mod byte_code;
mod compile_context;
mod error;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;

use crate::{
    ext::Unescape,
    parser::{Parser, Token, TokenType},
};

use super::value::Value;

use compile_context::CompileContext;
pub use {byte_code::ByteCode, error::Error};

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

    #[must_use]
    fn load_constant(dst: u8, src: u8) -> ByteCode {
        ByteCode::LoadConstant(dst, src)
    }

    #[must_use]
    fn get_global(dst: u8, src: u8) -> ByteCode {
        ByteCode::GetGlobal(dst, src)
    }

    // Non-terminals
    fn chunk(&mut self, chunk: &Token, compile_context: &mut CompileContext) -> Result<(), Error> {
        match chunk.tokens.as_slice() {
            [block @ Token {
                tokens: _,
                token_type: TokenType::Block,
            }] => self.block(block, compile_context),
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

    fn block(&mut self, block: &Token, compile_context: &mut CompileContext) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [block_stat @ Token {
                tokens: _,
                token_type: TokenType::BlockStat,
            }, block_retstat @ Token {
                tokens: _,
                token_type: TokenType::BlockRetstat,
            }] => self
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
        block: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [] => Ok(()),
            [stat @ Token {
                tokens: _,
                token_type: TokenType::Stat,
            }, blockstat @ Token {
                tokens: _,
                token_type: TokenType::BlockStat,
            }] => self
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
            [Token {
                tokens: _,
                token_type: TokenType::Retstat,
            }] => Err(Error::Unimplemented),
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

    fn stat(&mut self, stat: &Token, compile_context: &mut CompileContext) -> Result<(), Error> {
        match stat.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::SemiColon,
            }] => Ok(()),
            [varlist @ Token {
                tokens: _,
                token_type: TokenType::Varlist,
            }, Token {
                tokens: _,
                token_type: TokenType::Assign,
            }, explist @ Token {
                tokens: _,
                token_type: TokenType::Explist,
            }] => self
                .explist(explist, compile_context)
                .and_then(|()| self.varlist(varlist, compile_context)),
            [functioncall @ Token {
                tokens: _,
                token_type: TokenType::Functioncall,
            }] => self.functioncall(functioncall, compile_context),
            [Token {
                tokens: _,
                token_type: TokenType::Label,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Break,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Goto,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Do,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::End,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::While,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Do,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::End,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Repeat,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::Until,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::If,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Then,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::StatElseif,
            }, Token {
                tokens: _,
                token_type: TokenType::StatElse,
            }, Token {
                tokens: _,
                token_type: TokenType::End,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::For,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::Assign,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Comma,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::StatForexp,
            }, Token {
                tokens: _,
                token_type: TokenType::Do,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::End,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::For,
            }, Token {
                tokens: _,
                token_type: TokenType::Namelist,
            }, Token {
                tokens: _,
                token_type: TokenType::In,
            }, Token {
                tokens: _,
                token_type: TokenType::Explist,
            }, Token {
                tokens: _,
                token_type: TokenType::Do,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::End,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Function,
            }, Token {
                tokens: _,
                token_type: TokenType::Funcname,
            }, Token {
                tokens: _,
                token_type: TokenType::Funcbody,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Local,
            }, Token {
                tokens: _,
                token_type: TokenType::Function,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::Funcbody,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Local,
            }, attnamelist @ Token {
                tokens: _,
                token_type: TokenType::Attnamelist,
            }, stat_attexplist @ Token {
                tokens: _,
                token_type: TokenType::StatAttexplist,
            }] => self
                .stat_attexplist(stat_attexplist, compile_context)
                .and_then(|()| self.attnamelist(attnamelist, compile_context)),
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
            [Token {
                tokens: _,
                token_type: TokenType::Elseif,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Then,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }, Token {
                tokens: _,
                token_type: TokenType::StatElseif,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::Else,
            }, Token {
                tokens: _,
                token_type: TokenType::Block,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::Comma,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
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
        stat_attexplist: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match stat_attexplist.tokens.as_slice() {
            [] => Ok(()),
            [Token {
                tokens: _,
                token_type: TokenType::Assign,
            }, explist @ Token {
                tokens: _,
                token_type: TokenType::Explist,
            }] => self.explist(explist, compile_context),
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

    fn attnamelist(
        &mut self,
        attnamelist: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match attnamelist.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::Name(name),
            }, attrib @ Token {
                tokens: _,
                token_type: TokenType::Attrib,
            }, attnamelist_cont @ Token {
                tokens: _,
                token_type: TokenType::AttnamelistCont,
            }] => {
                compile_context.locals.push((*name).into());
                self.attnamelist_cont(attnamelist_cont, compile_context.increment_stack_top())
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
        &mut self,
        attnamelist_cont: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match attnamelist_cont.tokens.as_slice() {
            [] => Ok(()),
            [Token {
                tokens: _,
                token_type: TokenType::Comma,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(name),
            }, Token {
                tokens: _,
                token_type: TokenType::Attrib,
            }, attnamelist_cont @ Token {
                tokens: _,
                token_type: TokenType::AttnamelistCont,
            }] => {
                compile_context.locals.push((*name).into());
                self.attnamelist_cont(attnamelist_cont, compile_context)
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
            [Token {
                tokens: _,
                token_type: TokenType::Less,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::Greater,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::Return,
            }, Token {
                tokens: _,
                token_type: TokenType::RetstatExplist,
            }, Token {
                tokens: _,
                token_type: TokenType::RetstatEnd,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::Explist,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::SemiColon,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::DoubleColon,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::DoubleColon,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::FuncnameCont,
            }, Token {
                tokens: _,
                token_type: TokenType::FuncnameEnd,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::Dot,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::FuncnameCont,
            }] => Err(Error::Unimplemented),
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
            [Token {
                tokens: _,
                token_type: TokenType::Colon,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }] => Err(Error::Unimplemented),
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

    fn varlist(
        &mut self,
        varlist: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match varlist.tokens.as_slice() {
            [var @ Token {
                tokens: _,
                token_type: TokenType::Var,
            }, varlist_cont @ Token {
                tokens: _,
                token_type: TokenType::VarlistCont,
            }] => self
                .var_assign(var, compile_context)
                .and_then(|()| {
                    self.varlist_cont(varlist_cont, compile_context.increment_stack_top())
                })
                .inspect(|()| {
                    compile_context.decrement_stack_top();
                }),
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
        varlist_cont: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match varlist_cont.tokens.as_slice() {
            [] => Ok(()),
            [Token {
                tokens: _,
                token_type: TokenType::Comma,
            }, var @ Token {
                tokens: _,
                token_type: TokenType::VarlistCont,
            }, varlist_cont @ Token {
                tokens: _,
                token_type: TokenType::VarlistCont,
            }] => self.var_assign(var, compile_context).and_then(|()| {
                self.varlist_cont(varlist_cont, compile_context.increment_stack_top())
            }),
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

    fn var(&mut self, var: &Token, compile_context: &CompileContext) -> Result<(), Error> {
        match var.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::Name(name),
            }] => self.name(compile_context.stack_top, name, compile_context),
            [Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }, Token {
                tokens: _,
                token_type: TokenType::LSquare,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::RSquare,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }, Token {
                tokens: _,
                token_type: TokenType::Dot,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }] => Err(Error::Unimplemented),
            _ => {
                unreachable!(
                    "Var did not match any of the productions. Had {:#?}.",
                    var.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn var_assign(&mut self, var: &Token, compile_context: &CompileContext) -> Result<(), Error> {
        match var.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::Name(var_name),
            }] => {
                if let Some(var_dst) = compile_context
                    .locals
                    .iter()
                    .rposition(|name| name.eq(&Value::from(*var_name)))
                {
                    u8::try_from(var_dst).map_err(Error::from).map(|var_dst| {
                        self.byte_codes
                            .push(ByteCode::Move(var_dst, compile_context.stack_top));
                    })
                } else {
                    self.push_constant(*var_name).map(|global_pos| {
                        self.byte_codes
                            .push(ByteCode::SetGlobal(global_pos, compile_context.stack_top));
                    })
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
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match namelist.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::NamelistCont,
            }] => Err(Error::Unimplemented),
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
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match namelist_cont.tokens.as_slice() {
            [] => Ok(()),
            [Token {
                tokens: _,
                token_type: TokenType::Comma,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::NamelistCont,
            }] => Err(Error::Unimplemented),
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

    fn explist(
        &mut self,
        explist: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match explist.tokens.as_slice() {
            [exp @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, explist_cont @ Token {
                tokens: _,
                token_type: TokenType::ExplistCont,
            }] => self
                .exp(exp, compile_context)
                .and_then(|()| {
                    self.explist_cont(explist_cont, compile_context.increment_stack_top())
                })
                .inspect(|()| {
                    compile_context.decrement_stack_top();
                }),
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
        explist_cont: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match explist_cont.tokens.as_slice() {
            [] => Ok(()),
            [_comman @ Token {
                tokens: _,
                token_type: TokenType::Comma,
            }, exp @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, explist_cont @ Token {
                tokens: _,
                token_type: TokenType::ExplistCont,
            }] => self
                .exp(exp, compile_context)
                .and_then(|()| {
                    self.explist_cont(explist_cont, compile_context.increment_stack_top())
                })
                .inspect(|()| {
                    compile_context.decrement_stack_top();
                }),
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

    fn exp(&mut self, exp: &Token, compile_context: &CompileContext) -> Result<(), Error> {
        match exp.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::Nil,
            }] => {
                self.nil(compile_context.stack_top);
                Ok(())
            }
            [Token {
                tokens: _,
                token_type: TokenType::False,
            }] => {
                self.boolean(compile_context.stack_top, false);
                Ok(())
            }
            [Token {
                tokens: _,
                token_type: TokenType::True,
            }] => {
                self.boolean(compile_context.stack_top, true);
                Ok(())
            }
            [Token {
                tokens: _,
                token_type: TokenType::String(string),
            }] => self.string(compile_context.stack_top, string),
            [Token {
                tokens: _,
                token_type: TokenType::Integer(integer),
            }] => self.integer(compile_context.stack_top, *integer),
            [Token {
                tokens: _,
                token_type: TokenType::Float(float),
            }] => self.float(compile_context.stack_top, *float),
            [prefixexp @ Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }] => self.prefixexp(prefixexp, compile_context),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Or,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::And,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Less,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Greater,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Leq,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Geq,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Eq,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Neq,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::BitOr,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::BitXor,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::BitAnd,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::ShiftL,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::ShiftR,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Concat,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Add,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Sub,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Mul,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Div,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Idiv,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Mod,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::Pow,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Not,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Len,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::Sub,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::BitXor,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => Err(Error::Unimplemented),
            _ => {
                unreachable!(
                    "Exp did not match any of the productions. Had {:#?}.",
                    exp.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn prefixexp(
        &mut self,
        prefixexp: &Token,
        compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match prefixexp.tokens.as_slice() {
            [var @ Token {
                tokens: _,
                token_type: TokenType::Var,
            }] => self.var(var, compile_context),
            [Token {
                tokens: _,
                token_type: TokenType::Functioncall,
            }] => Err(Error::Unimplemented),
            [Token {
                tokens: _,
                token_type: TokenType::LParen,
            }, Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::RParen,
            }] => Err(Error::Unimplemented),
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

    fn functioncall(
        &mut self,
        functioncall: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match functioncall.tokens.as_slice() {
            [prefixexp @ Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }, args @ Token {
                tokens: _,
                token_type: TokenType::Args,
            }] => {
                let dst = compile_context.stack_top;
                self.prefixexp(prefixexp, compile_context)
                    .and_then(|()| self.args(args, compile_context.increment_stack_top()))
                    .inspect(|()| {
                        compile_context.stack_top = dst;
                        self.byte_codes.push(ByteCode::Call(dst, 1));
                    })
            }
            [Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }, Token {
                tokens: _,
                token_type: TokenType::Colon,
            }, Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }, Token {
                tokens: _,
                token_type: TokenType::Args,
            }] => Err(Error::Unimplemented),
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

    fn args(&mut self, args: &Token, compile_context: &mut CompileContext) -> Result<(), Error> {
        match args.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::LParen,
            }, args_explist @ Token {
                tokens: _,
                token_type: TokenType::ArgsExplist,
            }, Token {
                tokens: _,
                token_type: TokenType::RParen,
            }] => self.args_explist(args_explist, compile_context),
            [tableconstructor @ Token {
                tokens: _,
                token_type: TokenType::Tableconstructor,
            }] => self.tableconstructor(tableconstructor, compile_context),
            [Token {
                tokens: _,
                token_type: TokenType::String(string),
            }] => self.string(compile_context.stack_top, string),
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
        args_explist: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match args_explist.tokens.as_slice() {
            [] => Ok(()),
            [explist @ Token {
                tokens: _,
                token_type: TokenType::Explist,
            }] => self.explist(explist, compile_context),
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
            [Token {
                tokens: _,
                token_type: TokenType::Function,
            }, Token {
                tokens: _,
                token_type: TokenType::Funcbody,
            }] => Err(Error::Unimplemented),
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

    fn tableconstructor(
        &mut self,
        tableconstructor: &Token,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match tableconstructor.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::LCurly,
            }, tableconstructor_fieldlist @ Token {
                tokens: _,
                token_type: TokenType::TableconstructorFieldlist,
            }, Token {
                tokens: _,
                token_type: TokenType::RCurly,
            }] => {
                let dst = compile_context.stack_top;

                let table_initialization_bytecode_position = self.byte_codes.len();
                self.byte_codes.push(ByteCode::NewTable(0, 0, 0));

                let (array_items, table_items) = self.tableconstructor_fieldlist(
                    tableconstructor_fieldlist,
                    compile_context.increment_stack_top(),
                    dst,
                )?;

                self.byte_codes[table_initialization_bytecode_position] =
                    ByteCode::NewTable(dst, array_items, table_items);
                self.byte_codes.push(ByteCode::SetList(dst, array_items));

                Ok(())
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
        tableconstructor_fieldlist: &Token,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match tableconstructor_fieldlist.tokens.as_slice() {
            [] => Ok((0, 0)),
            [fieldlist @ Token {
                tokens: _,
                token_type: TokenType::Fieldlist,
            }] => self.fieldlist(fieldlist, compile_context, table),
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
        fieldlist: &Token,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match fieldlist.tokens.as_slice() {
            [field @ Token {
                tokens: _,
                token_type: TokenType::Field,
            }, fieldlist_cont @ Token {
                tokens: _,
                token_type: TokenType::FieldlistCont,
            }] => self
                .field(field, compile_context, table)
                .and_then(|(array_len, table_len)| {
                    compile_context.stack_top += array_len;
                    self.fieldlist_cont(fieldlist_cont, compile_context, table)
                        .map(|(array_items, table_items)| {
                            (array_items + array_len, table_items + table_len)
                        })
                }),
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
        fieldlist_cont: &Token,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match fieldlist_cont.tokens.as_slice() {
            [] => Ok((0, 0)),
            [fieldsep @ Token {
                tokens: _,
                token_type: TokenType::Fieldsep,
            }, field @ Token {
                tokens: _,
                token_type: TokenType::Field,
            }, fieldlist_cont @ Token {
                tokens: _,
                token_type: TokenType::FieldlistCont,
            }] => self
                .fieldsep(fieldsep)
                .and_then(|()| self.field(field, compile_context, table))
                .and_then(|(array_len, table_len)| {
                    self.fieldlist_cont(fieldlist_cont, compile_context, table)
                        .map(|(array_len_cont, table_len_cont)| {
                            (array_len_cont + array_len, table_len_cont + table_len)
                        })
                }),
            [fieldsep @ Token {
                tokens: _,
                token_type: TokenType::Fieldsep,
            }] => self.fieldsep(fieldsep).map(|()| (0, 0)),
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
        field: &Token,
        compile_context: &mut CompileContext,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match field.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::LSquare,
            }, key @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, Token {
                tokens: _,
                token_type: TokenType::RSquare,
            }, Token {
                tokens: _,
                token_type: TokenType::Assign,
            }, exp @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => {
                let dst = compile_context.stack_top;
                self.exp(key, compile_context)
                    .and_then(|()| self.exp(exp, compile_context))
                    .map(|()| {
                        compile_context.stack_top = dst;
                        self.byte_codes
                            .push(ByteCode::SetTable(table, dst, dst + 1));
                        (0, 1)
                    })
            }
            [Token {
                tokens: _,
                token_type: TokenType::Name(name),
            }, Token {
                tokens: _,
                token_type: TokenType::Assign,
            }, exp @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => {
                let dst = compile_context.stack_top;
                self.exp(exp, compile_context)
                    .and_then(|()| self.push_constant(*name))
                    .map(|constant_pos| {
                        compile_context.stack_top = dst;
                        self.byte_codes
                            .push(ByteCode::SetField(table, constant_pos, dst));
                        (0, 1)
                    })
            }
            [exp @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }] => self.exp(exp, compile_context).map(|()| (1, 0)),
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
    fn name(&mut self, dst: u8, name: &str, compile_context: &CompileContext) -> Result<(), Error> {
        if let Some(i) = compile_context
            .locals
            .iter()
            .rposition(|v| v == &Value::from(name))
        {
            u8::try_from(i).map_err(Error::from).map(|i| {
                self.byte_codes.push(ByteCode::Move(dst, i));
            })
        } else {
            let constant = self.push_constant(name)?;
            let bytecode = Self::get_global(dst, constant);
            self.byte_codes.push(bytecode);
            Ok(())
        }
    }

    fn string(&mut self, dst: u8, string: &str) -> Result<(), Error> {
        let constant = self.push_constant(string.unescape()?.as_str())?;
        let bytecode = Self::load_constant(dst, constant);
        self.byte_codes.push(bytecode);
        Ok(())
    }

    fn integer(&mut self, dst: u8, integer: i64) -> Result<(), Error> {
        if let Ok(ii) = i16::try_from(integer) {
            self.byte_codes.push(ByteCode::LoadInt(dst, ii));
        } else {
            let position = self.push_constant(integer)?;
            let byte_code = Self::load_constant(dst, position);
            self.byte_codes.push(byte_code);
        }
        Ok(())
    }

    fn float(&mut self, dst: u8, float: f64) -> Result<(), Error> {
        let position = self.push_constant(float)?;
        let byte_code = Self::load_constant(dst, position);
        self.byte_codes.push(byte_code);
        Ok(())
    }

    fn nil(&mut self, dst: u8) {
        self.byte_codes.push(ByteCode::LoadNil(dst));
    }

    fn boolean(&mut self, dst: u8, boolean: bool) {
        self.byte_codes.push(ByteCode::LoadBool(dst, boolean));
    }
}
