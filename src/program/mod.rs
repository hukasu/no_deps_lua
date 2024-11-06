mod byte_code;
mod error;
mod state;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;

use crate::lex::Lex;

use super::value::Value;

pub use {byte_code::ByteCode, error::Error};

#[derive(Debug)]
pub struct Program<'a> {
    pub(super) program: &'a str,
    pub(super) constants: Vec<Value<'a>>,
    pub(super) byte_codes: Vec<ByteCode>,
}

impl<'a> Program<'a> {
    pub fn parse(program: &'a str) -> Result<Self, Error> {
        let mut state = state::State {
            constants: Vec::new(),
            byte_codes: Vec::new(),
            machine: state::StateMachine::Start,
        };
        let lex = Lex::new(program);

        for lex_res in lex {
            match lex_res {
                Ok((token, None)) => state.process(&token),
                Ok((token, Some(second_token))) => {
                    state.process(&token)?;
                    state.process(&second_token)
                }
                Err(lex_err) => Err(Error::from(lex_err)),
            }?
        }

        Ok(Self {
            program,
            constants: state.constants,
            byte_codes: state.byte_codes,
        })
    }
}
