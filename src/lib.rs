#![no_std]

mod error;
mod lex;
mod parser;
mod program;
mod std;
mod value;

extern crate alloc;

use alloc::vec::Vec;

use self::{program::ByteCode, value::Value};

pub use {error::Error, program::Program};

pub struct Lua<'a> {
    globals: Vec<(&'static str, Value<'a>)>,
    stack: Vec<Value<'a>>,
}

impl<'a> Lua<'a> {
    fn new() -> Self {
        let globals = Vec::from([("print", Value::Function(std::lib_print))]);

        Self {
            globals,
            stack: Vec::new(),
        }
    }

    pub fn execute(program: &Program<'a>) -> Result<(), Error<'a>> {
        let mut vm = Self::new();

        for code in program.byte_codes.iter() {
            match code {
                ByteCode::GetGlobal(dst, name) => {
                    let name = &program.constants[*name as usize];
                    if let Value::String(key) = name {
                        let bin_search = vm.globals.binary_search_by(|(a, _)| a.cmp(key));
                        if let Ok(index) = bin_search {
                            vm.stack.insert(*dst as usize, vm.globals[index].1.clone());
                        } else {
                            vm.stack.insert(*dst as usize, Value::Nil);
                        }
                    } else {
                        return Err(Error::InvalidGlobalKey(name.clone()));
                    }
                }
                ByteCode::LoadNil(dst) => {
                    vm.stack.insert(*dst as usize, Value::Nil);
                }
                ByteCode::LoadBool(dst, value) => {
                    vm.stack.insert(*dst as usize, Value::Boolean(*value));
                }
                ByteCode::LoadInt(dst, value) => vm
                    .stack
                    .insert(*dst as usize, Value::Integer(i64::from(*value))),
                ByteCode::LoadConstant(dst, key) => {
                    vm.stack
                        .insert(*dst as usize, program.constants[*key as usize].clone());
                }
                ByteCode::Call(func, _) => {
                    let func = &vm.stack[*func as usize];
                    if let Value::Function(f) = func {
                        f(&mut vm);
                    } else {
                        return Err(Error::InvalidFunction(func.clone()));
                    }
                }
            }
        }

        Ok(())
    }
}
