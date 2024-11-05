mod byte_code;
mod error;
mod state;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        let program = Program::parse(
            r#"
print "hello world"
print "hello again..."
"#,
        )
        .unwrap();
        assert_eq!(
            &program.constants,
            &[
                Value::String("print"),
                Value::String("hello world"),
                Value::String("hello again...")
            ]
        );
        assert_eq!(
            &program.byte_codes,
            &[
                ByteCode::GetGlobal(0, 0),
                ByteCode::LoadConstant(1, 1),
                ByteCode::Call(0, 1),
                ByteCode::GetGlobal(0, 0),
                ByteCode::LoadConstant(1, 2),
                ByteCode::Call(0, 1),
            ]
        );

        let err = Program::parse(
            r#"
print "hello world"
print "hello again...
"#,
        )
        .expect_err("This program should fail");
        assert_eq!(err, Error::LexFailure);
    }
}
