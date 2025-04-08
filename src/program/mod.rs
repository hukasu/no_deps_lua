mod compile_context;
mod error;
mod locals;
#[cfg(test)]
mod tests;

use alloc::{boxed::Box, rc::Rc};

use crate::{bytecode::Bytecode, function::Function};

use super::value::Value;

use compile_context::{CompileContext, Proto};
pub use error::Error;
pub use locals::Local;

#[derive(Debug, Default, Clone)]
pub struct Program {
    pub(super) byte_codes: Rc<[Bytecode]>,
    pub(super) constants: Rc<[Value]>,
    pub(super) locals: Rc<[Local]>,
    pub(super) upvalues: Rc<[Box<str>]>,
    pub(super) functions: Rc<[Rc<Function>]>,
}

impl Program {
    pub fn parse(program: &str) -> Result<Self, Error> {
        CompileContext::parse(program).map(Program::from)
    }

    pub fn read_bytecode(&self, index: usize) -> Option<Bytecode> {
        self.byte_codes.get(index).copied()
    }
}

impl From<Proto> for Program {
    fn from(proto: Proto) -> Self {
        Self {
            byte_codes: proto.byte_codes.into(),
            constants: proto.constants.into(),
            locals: proto.locals.into(),
            upvalues: proto.upvalues.into(),
            functions: proto.functions.into(),
        }
    }
}
