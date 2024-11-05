use alloc::vec::Vec;

use crate::{lex::Token, value::Value};

use super::{ByteCode, Error};

#[derive(Debug)]
pub struct State {
    pub(super) constants: Vec<Value>,
    pub(super) byte_codes: Vec<ByteCode>,
    pub(super) machine: StateMachine,
}

impl State {
    pub fn process(&mut self, token: &Token) -> Result<(), Error> {
        match self.machine {
            StateMachine::Start => match token {
                Token::Name(name) => {
                    let name_position = self.push_constant(Value::String(name.clone()));
                    self.byte_codes
                        .push(ByteCode::GetGlobal(0, name_position as u8));
                    self.machine = StateMachine::SeenName;
                    Ok(())
                }
                _ => Err(Error::Unimplemented),
            },
            StateMachine::SeenName => match token {
                Token::String(string) => {
                    let string_position = self.push_constant(Value::String(string.clone()));
                    self.byte_codes
                        .push(ByteCode::LoadConstant(1, string_position as u8));
                    self.byte_codes.push(ByteCode::Call(0, 1));
                    self.machine = StateMachine::Start;
                    Ok(())
                }
                _ => Err(Error::Unimplemented),
            },
        }
    }

    fn push_constant(&mut self, value: Value) -> usize {
        self.constants
            .iter()
            .position(|inserted| inserted == &value)
            .unwrap_or_else(|| {
                self.constants.push(value);
                self.constants.len() - 1
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateMachine {
    Start,
    SeenName,
}
