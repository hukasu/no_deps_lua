use alloc::vec::Vec;

use crate::value::Value;

#[derive(Debug, PartialEq)]
pub struct Table {
    pub array: Vec<Value>,
    pub table: Vec<(Value, Value)>,
}

impl Table {
    pub fn new(array_initial_size: usize, table_initial_size: usize) -> Self {
        Self {
            array: Vec::with_capacity(array_initial_size),
            table: Vec::with_capacity(table_initial_size),
        }
    }
}
