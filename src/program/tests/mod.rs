//! Tests from [`wubingzheng`](https://wubingzheng.github.io/build-lua-in-rust/en/PREFACE.html)'s book

use alloc::boxed::Box;

use crate::{bytecode::Bytecode, value::Value};

use super::{Local, Program};

mod chapter1;
mod chapter2;
mod chapter3;
mod chapter4;
mod chapter5;
mod chapter6;
mod chapter7;
mod chapter8;
// mod chapter9;

fn compare_program(
    program: &Program,
    bytecodes: &[Bytecode],
    constants: &[Value],
    locals: &[Local],
    upvalues: &[Box<str>],
    function_count: usize,
) {
    assert_eq!(program.byte_codes.as_ref(), bytecodes);
    assert_eq!(program.constants.as_ref(), constants);
    assert_eq!(program.locals.as_ref(), locals);
    assert_eq!(program.upvalues.as_ref(), upvalues);
    assert_eq!(program.functions.len(), function_count);
}

fn get_closure_program(program: &Program, closure_id: usize) -> &Program {
    program.functions[closure_id].program()
}
