#![no_std]

mod error;
mod ext;
mod lex;
mod parser;
mod program;
mod stack_str;
mod std;
mod value;

extern crate alloc;

use alloc::vec::Vec;

use self::{program::ByteCode, value::Value};

pub use {error::Error, program::Program};

#[derive(Debug)]
pub struct Lua {
    func_index: usize,
    globals: Vec<(Value, Value)>,
    stack: Vec<Value>,
}

impl Lua {
    fn new() -> Self {
        let globals = Vec::from([("print".into(), Value::Function(std::lib_print))]);

        Self {
            func_index: 0,
            globals,
            stack: Vec::new(),
        }
    }

    pub fn execute(program: &Program) -> Result<(), Error> {
        let mut vm = Self::new();

        for code in &program.byte_codes {
            match code {
                ByteCode::GetGlobal(dst, name) => {
                    let key = &program.constants[*name as usize];
                    if let Some(index) = vm.globals.iter().position(|global| global.0.eq(key)) {
                        vm.stack.insert(*dst as usize, vm.globals[index].1.clone());
                    } else {
                        vm.stack.insert(*dst as usize, Value::Nil);
                    }
                }
                ByteCode::SetGlobal(dst, src) => {
                    let key = &program.constants[*dst as usize];
                    let value = vm.stack[*src as usize].clone();
                    if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
                        global.1 = value;
                    } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
                        vm.globals.push((key.clone(), value));
                    } else {
                        return Err(Error::ExpectedName);
                    }
                }
                ByteCode::SetGlobalConstant(dst, src) => {
                    let key = &program.constants[*dst as usize];
                    let value = program.constants[*src as usize].clone();
                    if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
                        global.1 = value;
                    } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
                        vm.globals.push((key.clone(), value));
                    } else {
                        return Err(Error::ExpectedName);
                    }
                }
                ByteCode::SetGlobalGlobal(dst, src) => {
                    let dst_key = &program.constants[*dst as usize];
                    let src_key = &program.constants[*src as usize];
                    let value = vm
                        .globals
                        .iter()
                        .find(|global| global.0.eq(src_key))
                        .map_or(Value::Nil, |global| global.1.clone());
                    if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(dst_key))
                    {
                        global.1 = value;
                    } else if matches!(dst_key, Value::String(_) | Value::ShortString(_)) {
                        vm.globals.push((dst_key.clone(), value));
                    } else {
                        return Err(Error::ExpectedName);
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
                ByteCode::Move(dst, src) => vm
                    .stack
                    .insert(*dst as usize, vm.stack[*src as usize].clone()),
                ByteCode::Call(func, _) => {
                    vm.func_index = *func as usize;
                    let func = &vm.stack[vm.func_index];
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
