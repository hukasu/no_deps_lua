mod byte_code;
mod compile_context;
mod error;
mod exp_desc;
#[cfg(test)]
mod tests;

use alloc::{collections::vec_deque::VecDeque, vec::Vec};

use crate::parser::{Parser, Token, TokenType};

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
            exp_descs: VecDeque::new(),
            new_locals: Vec::new(),
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
    fn chunk<'a>(
        &mut self,
        chunk: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
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

    fn block<'a>(
        &mut self,
        block: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
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

    fn block_stat<'a>(
        &mut self,
        block: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
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

    fn stat<'a>(
        &mut self,
        stat: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        assert!(
            compile_context.exp_descs.is_empty(),
            "ExpDesc should be empty when starting a new `stat`."
        );
        match stat.tokens.as_slice() {
            make_deconstruct!(_semicolon=> TokenType::SemiColon) => Ok(()),
            make_deconstruct!(
                varlist=> TokenType::Varlist,
                _assing=> TokenType::Assign,
                explist=> TokenType::Explist
            ) => self
                .varlist(varlist, compile_context)
                .and_then(|()| self.explist(explist, compile_context, false)),
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
            ) => self
                .attnamelist(attnamelist, compile_context)
                .and_then(|()| self.stat_attexplist(stat_attexplist, compile_context))
                .inspect(|()| {
                    let new_locals = core::mem::take(&mut compile_context.new_locals);
                    compile_context.locals.extend(new_locals);
                }),
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

    fn stat_attexplist<'a>(
        &mut self,
        stat_attexplist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match stat_attexplist.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _assign=> TokenType::Assign,
                explist=> TokenType::Explist
            ) => self.explist(explist, compile_context, false),
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
    ) -> Result<(), Error> {
        match attnamelist.tokens.as_slice() {
            make_deconstruct!(
                _name=> TokenType::Name(name),
                _attrib=> TokenType::Attrib,
                attnamelist_cont=> TokenType::AttnamelistCont
            ) => {
                let stack_top = compile_context.reserve_stack_top();
                compile_context.exp_descs.push_back(stack_top);
                compile_context.new_locals.push((*name).into());
                self.attnamelist_cont(attnamelist_cont, compile_context)
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
            make_deconstruct!(
                _comma=> TokenType::Comma,
                _name=> TokenType::Name(name),
                _attrib=> TokenType::Attrib,
                attnamelist_cont=> TokenType::AttnamelistCont
            ) => {
                let stack_top = compile_context.reserve_stack_top();
                compile_context.exp_descs.push_back(stack_top);
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
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match varlist.tokens.as_slice() {
            make_deconstruct!(var=> TokenType::Var, varlist_cont=> TokenType::VarlistCont) => self
                .var(var, compile_context)
                .and_then(|()| self.varlist_cont(varlist_cont, compile_context)),
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
    ) -> Result<(), Error> {
        match varlist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma=> TokenType::Comma,
                var=> TokenType::Var,
                varlist_cont=> TokenType::VarlistCont
            ) => self
                .var(var, compile_context)
                .and_then(|()| self.varlist_cont(varlist_cont, compile_context)),
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
    ) -> Result<(), Error> {
        match var.tokens.as_slice() {
            make_deconstruct!(_name=> TokenType::Name(name)) => self
                .name(name, compile_context)
                .map(|exp_desc| compile_context.exp_descs.push_back(exp_desc)),
            make_deconstruct!(
                _var=> TokenType::Var,
                _lsquare=> TokenType::LSquare,
                _exp=> TokenType::Exp,
                _rsquare=> TokenType::RSquare
            ) => Err(Error::Unimplemented),
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
                _var=> TokenType::Var,
                _dot=> TokenType::Dot,
                _name=> TokenType::Name(_)
            ) => Err(Error::Unimplemented),
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
        _compile_context: &CompileContext,
    ) -> Result<(), Error> {
        match namelist.tokens.as_slice() {
            make_deconstruct!(
                _name=> TokenType::Name(_),
                _namelist_cont=> TokenType::NamelistCont
            ) => Err(Error::Unimplemented),
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
            make_deconstruct!(
                _comma=> TokenType::Comma,
                _name=> TokenType::Name(_),
                _namelist_cont=> TokenType::NamelistCont
            ) => Err(Error::Unimplemented),
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
        discharge_on_top: bool,
    ) -> Result<(), Error> {
        match explist.tokens.as_slice() {
            make_deconstruct!(
                exp => TokenType::Exp,
                explist_cont => TokenType::ExplistCont
            ) => self
                .exp(exp, compile_context, discharge_on_top)
                .and_then(|()| self.explist_cont(explist_cont, compile_context, discharge_on_top)),
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
        discharge_on_top: bool,
    ) -> Result<(), Error> {
        match explist_cont.tokens.as_slice() {
            [] => Ok(()),
            make_deconstruct!(
                _comma => TokenType::Comma,
                exp => TokenType::Exp,
                explist_cont => TokenType::ExplistCont
            ) => self
                .exp(exp, compile_context, discharge_on_top)
                .and_then(|()| self.explist_cont(explist_cont, compile_context, discharge_on_top)),
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

    fn exp<'a>(
        &mut self,
        exp: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        discharge_on_top: bool,
    ) -> Result<(), Error> {
        let exp_desc = if discharge_on_top {
            compile_context.reserve_stack_top()
        } else {
            compile_context
                .exp_descs
                .pop_front()
                .ok_or(Error::OrphanExp)?
        };
        match exp.tokens.as_slice() {
            make_deconstruct!(_nil => TokenType::Nil) => self.nil().discharge(exp_desc, self),
            make_deconstruct!(_false => TokenType::False) => {
                self.boolean(false).discharge(exp_desc, self)
            }
            make_deconstruct!(_true => TokenType::True) => {
                self.boolean(true).discharge(exp_desc, self)
            }
            make_deconstruct!(
                _string => TokenType::String(string)
            ) => self.string(string).discharge(exp_desc, self),
            make_deconstruct!(_integer => TokenType::Integer(integer)) => {
                self.integer(*integer).discharge(exp_desc, self)
            }
            make_deconstruct!(_float => TokenType::Float(float)) => {
                self.float(*float).discharge(exp_desc, self)
            }
            make_deconstruct!(_functiondef => TokenType::Functiondef) => Err(Error::Unimplemented),
            make_deconstruct!(var => TokenType::Var) => self
                .var(var, compile_context)
                .and_then(|()| {
                    compile_context
                        .exp_descs
                        .pop_front()
                        .ok_or(Error::OrphanExp)
                })
                .and_then(|var| var.discharge(exp_desc, self)),
            make_deconstruct!(functioncall => TokenType::Functioncall) => {
                self.functioncall(functioncall, compile_context)
            }
            make_deconstruct!(
                _lparen => TokenType::LParen,
                exp => TokenType::Exp,
                _rparen => TokenType::RParen
            ) => self.exp(exp, compile_context, discharge_on_top),
            make_deconstruct!(
                tableconstructor => TokenType::Tableconstructor
            ) => self.tableconstructor(tableconstructor, compile_context),
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
                _lhs => TokenType::Exp,
                _op => TokenType::Add,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Sub,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Mul,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _lhs => TokenType::Exp,
                _op => TokenType::Div,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
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
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _op => TokenType::Len,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _op => TokenType::Sub,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            make_deconstruct!(
                _op => TokenType::BitXor,
                _rhs => TokenType::Exp
            ) => Err(Error::Unimplemented),
            _ => {
                unreachable!(
                    "Exp did not match any of the productions. Had {:#?}.",
                    exp.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn functioncall<'a>(
        &mut self,
        functioncall: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let func_index = compile_context.stack_top;
        match functioncall.tokens.as_slice() {
            make_deconstruct!(
                var => TokenType::Var,
                args => TokenType::Args
            ) => self
                .var(var, compile_context)
                .and_then(|()| {
                    compile_context
                        .exp_descs
                        .pop_front()
                        .ok_or(Error::OrphanExp)
                        .and_then(|top| top.discharge(compile_context.reserve_stack_top(), self))
                })
                .and_then(|()| self.args(args, compile_context))
                .map(|()| {
                    self.byte_codes.push(ByteCode::Call(func_index, 1));
                    compile_context.stack_top = func_index;
                }),
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

    fn args<'a>(
        &mut self,
        args: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match args.tokens.as_slice() {
            make_deconstruct!(
                _lparen => TokenType::LParen,
                args_explist => TokenType::ArgsExplist,
                _rparen =>TokenType::RParen
            ) => self.args_explist(args_explist, compile_context),
            make_deconstruct!(
                tableconstructor => TokenType::Tableconstructor
            ) => self.tableconstructor(tableconstructor, compile_context),
            make_deconstruct!(
                _string => TokenType::String(string)
            ) => self
                .string(string)
                .discharge(compile_context.reserve_stack_top(), self),
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
            make_deconstruct!(
                explist => TokenType::Explist
            ) => self.explist(explist, compile_context, true),
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
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match tableconstructor.tokens.as_slice() {
            make_deconstruct!(
                _lcurly => TokenType::LCurly,
                tableconstructor_fieldlist => TokenType::TableconstructorFieldlist,
                _rcurly => TokenType::RCurly
            ) => {
                let dst = compile_context.stack_top;
                // Setting the stack top to one register after table destination
                compile_context.stack_top += 1;

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

                // Setting the stack top back to one register after table destination
                // clearing the list values
                compile_context.stack_top = dst + 1;

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

    fn tableconstructor_fieldlist<'a>(
        &mut self,
        tableconstructor_fieldlist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
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

    fn fieldlist<'a>(
        &mut self,
        fieldlist: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
        table: u8,
    ) -> Result<(u8, u8), Error> {
        match fieldlist.tokens.as_slice() {
            make_deconstruct!(
                field => TokenType::Field,
                fieldlist_cont => TokenType::FieldlistCont
            ) => self
                .field(field, compile_context, table)
                .and_then(|(array_item, table_item)| {
                    self.fieldlist_cont(fieldlist_cont, compile_context, table)
                        .map(|(array_len, table_len)| {
                            (array_len + array_item, table_len + table_item)
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

    fn fieldlist_cont<'a>(
        &mut self,
        fieldlist_cont: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
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

    fn field<'a>(
        &mut self,
        field: &Token<'a>,
        compile_context: &mut CompileContext<'a>,
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
                let dst = compile_context.stack_top;
                self.exp(key, compile_context, true)
                    .and_then(|()| self.exp(exp, compile_context, true))
                    .inspect(|()| {
                        self.byte_codes
                            .push(ByteCode::SetTable(table, dst, dst + 1));
                    })
                    .map(|()| (0, 1))
            }
            make_deconstruct!(
                _name => TokenType::Name(name),
                _assign => TokenType::Assign,
                exp => TokenType::Exp
            ) => {
                let dst = compile_context.stack_top;
                self.exp(exp, compile_context, true)
                    .and_then(|()| self.push_constant(*name))
                    .map(|constant_pos| {
                        compile_context.stack_top = dst;
                        self.byte_codes
                            .push(ByteCode::SetField(table, constant_pos, dst));
                    })
                    .map(|()| (0, 1))
            }
            make_deconstruct!(
                exp => TokenType::Exp
            ) => self.exp(exp, compile_context, true).map(|()| (1, 0)),
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
