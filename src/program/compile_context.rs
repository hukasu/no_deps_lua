use alloc::{collections::vec_deque::VecDeque, vec::Vec};

use crate::value::Value;

use super::exp_desc::ExpDesc;

#[derive(Debug)]
pub struct CompileContext<'a> {
    pub stack_top: u8,
    pub locals: Vec<Value>,
    pub exp_descs: VecDeque<ExpDesc<'a>>,
    pub new_locals: Vec<Value>,
}

impl<'a> CompileContext<'a> {
    pub fn reserve_stack_top(&mut self) -> ExpDesc<'a> {
        let top = self.stack_top;
        self.stack_top += 1;
        ExpDesc::Local(usize::from(top))
    }
}
