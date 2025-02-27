use core::ops::{Deref, DerefMut};

use alloc::{boxed::Box, vec::Vec};

use crate::value::Value;

use super::exp_desc::ExpDesc;

macro_rules! make_exp_list {
    ($name:ident) => {
        #[must_use = "Contains a list of expressions that need to be discharged"]
        #[derive(Debug, Default, Clone, PartialEq)]
        pub struct $name<'a>(Vec<ExpDesc<'a>>);

        impl<'a> Deref for $name<'a> {
            type Target = Vec<ExpDesc<'a>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name<'_> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

make_exp_list!(ExpList);
make_exp_list!(VarList);

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
