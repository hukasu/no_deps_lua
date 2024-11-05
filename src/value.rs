use core::fmt::Debug;

use alloc::string::String;

use crate::Lua;

#[derive(Clone, PartialEq)]
pub enum Value {
    Nil,
    String(String),
    Function(fn(&mut Lua) -> i32),
}

impl Debug for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{s}"),
            Value::Function(_) => write!(f, "function"),
        }
    }
}
