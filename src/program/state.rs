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
        #[cfg(test)]
        log::trace!(target: "lua_program", "Processing token {:?}", token);
        match (&self.machine, &token.token) {
            (StateMachine::Start, TokenType::Name(name)) => {
                let name_position = self.push_constant(Value::String(name));
                self.byte_codes
                    .push(ByteCode::GetGlobal(0, name_position as u8));
                self.machine = StateMachine::SeenName;
                Ok(())
            }

            (StateMachine::SeenName, TokenType::LParen) => {
                self.machine = StateMachine::FunctionArgs;
                Ok(())
            }
            (StateMachine::SeenName, TokenType::String(string)) => {
                let string_position = self.push_constant(Value::String(string));
                self.push_byte_code(ByteCode::LoadConstant(1, string_position as u8));
                self.push_byte_code(ByteCode::Call(0, 1));
                self.machine = StateMachine::Start;
                Ok(())
            }
            (StateMachine::SeenName, _) => Err(Error::InvalidTokenAfterName(token.clone())),

            (StateMachine::FunctionArgs, TokenType::RParen) => {
                self.push_byte_code(ByteCode::Call(0, 0));
                Ok(())
            }
            (StateMachine::FunctionArgs, TokenType::Nil) => {
                self.push_byte_code(ByteCode::LoadNil(1));
                self.machine = StateMachine::FunctionArgs2;
                Ok(())
            }
            (StateMachine::FunctionArgs, TokenType::True) => {
                self.push_byte_code(ByteCode::LoadBool(1, true));
                self.machine = StateMachine::FunctionArgs2;
                Ok(())
            }
            (StateMachine::FunctionArgs, TokenType::False) => {
                self.push_byte_code(ByteCode::LoadBool(1, false));
                self.machine = StateMachine::FunctionArgs2;
                Ok(())
            }
            (StateMachine::FunctionArgs, TokenType::Integer(int)) => {
                let code = match i16::try_from(*int) {
                    Ok(int) => ByteCode::LoadInt(1, int),
                    Err(_) => {
                        let integer_position = self.push_constant(Value::Integer(*int)) as u8;
                        ByteCode::LoadConstant(1, integer_position)
                    }
                };
                self.push_byte_code(code);
                self.machine = StateMachine::FunctionArgs2;
                Ok(())
            }
            (StateMachine::FunctionArgs, TokenType::Float(float)) => {
                let float_position = self.push_constant(Value::Float(*float)) as u8;
                self.push_byte_code(ByteCode::LoadConstant(1, float_position));
                self.machine = StateMachine::FunctionArgs2;
                Ok(())
            }

            (StateMachine::FunctionArgs2, TokenType::RParen) => {
                self.push_byte_code(ByteCode::Call(0, 1));
                self.machine = StateMachine::Start;
                Ok(())
            }

            _ => Err(Error::Unimplemented),
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
    FunctionArgs,
    FunctionArgs2,
}
