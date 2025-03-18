use alloc::{boxed::Box, vec::Vec};

use super::{exp_desc::ExpDesc, Error};

#[derive(Debug, Default)]
pub struct CompileContext<'a> {
    pub stack_top: u8,
    pub previous_context: Option<&'a CompileContext<'a>>,
    pub var_args: Option<bool>,
    pub locals: Vec<Box<str>>,
    pub breaks: Option<Vec<usize>>,
    pub gotos: Vec<GotoLabel<'a>>,
    pub labels: Vec<GotoLabel<'a>>,
    pub jumps_to_block: Vec<usize>,
    pub jumps_to_end: Vec<usize>,
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

    pub fn find_name(&self, name: &'a str) -> Option<usize> {
        self.locals.iter().rposition(|local| local.as_ref() == name)
    }

    pub fn exists_in_upvalue(&self, name: &'a str) -> bool {
        if self
            .locals
            .iter()
            .any(|local| local.as_ref() == name || local.as_ref() == "_ENV")
        {
            true
        } else {
            self.previous_context
                .filter(|context| context.exists_in_upvalue(name))
                .is_some()
        }
    }
}

#[derive(Debug)]
pub struct GotoLabel<'a> {
    pub name: &'a str,
    pub bytecode: usize,
    pub nvar: usize,
}
