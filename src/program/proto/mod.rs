mod binops;
mod compile_context;
mod error;
mod exp_desc;
mod helper_types;
mod unops;

use alloc::{boxed::Box, rc::Rc, vec::Vec};

use crate::{bytecode::Bytecode, function::Function, parser::Parser, value::Value};

pub use self::error::Error;
use self::{compile_context::CompileContext, exp_desc::ExpDesc};

use super::Local;

type ExpList<'a> = Vec<ExpDesc<'a>>;
type NameList<'a> = Vec<Box<str>>;

#[derive(Debug, Default)]
pub struct Proto {
    pub byte_codes: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub locals: Vec<Local>,
    pub upvalues: Vec<Box<str>>,
    pub functions: Vec<Rc<Function>>,
}

impl Proto {
    pub fn parse(program: &str) -> Result<Self, Error> {
        let chunk = Parser::parse(program)?;

        let mut compile_context = CompileContext::default().with_var_args(true);
        compile_context.chunk(&chunk)?;

        Ok(compile_context.proto)
    }

    fn push_constant(&mut self, value: impl Into<Value>) -> Result<u32, Error> {
        let value = value.into();

        let new_position = self
            .constants
            .iter()
            .position(|v| v == &value)
            .unwrap_or_else(|| {
                self.constants.push(value);
                self.constants.len() - 1
            });
        u32::try_from(new_position).map_err(Error::from)
    }

    fn push_function(&mut self, function: Function) -> usize {
        let new_position = self.functions.len();
        self.functions.push(Rc::new(function));
        new_position
    }

    fn push_upvalue(&mut self, upvalue: &str) -> usize {
        let value = upvalue.into();
        self.upvalues
            .iter()
            .position(|v| v == &value)
            .unwrap_or_else(|| {
                self.upvalues.push(value);
                self.upvalues.len() - 1
            })
    }
}
