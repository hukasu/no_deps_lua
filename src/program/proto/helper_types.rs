use alloc::{boxed::Box, vec::Vec};

use crate::value::Value;

use super::exp_desc::ExpDesc;

pub type NameList = Vec<Value>;
pub type TableFields<'a> = Vec<(TableKey<'a>, ExpDesc<'a>)>;

#[must_use = "Contains a list of names that need to be added to constants"]
#[derive(Debug, Default)]
pub struct ParList {
    pub names: Vec<Value>,
    pub variadic_args: bool,
}

#[must_use = "Contains a list of names that need to be added to constants"]
#[derive(Debug, Default)]
pub struct FunctionNameList<'a> {
    pub names: Vec<&'a str>,
    pub has_method: bool,
}

#[must_use = "Contains a key to index into a table"]
#[derive(Debug, Clone, PartialEq)]
pub enum TableKey<'a> {
    Array,
    Record(Box<ExpDesc<'a>>),
    General(Box<ExpDesc<'a>>),
}
