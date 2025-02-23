#[derive(Debug)]
pub struct StackFrame {
    /// Function index
    pub function_index: FunctionIndex,
    /// Program counter of the current program
    pub program_counter: usize,
    /// The location on stack immediately after the function's location.
    ///
    /// The main program does not insert it's location in this list.
    pub stack_frame: usize,
    /// Number of variadic arguments in each function stack.
    pub variadic_arguments: usize,
    /// Number of values that should be moved at the end of a call
    pub out_params: usize,
}

#[derive(Debug)]
pub enum FunctionIndex {
    Main,
    Closure(usize),
}
