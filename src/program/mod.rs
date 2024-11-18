mod byte_code;
mod error;
#[cfg(test)]
mod tests;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::parser::{Parser, Token, TokenType};

use super::value::Value;

pub use {byte_code::ByteCode, error::Error};

#[derive(Debug)]
pub struct Program<'a> {
    pub(super) constants: Vec<Value<'a>>,
    pub(super) byte_codes: Vec<ByteCode>,
}

impl<'a> Program<'a> {
    pub fn parse(program: &'a str) -> Result<Self, Error> {
        let chunk = Parser::parse(program)?;

        let mut program = Program {
            constants: Vec::new(),
            byte_codes: Vec::new(),
        };
        let mut locals = Vec::new();

        program.chunk(&chunk, &mut locals)?;

        Ok(program)
    }

    fn push_constant(&mut self, value: Value<'a>) -> u8 {
        self.constants
            .iter()
            .position(|v| v == &value)
            .unwrap_or_else(|| {
                self.constants.push(value);
                self.constants.len() - 1
            }) as u8
    }

    #[must_use]
    fn load_constant(&mut self, dst: u8, src: u8) -> ByteCode {
        ByteCode::LoadConstant(dst, src)
    }

    #[must_use]
    fn get_global(&mut self, dst: u8, src: u8) -> ByteCode {
        ByteCode::GetGlobal(dst, src)
    }

    // Non-terminals
    fn chunk(&mut self, chunk: &Token<'a>, locals: &mut Vec<String>) -> Result<(), Error> {
        match chunk.tokens.as_slice() {
            [block @ Token {
                tokens: _,
                token_type: TokenType::Block,
            }] => self.block(block, locals, 0),
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

    fn block(&mut self, block: &Token<'a>, locals: &mut Vec<String>, dst: u8) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [blockstat @ Token {
                tokens: _,
                token_type: TokenType::BlockStat,
            }, _blockretstat @ Token {
                tokens: _,
                token_type: TokenType::BlockRetstat,
            }] => {
                self.block_stat(blockstat, locals, dst)?;
                // TODO retstat
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

        Ok(())
    }

    fn block_stat(
        &mut self,
        block: &Token<'a>,
        locals: &mut Vec<String>,
        dst: u8,
    ) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [] => {}
            [stat @ Token {
                tokens: _,
                token_type: TokenType::Stat,
            }, blockstat @ Token {
                tokens: _,
                token_type: TokenType::BlockStat,
            }] => {
                self.stat(stat, locals, dst)?;
                self.block_stat(blockstat, locals, locals.len() as u8)?;
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

        Ok(())
    }

    fn stat(&mut self, stat: &Token<'a>, locals: &mut Vec<String>, dst: u8) -> Result<(), Error> {
        match stat.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::SemiColon,
            }] => Ok(()),
            [functioncall @ Token {
                tokens: _,
                token_type: TokenType::Functioncall,
            }] => self.functioncall(functioncall, locals, dst),
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
                .stat_attexplist(stat_attexplist, locals, dst)
                .and_then(|()| self.attnamelist(attnamelist, locals, dst)),
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

