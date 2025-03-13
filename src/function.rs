use super::Program;

#[derive(Debug, Clone)]
pub struct Function {
    program: Program,
    arg_count: usize,
    variadic_args: bool,
}

impl Function {
    pub const fn new(program: Program, arg_count: usize, variadic_args: bool) -> Self {
        Self {
            program,
            arg_count,
            variadic_args,
        }
    }

    pub const fn program(&self) -> &Program {
        &self.program
    }

    pub const fn arg_count(&self) -> usize {
        self.arg_count
    }

    pub const fn variadic_args(&self) -> bool {
        self.variadic_args
    }
}

impl From<Program> for Function {
    fn from(program: Program) -> Self {
        Self::new(program, 0, false)
    }
}
