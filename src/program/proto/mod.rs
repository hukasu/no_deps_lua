mod binops;
mod compile_context;
mod compile_stack;
mod exp_desc;
mod helper_types;
mod unops;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use compile_stack::{CompileFrame, CompileStack};

use crate::{bytecode::Bytecode, function::Function, parser::Parser, program::Error, value::Value};

use super::Local;

use compile_context::CompileContext;

#[derive(Debug, Default)]
pub struct Proto {
    pub byte_codes: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub locals: Vec<Local>,
    pub upvalues: Vec<Box<str>>,
    pub functions: Vec<Rc<Function>>,
}

impl Proto {
    pub fn parse(program: &str) -> Result<Proto, Error> {
        let chunk = Parser::parse(program)?;

        let compile_context = CompileContext::new_with_var_args(true);
        let proto = Self::default();
        let mut compile_stack = CompileStack {
            stack: vec![CompileFrame {
                proto,
                compile_context,
            }],
        };
        compile_stack.chunk(&chunk)?;

        assert_eq!(
            compile_stack.stack.len(),
            1,
            "CompileStack must only have 1 frame at the end of parsing, but was {}.",
            compile_stack.stack.len()
        );

        let Some(CompileFrame {
            proto,
            compile_context: _,
        }) = compile_stack.stack.pop()
        else {
            unreachable!();
        };

        Ok(proto)
    }

    pub(super) fn push_constant(&mut self, value: impl Into<Value>) -> Result<u32, Error> {
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

    pub(super) fn push_function(&mut self, function: Function) -> usize {
        let new_position = self.functions.len();
        self.functions.push(Rc::new(function));
        new_position
    }

    pub(super) fn push_upvalue(&mut self, upvalue: &str) -> usize {
        let value = upvalue.into();
        self.upvalues
            .iter()
            .position(|v| v == &value)
            .unwrap_or_else(|| {
                self.upvalues.push(value);
                self.upvalues.len() - 1
            })
    }

    pub fn find_upvalue(&self, name: &str) -> Option<usize> {
        self.upvalues
            .iter()
            .rposition(|upvalue| upvalue.as_ref() == name)
    }
}
