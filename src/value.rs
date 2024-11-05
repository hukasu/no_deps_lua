use core::fmt::Debug;

use crate::Lua;

#[derive(Clone)]
pub enum Value<'a> {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    Function(fn(&mut Lua) -> i32),
}

impl<'a> Debug for Value<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Float(n) => write!(f, "{n:?}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Function(_) => write!(f, "function"),
        }
    }
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        // TODO compare Integer vs Float
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(b1), Value::Boolean(b2)) => *b1 == *b2,
            (Value::Integer(i1), Value::Integer(i2)) => *i1 == *i2,
            (Value::Float(f1), Value::Float(f2)) => *f1 == *f2,
            (Value::String(s1), Value::String(s2)) => *s1 == *s2,
            (Value::Function(f1), Value::Function(f2)) => core::ptr::eq(f1, f2),
            (_, _) => false,
        }
    }
}
