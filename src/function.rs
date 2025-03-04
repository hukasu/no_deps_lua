use super::Program;

#[derive(Debug)]
pub struct Function {
    program: Program,
    arg_count: usize,
    variadic_args: bool,
}

impl Function {
    pub fn new(program: Program, arg_count: usize, variadic_args: bool) -> Self {
        Self {
            program,
            arg_count,
            variadic_args,
        }
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn arg_count(&self) -> usize {
        self.arg_count
    }

    pub fn variadic_args(&self) -> bool {
        self.variadic_args
    }
}

impl From<Program> for Function {
    fn from(program: Program) -> Self {
        Self {
            program,
            arg_count: 0,
            variadic_args: false,
        }
    }
}
