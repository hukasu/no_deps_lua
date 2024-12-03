use alloc::vec::Vec;

use crate::value::Value;

use super::exp_desc::ExpDesc;

#[derive(Debug)]
pub struct CompileContext {
    pub stack_top: u8,
    pub locals: Vec<Value>,
    pub breaks: Option<Vec<usize>>,
}

impl CompileContext {
    pub fn reserve_stack_top<'a>(&mut self) -> (u8, ExpDesc<'a>) {
        let top = self.stack_top;
        self.stack_top += 1;
        (top, ExpDesc::Local(usize::from(top)))
    }
}
