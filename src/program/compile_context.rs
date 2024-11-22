use alloc::vec::Vec;

use crate::value::Value;

pub struct CompileContext {
    pub locals: Vec<Value>,
}
