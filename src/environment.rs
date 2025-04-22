use core::{
    cell::RefCell,
    cmp::Ordering,
    fmt::Display,
    num::TryFromIntError,
    ops::{Deref, DerefMut},
};

use alloc::{rc::Rc, vec};

use crate::{
    closure::{Closure, NativeClosure, Upvalue},
    std,
    table::Table,
    value::{Value, ValueKey},
};

pub struct Environment(Rc<RefCell<Table>>);

impl Environment {
    pub fn push(
        &mut self,
        value_key: impl Into<Value>,
        value: impl Into<Value>,
    ) -> Result<(), EnvironmentError> {
        let mut table = self.borrow_mut();
        match value_key.into() {
            Value::Integer(array_item @ 1..) => {
                let index = usize::try_from(array_item)?;
                match index.cmp(&table.array.len()) {
                    Ordering::Greater => {
                        table.array.resize(index, Value::Nil);
                        table.array.push(value.into());
                    }
                    Ordering::Equal => {
                        table.array.push(value.into());
                    }
                    Ordering::Less => {
                        table.array[index] = value.into();
                    }
                }
            }
            table_item => {
                let key = ValueKey(table_item);
                match table.table.binary_search_by_key(&&key, |a| &a.0) {
                    Ok(index) => table.table[index].1 = value.into(),
                    Err(index) => table.table.insert(index, (key, value.into())),
                }
            }
        }
        Ok(())
    }
}

impl Default for Environment {
    fn default() -> Self {
        let mut table = Table::new(0, 4);

        table.table.extend([
            (
                ValueKey("assert".into()),
                Value::from(std::lib_assert as NativeClosure),
            ),
            (
                ValueKey("print".into()),
                Value::from(std::lib_print as NativeClosure),
            ),
            (
                ValueKey("type".into()),
                Value::from(std::lib_type as NativeClosure),
            ),
            (
                ValueKey("warn".into()),
                Value::Closure(Rc::new(Closure::new_native(
                    std::lib_warn,
                    vec![Rc::new(RefCell::new(Upvalue::Closed(Value::Boolean(
                        false,
                    ))))],
                ))),
            ),
        ]);

        table.table.sort_by_key(|val| val.0.clone());

        Self(Rc::new(RefCell::new(table)))
    }
}

impl Deref for Environment {
    type Target = Rc<RefCell<Table>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Environment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub enum EnvironmentError {
    ArrayOutOfBounds,
}

impl Display for EnvironmentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ArrayOutOfBounds => {
                write!(
                    f,
                    "Tried to push a value out of bounds of system's `usize`."
                )
            }
        }
    }
}

impl From<TryFromIntError> for EnvironmentError {
    fn from(_value: TryFromIntError) -> Self {
        Self::ArrayOutOfBounds
    }
}
