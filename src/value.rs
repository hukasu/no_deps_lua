use core::{
    cell::RefCell,
    cmp::Ordering,
    fmt::{Debug, Display},
};

use alloc::rc::Rc;

use crate::{stack_str::StackStr, table::Table, Lua, Program};

const SHORT_STRING_LEN: usize = 23;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    ShortString(StackStr<SHORT_STRING_LEN>),
    String(Rc<str>),
    Table(Rc<RefCell<Table>>),
    Function(fn(&mut Lua) -> i32),
    Closure(Rc<Program>),
}

impl Value {
    pub fn try_float(&self) -> Option<Value> {
        match self {
            Value::Integer(i) => Some(Value::Float(*i as f64)),
            Value::Float(f) => Some(Value::Float(*f)),
            _ => None,
        }
    }

    pub fn static_type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::ShortString(_) | Self::String(_) => "string",
            Self::Table(_) => "table",
            Self::Function(_) | Self::Closure(_) => "function",
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Integer(l), Value::Integer(r)) => Some(l.cmp(r)),
            (Value::Integer(l), Value::Float(r)) => (*l as f64).partial_cmp(r),
            (Value::Float(l), Value::Integer(r)) => l.partial_cmp(&(*r as f64)),
            (Value::Float(l), Value::Float(r)) => l.partial_cmp(r),

            (Value::ShortString(l), Value::ShortString(r)) => Some(l.cmp(r)),
            (Value::ShortString(l), Value::String(r)) => Some(l.as_ref().cmp(r.as_bytes())),
            (Value::String(l), Value::ShortString(r)) => Some(l.as_bytes().cmp(r.as_ref())),
            (Value::String(l), Value::String(r)) => Some(l.cmp(r)),

            _ => None,
        }
    }
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Value::Nil
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Integer(i64::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<&str> for Value {
    fn from(string: &str) -> Self {
        match StackStr::new(string) {
            Ok(stack_str) => Value::ShortString(stack_str),
            Err(_) => Value::String(string.into()),
        }
    }
}

impl From<Rc<Program>> for Value {
    fn from(closure: Rc<Program>) -> Self {
        Self::Closure(closure)
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::Boolean(b) => write!(f, "Boolean({b})"),
            Self::Integer(i) => write!(f, "Integer({i})"),
            Self::Float(n) => write!(f, "Float({n:?})"),
            Self::ShortString(s) => write!(f, "ShortString({s})"),
            Self::String(s) => write!(f, "String({s})"),
            Self::Table(table) => {
                let t = table.borrow();
                write!(f, "Table({}:{})", t.array.len(), t.table.len())
            }
            Self::Function(func) => write!(f, "Function({:?})", func),
            Self::Closure(func) => write!(f, "Closure({:?})", func),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(n) => write!(f, "{n:?}"),
            Self::ShortString(s) => write!(f, "{s}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Table(table) => write!(f, "table:{:?}", table.as_ptr()),
            Self::Function(_) | Self::Closure(_) => write!(f, "function"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // TODO compare Integer vs Float
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(b1), Self::Boolean(b2)) => b1 == b2,
            (Self::Integer(i1), Self::Integer(i2)) => i1 == i2,
            (Self::Float(f1), Self::Float(f2)) => f1 == f2,
            (Self::ShortString(s1), Self::ShortString(s2)) => s1 == s2,
            (Self::String(s1), Self::String(s2)) => s1 == s2,
            (Self::Table(t1), Self::Table(t2)) => t1 == t2,
            (Self::Function(f1), Self::Function(f2)) => core::ptr::eq(f1, f2),
            (_, _) => false,
        }
    }
}

impl Eq for Value {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_short_string_static_assert() {
        assert_eq!(size_of::<Value>(), 24);
    }
}

#[derive(Debug, PartialEq, Eq)]
/// A wrapper around value so that it can be ordered on a [`Vec`] and
/// be searched using `binary_search`
pub struct ValueKey(Value);

impl ValueKey {
    fn ord_priority(&self) -> usize {
        match self.0 {
            Value::Nil => 0,
            Value::Boolean(_) => 1,
            Value::Integer(_) => 2,
            Value::Float(_) => 3,
            Value::ShortString(_) => 4,
            Value::String(_) => 5,
            Value::Table(_) => 6,
            Value::Function(_) => 7,
            Value::Closure(_) => 8,
        }
    }
}

impl PartialOrd for ValueKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.ord_priority().cmp(&other.ord_priority()) {
            Ordering::Equal => match (&self.0, &other.0) {
                (Value::Nil, Value::Nil) => Ordering::Equal,
                (Value::Boolean(lhs), Value::Boolean(rhs)) => lhs.cmp(rhs),
                (Value::Integer(lhs), Value::Integer(rhs)) => lhs.cmp(rhs),
                (Value::Float(lhs), Value::Float(rhs)) => lhs.total_cmp(rhs),
                (Value::ShortString(lhs), Value::ShortString(rhs)) => lhs.cmp(rhs),
                (Value::String(lhs), Value::String(rhs)) => lhs.cmp(rhs),
                (Value::Table(lhs), Value::Table(rhs)) => Rc::as_ptr(lhs).cmp(&Rc::as_ptr(rhs)),
                (Value::Function(lhs), Value::Function(rhs)) => lhs.cmp(rhs),
                (Value::Closure(lhs), Value::Closure(rhs)) => Rc::as_ptr(lhs).cmp(&Rc::as_ptr(rhs)),
                _ => unreachable!("Equal `ord_priority` means equal types"),
            },
            other => other,
        }
    }
}

impl From<Value> for ValueKey {
    fn from(value: Value) -> Self {
        Self(value)
    }
}
