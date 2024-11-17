mod byte_code;
mod error;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;

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

        program.chunk(&chunk)?;

        Ok(program)
    }

    fn push_constant(&mut self, value: Value<'a>) -> usize {
        self.constants
            .iter()
            .position(|v| v == &value)
            .unwrap_or_else(|| {
                self.constants.push(value);
                self.constants.len() - 1
            })
    }

    #[must_use]
    fn load_constant(&mut self, dst: usize, src: usize) -> ByteCode {
        ByteCode::LoadConstant(dst as u8, src as u8)
    }

    #[must_use]
    fn get_global(&mut self, dst: usize, src: usize) -> ByteCode {
        ByteCode::GetGlobal(dst as u8, src as u8)
    }

    // Non-terminals
    fn chunk(&mut self, chunk: &Token<'a>) -> Result<(), Error> {
        match chunk.tokens.as_slice() {
            [block @ Token {
                tokens: _,
                token_type: TokenType::Block,
            }] => self.block(block),
            _ => {
                unreachable!(
                    "Chunk did not match any of the productions. Had {:#?}.",
                    chunk
                );
            }
        }
    }

    fn block(&mut self, block: &Token<'a>) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [blockstat @ Token {
                tokens: _,
                token_type: TokenType::BlockStat,
            }, _blockretstat @ Token {
                tokens: _,
                token_type: TokenType::BlockRetstat,
            }] => {
                self.block_stat(blockstat)?;
                // TODO retstat
            }
            _ => {
                unreachable!("Block did not match any production. Had {:#?}.", block);
            }
        }

        Ok(())
    }

    fn block_stat(&mut self, block: &Token<'a>) -> Result<(), Error> {
        match block.tokens.as_slice() {
            [] => {}
            [stat @ Token {
                tokens: _,
                token_type: TokenType::Stat,
            }, blockstat @ Token {
                tokens: _,
                token_type: TokenType::BlockStat,
            }] => {
                self.stat(stat)?;
                self.block_stat(blockstat)?;
            }
            _ => {
                unreachable!("BlockStat did not match any production. Had {:#?}.", block);
            }
        }

        Ok(())
    }

    fn stat(&mut self, stat: &Token<'a>) -> Result<(), Error> {
        match stat.tokens.as_slice() {
            [Token {
                tokens: _,
                token_type: TokenType::SemiColon,
            }] => {}
            [functioncall @ Token {
                tokens: _,
                token_type: TokenType::Functioncall,
            }] => self.functioncall(functioncall)?,
            _ => {
                unreachable!(
                    "Stat did not match any of the productions. Had {:#?}.",
                    stat
                );
            }
        }

        Ok(())
    }

    fn var(&mut self, var: &Token<'a>) -> Result<(), Error> {
        match var.tokens.as_slice() {
            [name @ Token {
                tokens: _,
                token_type: TokenType::Name(_),
            }] => self.name(name),
            _ => {
                unreachable!("Var did not match any of the productions. Had {:#?}.", var);
            }
        }
    }

    fn explist(&mut self, explist: &Token<'a>) -> Result<(), Error> {
        match explist.tokens.as_slice() {
            [exp @ Token {
                tokens: _,
                token_type: TokenType::Exp,
            }, explist_cont @ Token {
                tokens: _,
                token_type: TokenType::ExplistCont,
            }] => self.exp(exp).and_then(|()| self.explist_cont(explist_cont)),
            _ => {
                unreachable!(
                    "Explist did not match any of the productions. Had {:#?}.",
                    explist
                );
            }
        }
    }

    fn explist_cont(&mut self, explist_cont: &Token<'a>) -> Result<(), Error> {
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
            }] => self.exp(exp).and_then(|()| self.explist_cont(explist_cont)),
            _ => {
                unreachable!(
                    "ExplistCont did not match any of the productions. Had {:#?}.",
                    explist_cont
                );
            }
        }
    }

    fn exp(&mut self, exp: &Token<'a>) -> Result<(), Error> {
        match exp.tokens.as_slice() {
            [_nil @ Token {
                tokens: _,
                token_type: TokenType::Nil,
            }] => {
                self.byte_codes.push(ByteCode::LoadNil(1));
                Ok(())
            }
            [_false @ Token {
                tokens: _,
                token_type: TokenType::False,
            }] => {
                self.byte_codes.push(ByteCode::LoadBool(1, false));
                Ok(())
            }
            [_true @ Token {
                tokens: _,
                token_type: TokenType::True,
            }] => {
                self.byte_codes.push(ByteCode::LoadBool(1, true));
                Ok(())
            }
            [Token {
                tokens: _,
                token_type: TokenType::Integer(int),
            }] => {
                if let Ok(ii) = i16::try_from(*int) {
                    self.byte_codes.push(ByteCode::LoadInt(1, ii));
                } else {
                    let position = self.push_constant(Value::Integer(*int));
                    let byte_code = self.load_constant(1, position);
                    self.byte_codes.push(byte_code);
                }
                Ok(())
            }
            [Token {
                tokens: _,
                token_type: TokenType::Float(float),
            }] => {
                let position = self.push_constant(Value::Float(*float));
                let byte_code = self.load_constant(1, position);
                self.byte_codes.push(byte_code);

                Ok(())
            }
            [Token {
                tokens: _,
                token_type: TokenType::String(string),
            }] => {
                let position = self.push_constant(Value::String(string));
                let byte_code = self.load_constant(1, position);
                self.byte_codes.push(byte_code);

                Ok(())
            }
            _ => {
                unreachable!("Exp did not match any of the productions. Had {:#?}.", exp);
            }
        }
    }

    fn prefixexp(&mut self, prefixexp: &Token<'a>) -> Result<(), Error> {
        match prefixexp.tokens.as_slice() {
            [var @ Token {
                tokens: _,
                token_type: TokenType::Var,
            }] => self.var(var),
            _ => {
                unreachable!(
                    "Prefixexp did not match any of the productions. Had {:#?}.",
                    prefixexp
                );
            }
        }
    }

    fn functioncall(&mut self, functioncall: &Token<'a>) -> Result<(), Error> {
        match functioncall.tokens.as_slice() {
            [prefixexp @ Token {
                tokens: _,
                token_type: TokenType::Prefixexp,
            }, args @ Token {
                tokens: _,
                token_type: TokenType::Args,
            }] => {
                self.prefixexp(prefixexp)?;
                self.args(args)?;
                self.byte_codes.push(ByteCode::Call(0, 1));
                Ok(())
            }
            _ => {
                unreachable!(
                    "Functioncall did not match any of the productions. Had {:#?}.",
                    functioncall
                );
            }
        }
    }

    fn args(&mut self, args: &Token<'a>) -> Result<(), Error> {
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
            }] => self.args_explist(args_explist),
            [string @ Token {
                tokens: _,
                token_type: TokenType::String(_),
            }] => self.string(string),
            _ => {
                unreachable!(
                    "Args did not match any of the productions. Had {:#?}.",
                    args
                );
            }
        }
    }

    fn args_explist(&mut self, args_explist: &Token<'a>) -> Result<(), Error> {
        match args_explist.tokens.as_slice() {
            [] => Ok(()),
            [explist @ Token {
                tokens: _,
                token_type: TokenType::Explist,
            }] => self.explist(explist),
            _ => {
                unreachable!(
                    "ArgsExplist did not match any of the productions. Had {:#?}.",
                    args_explist
                );
            }
        }
    }

    // Terminals
    fn name(&mut self, name: &Token<'a>) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::Name(name),
        } = name
        else {
            unreachable!("Name should have Name token type.");
        };
        let constant = self.push_constant(Value::String(name));
        let bytecode = self.get_global(0, constant);
        self.byte_codes.push(bytecode);
        Ok(())
    }

    fn string(&mut self, string: &Token<'a>) -> Result<(), Error> {
        let Token {
            tokens: _,
            token_type: TokenType::String(string),
        } = string
        else {
            unreachable!("String should have String token type.");
        };
        let constant = self.push_constant(Value::String(string));
        let bytecode = self.load_constant(1, constant);
        self.byte_codes.push(bytecode);
        Ok(())
    }
}