    fn stat_attexplist(
        &mut self,
        stat_attexplist: &Token<'a>,
        locals: &mut Vec<String>,
        dst: u8,
    ) -> Result<(), Error> {
        match stat_attexplist.tokens.as_slice() {
            [] => Ok(()),
            [_assign @ Token {
                tokens: _,
                token_type: TokenType::Assign,
            }, explist @ Token {
                tokens: _,
                token_type: TokenType::Explist,
            }] => self.explist(explist, locals, dst),
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
        attnamelist: &Token<'a>,
        locals: &mut Vec<String>,
        dst: u8,
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
                locals.push(name.to_string());
                Ok(())
            }
            _ => {
                unreachable!(
                    "Stat did not match any of the productions. Had {:#?}.",
                    attnamelist
                        .tokens
                        .iter()
                        .map(|t| &t.token_type)
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    fn var(&mut self, var: &Token<'a>, locals: &mut Vec<String>, dst: u8) -> Result<(), Error> {
        match var.tokens.as_slice() {
            [name @ Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }] => self.name(name, locals, dst),
            _ => {
                unreachable!(
                    "Var did not match any of the productions. Had {:#?}.",
                    var.tokens.iter().map(|t| &t.token_type).collect::<Vec<_>>()
                );
            }
        }
    }

    fn explist(
        &mut self,
        explist: &Token<'a>,
        locals: &mut Vec<String>,
        dst: u8,
    ) -> Result<(), Error> {
        match explist.tokens.as_slice() {
            [exp @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, explist_cont @ Token {
                tokens: _,
                token_type: TokenType::ExplistCont,
            }] => self
                .exp(exp, locals, dst)
                .and_then(|()| self.explist_cont(explist_cont, locals, dst)),
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
        locals: &mut Vec<String>,
        dst: u8,
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
                .exp(exp, locals, dst)
                .and_then(|()| self.explist_cont(explist_cont, locals, dst + 1)),
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

    fn exp(&mut self, exp: &Token<'a>, locals: &mut Vec<String>, dst: u8) -> Result<(), Error> {
        match exp.tokens.as_slice() {
            [nil @ Token {
                tokens: _,
                token_type: TokenType::Nil,
            }] => self.nil(nil, dst),
            [false_token @ Token {
                tokens: _,
                token_type: TokenType::False,
            }] => self.false_token(false_token, dst),
            [true_token @ Token {
                tokens: _,
                token_type: TokenType::True,
            }] => self.true_token(true_token, dst),
            [string @ Token {
                tokens: _,
                token_type: TokenType::String(_),
            }] => self.string(string, dst),
            [integer @ Token {
                tokens: _,
                token_type: TokenType::Integer(_),
            }] => self.integer(integer, dst),
            [float @ Token {
                tokens: _,
                token_type: TokenType::Float(_),
            }] => self.float(float, dst),
            [prefixexp @ Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }] => self.prefixexp(prefixexp, locals, dst),
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
        prefixexp: &Token<'a>,
        locals: &mut Vec<String>,
        dst: u8,
    ) -> Result<(), Error> {
        match prefixexp.tokens.as_slice() {
            [var @ Token {
                tokens: _,
                token_type: TokenType::Var,
            }] => self.var(var, locals, dst),
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
        functioncall: &Token<'a>,
        locals: &mut Vec<String>,
        dst: u8,
    ) -> Result<(), Error> {
        match functioncall.tokens.as_slice() {
            [prefixexp @ Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }, args @ Token {
                tokens: _,
                token_type: TokenType::Args,
            }] => {
                self.prefixexp(prefixexp, locals, dst)?;
                self.args(args, locals, dst + 1)?;
                self.byte_codes.push(ByteCode::Call(dst, 1));
                Ok(())
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

    fn args(&mut self, args: &Token<'a>, locals: &mut Vec<String>, dst: u8) -> Result<(), Error> {
        match args.tokens.as_slice() {
            [_lparen @ Token {
                tokens: _,
                token_type: TokenType::LParen,
            }, args_explist @ Token {
                tokens: _,
                token_type: TokenType::ArgsExplist,
            }, _rparen @ Token {
                tokens: _,
                token_type: TokenType::RParen,
            }] => self.args_explist(args_explist, locals, dst),
            [string @ Token {
                tokens: _,
                token_type: TokenType::String(_),
            }] => self.string(string, dst),
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
        args_explist: &Token<'a>,
        locals: &mut Vec<String>,
        dst: u8,
    ) -> Result<(), Error> {
        match args_explist.tokens.as_slice() {
            [] => Ok(()),
            [explist @ Token {
                tokens: _,
                token_type: TokenType::Explist,
            }] => self.explist(explist, locals, dst),
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

    // Terminals
    fn name(&mut self, name: &Token<'a>, locals: &[String], dst: u8) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::Name(name),
        } = name
        else {
            unreachable!("Name should be Name token type.");
        };
        if let Some(i) = locals.iter().rposition(|v| v == name) {
            self.byte_codes.push(ByteCode::Move(dst, i as u8));
            Ok(())
        } else {
            let constant = self.push_constant(Value::String(name));
            let bytecode = self.get_global(dst, constant);
            self.byte_codes.push(bytecode);
            Ok(())
        }
    }

    fn string(&mut self, string: &Token<'a>, dst: u8) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::String(string),
        } = string
        else {
            unreachable!("String should be String token type.");
        };
        let constant = self.push_constant(Value::String(string));
        let bytecode = self.load_constant(dst, constant);
        self.byte_codes.push(bytecode);
        Ok(())
    }

    fn integer(&mut self, integer: &Token<'a>, dst: u8) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::Integer(int),
        } = integer
        else {
            unreachable!("Integer should be Integer token type.");
        };
        if let Ok(ii) = i16::try_from(*int) {
            self.byte_codes.push(ByteCode::LoadInt(dst, ii));
        } else {
            let position = self.push_constant(Value::Integer(*int));
            let byte_code = self.load_constant(dst, position);
            self.byte_codes.push(byte_code);
        }
        Ok(())
    }

    fn float(&mut self, float: &Token<'a>, dst: u8) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::Float(float),
        } = float
        else {
            unreachable!("Float should be Float token type.");
        };
        let position = self.push_constant(Value::Float(*float));
        let byte_code = self.load_constant(dst, position);
        self.byte_codes.push(byte_code);
        Ok(())
    }

    fn false_token(&mut self, false_token: &Token<'a>, dst: u8) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::False,
        } = false_token
        else {
            unreachable!("Float should be Float token type.");
        };
        self.byte_codes.push(ByteCode::LoadBool(dst, false));
        Ok(())
    }

    fn nil(&mut self, nil: &Token<'a>, dst: u8) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::Nil,
        } = nil
        else {
            unreachable!("Float should be Float token type.");
        };
        self.byte_codes.push(ByteCode::LoadNil(dst));
        Ok(())
    }

    fn true_token(&mut self, true_token: &Token<'a>, dst: u8) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::True,
        } = true_token
        else {
            unreachable!("Float should be Float token type.");
        };
        self.byte_codes.push(ByteCode::LoadBool(dst, true));
        Ok(())
    }
}
