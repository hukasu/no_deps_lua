use alloc::boxed::Box;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Local {
    name: Box<str>,
    scope_start: usize,
    scope_end: usize,
}

impl Local {
    pub fn new(name: Box<str>, scope_start: usize, scope_end: usize) -> Self {
        Self {
            name,
            scope_start,
            scope_end,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn new_no_end(name: Box<str>, scope_start: usize) -> Self {
        Self {
            name,
            scope_start,
            scope_end: usize::MAX,
        }
    }

    pub(crate) fn update_scope_end(&mut self, scope_end: usize) {
        assert_eq!(
            self.scope_end,
            usize::MAX,
            "`update_scope_end` should only be called once."
        );
        self.scope_end = scope_end;
    }
}
