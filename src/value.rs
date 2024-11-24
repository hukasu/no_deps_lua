use core::{
    cell::RefCell,
    cmp::Ordering,
    fmt::{Debug, Display},
};

use alloc::rc::Rc;

use crate::{stack_str::StackStr, table::Table, Lua};

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
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Nil, Self::Nil) => Ordering::Equal,
            (Self::Nil, _) => Ordering::Less,
            (Self::Boolean(_), Self::Nil) => Ordering::Greater,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs.cmp(rhs),
            (Self::Boolean(_), _) => Ordering::Less,
            (Self::Integer(_), Self::Nil | Self::Boolean(_)) => Ordering::Greater,
            (Self::Integer(lhs), Self::Integer(rhs)) => lhs.cmp(rhs),
            (Self::Integer(_), _) => Ordering::Less,
            (Self::Float(_), Self::Nil | Self::Boolean(_) | Self::Integer(_)) => Ordering::Greater,
            (Self::Float(lhs), Self::Float(rhs)) => lhs.total_cmp(rhs),
            (Self::Float(_), _) => Ordering::Less,
            (
                Self::ShortString(_),
                Self::Nil | Self::Boolean(_) | Self::Integer(_) | Self::Float(_),
            ) => Ordering::Greater,
            (Self::ShortString(lhs), Self::ShortString(rhs)) => lhs.cmp(rhs),
            (Self::ShortString(_), _) => Ordering::Less,
            (
                Self::String(_),
                Self::Nil
                | Self::Boolean(_)
                | Self::Integer(_)
                | Self::Float(_)
                | Self::ShortString(_),
            ) => Ordering::Greater,
            (Self::String(lhs), Self::String(rhs)) => lhs.cmp(rhs),
            (Self::String(_), _) => Ordering::Less,
            (
                Self::Table(_),
                Self::Nil
                | Self::Boolean(_)
                | Self::Integer(_)
                | Self::Float(_)
                | Self::ShortString(_)
                | Self::String(_),
            ) => Ordering::Greater,
            (Self::Table(lhs), Self::Table(rhs)) => lhs.as_ptr().cmp(&rhs.as_ptr()),
            (Self::Table(_), _) => Ordering::Less,
            (Self::Function(lhs), Self::Function(rhs)) => lhs.cmp(rhs),
            (Self::Function(_), _) => Ordering::Greater,
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
            Self::Function(_) => write!(f, "function"),
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
