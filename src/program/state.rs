use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::{lex::Token, value::Value};

use super::ByteCode;

#[derive(Debug)]
pub struct State {
    pub(super) constants: Vec<Value>,
    pub(super) byte_codes: Vec<ByteCode>,
    pub(super) machine: StateMachine,
}

impl State {
    pub fn process(&mut self, token: &Token) -> Result<(), String> {
        match self.machine {
            StateMachine::Start => match token {
                Token::Name(name) => {
                    self.constants.push(Value::String(name.clone()));
                    self.byte_codes
                        .push(ByteCode::GetGlobal(0, (self.constants.len() - 1) as u8));
                    self.machine = StateMachine::SeenName;
                    Ok(())
                }
                _ => Err("Unimplemented".to_owned()),
            },
            StateMachine::SeenName => match token {
                Token::String(string) => {
                    self.constants.push(Value::String(string.clone()));
                    self.byte_codes
                        .push(ByteCode::LoadConstant(1, (self.constants.len() - 1) as u8));
                    self.byte_codes.push(ByteCode::Call(0, 1));
                    self.machine = StateMachine::Start;
                    Ok(())
                }
                _ => Err("Unimplemented".to_owned()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateMachine {
    Start,
    SeenName,
}
