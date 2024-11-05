mod byte_code;
mod error;
mod state;

use alloc::vec::Vec;

use crate::lex::Lex;

use super::value::Value;

pub use {byte_code::ByteCode, error::Error};

#[derive(Debug)]
pub struct Program {
    pub(super) constants: Vec<Value>,
    pub(super) byte_codes: Vec<ByteCode>,
}

impl Program {
    pub fn parse(program: &[u8]) -> Result<Self, Error> {
        let mut state = state::State {
            constants: Vec::new(),
            byte_codes: Vec::new(),
            machine: state::StateMachine::Start,
        };
        let lex = Lex::new(program);

        for lex_res in lex {
            match lex_res {
                Ok(token) => state.process(&token),
                Err(lex_err) => Err(Error::from(lex_err)),
            }?
        }

        Ok(Self {
            constants: state.constants,
            byte_codes: state.byte_codes,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;

    use super::*;

    #[test]
    fn hello_world() {
        let program = Program::parse(b"print \"hello world\"\nprint \"hello again...\"").unwrap();
        assert_eq!(
            &program.constants,
            &[
                Value::String("print".to_owned()),
                Value::String("hello world".to_owned()),
                Value::String("print".to_owned()),
                Value::String("hello again...".to_owned())
            ]
        );
        assert_eq!(
            &program.byte_codes,
            &[
                ByteCode::GetGlobal(0, 0),
                ByteCode::LoadConstant(1, 1),
                ByteCode::Call(0, 1),
                ByteCode::GetGlobal(0, 2),
                ByteCode::LoadConstant(1, 3),
                ByteCode::Call(0, 1),
            ]
        );

        let err = Program::parse(b"print \"hello world\"\nprint \"hello again...")
            .expect_err("This program should fail");
        assert_eq!(err, Error::LexError);
    }
}
