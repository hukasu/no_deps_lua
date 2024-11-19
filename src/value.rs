use core::{fmt::Debug, ops::Deref};

use alloc::rc::Rc;

use crate::{stack_str::StackStr, Lua};

const SHORT_STRING_LEN: usize = 23;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    ShortString(StackStr<SHORT_STRING_LEN>),
    String(Rc<str>),
    Function(fn(&mut Lua) -> i32),
}

impl Value {
    pub fn new_string<T: Deref<Target = str>>(string: T) -> Value {
        match StackStr::new(string.deref()) {
            Ok(stack_str) => Value::ShortString(stack_str),
            Err(_) => Value::String(string.deref().into()),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Float(n) => write!(f, "{n:?}"),
            Value::ShortString(s) => write!(f, "{s}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Function(_) => write!(f, "function"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // TODO compare Integer vs Float
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(b1), Value::Boolean(b2)) => *b1 == *b2,
            (Value::Integer(i1), Value::Integer(i2)) => *i1 == *i2,
            (Value::Float(f1), Value::Float(f2)) => *f1 == *f2,
            (Value::ShortString(s1), Value::ShortString(s2)) => *s1 == *s2,
            (Value::String(s1), Value::String(s2)) => *s1 == *s2,
            (Value::Function(f1), Value::Function(f2)) => core::ptr::eq(f1, f2),
            (_, _) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_short_string_static_assert() {
        assert_eq!(size_of::<Value>(), 24);
    }
}
