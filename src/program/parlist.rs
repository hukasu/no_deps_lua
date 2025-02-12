use alloc::vec::Vec;

use crate::value::Value;

#[derive(Debug, Default)]
pub struct Parlist {
    pub names: Vec<Value>,
    pub variadic_args: bool,
}
