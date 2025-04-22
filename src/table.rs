use alloc::vec::Vec;

use crate::{
    Error,
    value::{Value, ValueKey},
};

#[derive(Debug, PartialEq)]
pub struct Table {
    pub array: Vec<Value>,
    pub table: Vec<(ValueKey, Value)>,
}

impl Table {
    pub fn new(array_initial_size: usize, table_initial_size: usize) -> Self {
        Self {
            array: Vec::with_capacity(array_initial_size),
            table: Vec::with_capacity(table_initial_size),
        }
    }

    pub fn get(&self, key: ValueKey) -> &Value {
        match self.table.binary_search_by_key(&&key, |(key, _)| key) {
            Ok(found) => &self.table[found].1,
            Err(_) => &Value::Nil,
        }
    }

    pub fn set(&mut self, key: ValueKey, value: Value) -> Result<(), Error> {
        match self.table.binary_search_by_key(&&key, |(key, _)| key) {
            Ok(index) => {
                self.table[index].1 = value;
                Ok(())
            }
            Err(index) => {
                if matches!(key, ValueKey(Value::ShortString(_) | Value::String(_))) {
                    self.table.insert(index, (key, value));
                    Ok(())
                } else {
                    Err(Error::ExpectedName)
                }
            }
        }
    }
}
