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
