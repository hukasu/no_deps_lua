mod proto;
#[cfg(test)]
mod tests;

use alloc::{boxed::Box, rc::Rc};

use crate::{bytecode::Bytecode, function::Function};

use super::value::Value;

use proto::Proto;

#[derive(Debug, Default, Clone)]
pub struct Program {
    pub(super) byte_codes: Rc<[Bytecode]>,
    pub(super) constants: Rc<[Value]>,
    pub(super) functions: Rc<[Rc<Function>]>,
    pub(super) upvalues: Rc<[Box<str>]>,
}

impl Program {
    pub fn parse(program: &str) -> Result<Self, proto::Error> {
        Proto::parse(program).map(Program::from)
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
            functions: proto.functions.into(),
            upvalues: proto.upvalues.into(),
        }
    }
}
