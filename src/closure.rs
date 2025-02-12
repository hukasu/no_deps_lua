use super::Program;

#[derive(Debug)]
pub struct Closure {
    program: Program,
    arg_count: usize,
    variadic_args: bool,
}

impl Closure {
    pub fn new(program: Program, arg_count: usize, variadic_args: bool) -> Self {
        Self {
            program,
            arg_count,
            variadic_args,
        }
    }

    pub fn from_program(program: Program) -> Self {
        Self {
            program,
            arg_count: 0,
            variadic_args: false,
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
