use alloc::vec::Vec;

use crate::{
    lex::{Token, TokenType},
    value::Value,
};

use super::{ByteCode, Error};

#[derive(Debug)]
pub struct State<'a> {
    pub(super) constants: Vec<Value<'a>>,
    pub(super) byte_codes: Vec<ByteCode>,
    pub(super) machine: StateMachine,
}

impl<'a> State<'a> {
    pub fn process(&mut self, token: &Token<'a>) -> Result<(), Error<'a>> {
        match self.machine {
            StateMachine::Start => match &token.token {
                TokenType::Name(name) => {
                    let name_position = self.push_constant(Value::String(name));
                    self.byte_codes
                        .push(ByteCode::GetGlobal(0, name_position as u8));
                    self.machine = StateMachine::SeenName;
                    Ok(())
                }
                _ => Err(Error::Unimplemented),
            },
            StateMachine::SeenName => match &token.token {
                TokenType::LParen => {
                    self.machine = StateMachine::Start;
                    Ok(())
                }
                TokenType::String(string) => {
                    let string_position = self.push_constant(Value::String(string));
                    self.push_byte_code(ByteCode::LoadConstant(1, string_position as u8));
                    self.push_byte_code(ByteCode::Call(0, 1));
                    self.machine = StateMachine::Start;
                    Ok(())
                }
                _ => Err(Error::InvalidTokenAfterName(token.clone())),
            },
        }
    }

    fn push_constant(&mut self, value: Value<'a>) -> usize {
        self.constants
            .iter()
            .position(|inserted| inserted == &value)
            .unwrap_or_else(|| {
                self.constants.push(value);
                self.constants.len() - 1
            })
    }

    fn push_byte_code(&mut self, byte_code: ByteCode) {
        self.byte_codes.push(byte_code);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateMachine {
    Start,
    SeenName,
}
