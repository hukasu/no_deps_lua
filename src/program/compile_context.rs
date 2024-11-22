use alloc::vec::Vec;

use crate::value::Value;

pub struct CompileContext {
    pub stack_top: u8,
    pub locals: Vec<Value>,
}

impl CompileContext {
    pub fn increment_stack_top(&mut self) -> &mut Self {
        self.stack_top += 1;
        self
    }

    pub fn decrement_stack_top(&mut self) -> &mut Self {
        self.stack_top -= 1;
        self
    }
}
