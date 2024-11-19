#![no_std]

mod error;
mod lex;
mod parser;
mod program;
mod stack_str;
mod std;
mod value;

extern crate alloc;

use alloc::{rc::Rc, vec::Vec};

use self::{program::ByteCode, value::Value};

pub use {error::Error, program::Program};

pub struct Lua {
    func_index: usize,
    globals: Vec<(Rc<str>, Value)>,
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

        for code in program.byte_codes.iter() {
            match code {
                ByteCode::GetGlobal(dst, name) => {
                    let name = &program.constants[*name as usize];
                    if let Value::String(key) = name {
                        if let Some(index) = vm.globals.iter().position(|global| global.0.eq(key)) {
                            vm.stack.insert(*dst as usize, vm.globals[index].1.clone());
                        } else {
                            vm.stack.insert(*dst as usize, Value::Nil);
                        }
                    } else {
                        return Err(Error::InvalidGlobalKey(name.clone()));
                    }
                }
                ByteCode::SetGlobal(dst, src) => {
                    let name = &program.constants[*dst as usize];
                    if let Value::String(global_name) = name {
                        let value = vm.stack[*src as usize].clone();
                        if let Some(global) = vm
                            .globals
                            .iter_mut()
                            .find(|global| global.0.eq(global_name))
                        {
                            global.1 = value;
                        } else {
                            vm.globals.push((global_name.clone(), value));
                        }
                    } else {
                        return Err(Error::ExpectedName);
                    }
                }
                ByteCode::SetGlobalConstant(dst, src) => {
                    let name = &program.constants[*dst as usize];
                    if let Value::String(global_name) = name {
                        let value = program.constants[*src as usize].clone();
                        if let Some(global) = vm
                            .globals
                            .iter_mut()
                            .find(|global| global.0.eq(global_name))
                        {
                            global.1 = value;
                        } else {
                            vm.globals.push((global_name.clone(), value));
                        }
                    } else {
                        return Err(Error::ExpectedName);
                    }
                }
                ByteCode::SetGlobalGlobal(dst, src) => {
                    let name = &program.constants[*dst as usize];
                    if let Value::String(global_name) = name {
                        let value = &program.constants[*src as usize];
                        if let Value::String(src_global_name) = value {
                            let value = vm
                                .globals
                                .iter()
                                .find(|global| global.0.eq(src_global_name))
                                .map(|global| global.1.clone())
                                .unwrap_or(Value::Nil);
                            if let Some(global) = vm
                                .globals
                                .iter_mut()
                                .find(|global| global.0.eq(global_name))
                            {
                                global.1 = value;
                            } else {
                                vm.globals.push((global_name.clone(), value));
                            }
                        }
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
