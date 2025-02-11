use alloc::vec::Vec;

use crate::value::Value;

use super::{exp_desc::ExpDesc, Error};

#[derive(Debug)]
pub struct CompileContext<'a> {
    pub stack_top: u8,
    pub locals: Vec<Value>,
    pub breaks: Option<Vec<usize>>,
    pub gotos: Vec<GotoLabel<'a>>,
    pub labels: Vec<GotoLabel<'a>>,
    pub jumps_to_block: Vec<usize>,
    pub jumps_to_end: Vec<usize>,
    pub last_expdesc_was_or: bool,
}

impl<'a> CompileContext<'a> {
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
}

#[derive(Debug)]
pub struct GotoLabel<'a> {
    pub name: &'a str,
    pub bytecode: usize,
    pub nvar: usize,
}
