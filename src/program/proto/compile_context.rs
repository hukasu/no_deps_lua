use alloc::{boxed::Box, collections::btree_set::BTreeSet, vec::Vec};

use crate::program::Error;

use super::exp_desc::ExpDesc;

#[derive(Debug, Default)]
pub struct CompileContext<'a> {
    pub stack_top: u8,
    pub var_args: Option<bool>,
    pub locals: Vec<Box<str>>,
    pub breaks: Option<Vec<usize>>,
    pub gotos: Vec<GotoLabel<'a>>,
    pub labels: Vec<GotoLabel<'a>>,
    pub jumps_to_block: Vec<usize>,
    pub jumps_to_end: Vec<usize>,
    pub captured_locals: BTreeSet<usize>,
}

impl<'a> CompileContext<'a> {
    pub fn new_with_var_args(var_args: bool) -> Self {
        Self {
            var_args: Some(var_args),
            ..Default::default()
        }
    }

    pub fn reserve_stack_top(&mut self) -> (u8, ExpDesc<'a>) {
        let top = self.stack_top;
        self.stack_top += 1;
        (top, ExpDesc::Local(usize::from(top)))
    }

    pub fn find_name(&self, name: &'a str) -> Option<usize> {
        self.locals.iter().rposition(|local| local.as_ref() == name)
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

    pub fn push_capture(&mut self, local: usize) {
        self.captured_locals.insert(local);
    }

    pub fn clear_captures_above(&mut self, first_local: usize) -> bool {
        if let Some(max) = self
            .captured_locals
            .iter()
            .max()
            .filter(|local| **local >= first_local)
            .copied()
        {
            self.captured_locals.retain(|local| *local < max);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GotoLabel<'a> {
    pub name: &'a str,
    pub bytecode: usize,
    pub nvar: usize,
}
