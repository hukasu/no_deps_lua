use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;

use crate::closure::Upvalue;

#[derive(Debug)]
pub struct StackFrame {
    /// Function index
    pub function_index: usize,
    /// Program counter of the current program
    pub program_counter: usize,
    /// The location on stack immediately after the function's location.
    pub stack_frame: usize,
    /// Number of variadic arguments in each function stack.
    pub variadic_arguments: usize,
    /// Number of values that should be moved at the end of a call
    pub out_params: usize,
    /// Upvalues that target locals from this stack frame
    pub open_upvalues: Vec<Rc<RefCell<Upvalue>>>,
}
