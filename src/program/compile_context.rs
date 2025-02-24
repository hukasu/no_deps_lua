use alloc::vec::Vec;

use crate::value::Value;

use super::{exp_desc::ExpDesc, Error};

#[derive(Debug, Default)]
pub struct CompileContext<'a> {
    pub stack_top: u8,
    pub var_args: Option<bool>,
    pub locals: Vec<Value>,
    pub breaks: Option<Vec<usize>>,
    pub gotos: Vec<GotoLabel<'a>>,
    pub labels: Vec<GotoLabel<'a>>,
    pub jumps_to_block: Vec<usize>,
    pub jumps_to_end: Vec<usize>,
    pub jumps_to_false: Vec<usize>,
}

impl<'a> CompileContext<'a> {
    pub fn with_var_args(mut self, var_args: bool) -> Self {
        self.var_args = Some(var_args);
        self
    }

    pub fn reserve_stack_top(&mut self) -> (u8, ExpDesc<'a>) {
        let top = self.stack_top;
        self.stack_top += 1;
        (top, ExpDesc::Local(usize::from(top)))
    }

    pub fn push_goto(&mut self, goto_label: GotoLabel<'a>) {
        self.gotos.push(goto_label);
    }

    pub fn push_label(&mut self, goto_label: GotoLabel<'a>) -> Result<(), Error> {
        if self
            .labels
            .iter()
            .any(|label| label.name == goto_label.name)
        {
            Err(Error::LabelRedefinition)
        } else {
            self.labels.push(goto_label);
            Ok(())
        }
    }

    pub fn find_name(&self, name: &'a str) -> Option<ExpDesc<'a>> {
        let name: Value = name.into();
        self.locals
            .iter()
            .rposition(|local| local == &name)
            .map(ExpDesc::Local)
    }
}

#[derive(Debug)]
pub struct GotoLabel<'a> {
    pub name: &'a str,
    pub bytecode: usize,
    pub nvar: usize,
}
