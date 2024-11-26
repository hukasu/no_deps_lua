#![no_std]

mod error;
mod ext;
mod lex;
mod parser;
mod program;
mod stack_str;
mod std;
mod table;
mod value;

extern crate alloc;

use core::{cell::RefCell, cmp::Ordering};

use alloc::{rc::Rc, vec::Vec};
use table::Table;

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

    fn set_stack(&mut self, dst: u8, value: Value) -> Result<(), Error> {
        let dst = usize::from(dst);
        match self.stack.len().cmp(&dst) {
            Ordering::Greater => {
                self.stack[dst] = value;
                Ok(())
            }
            Ordering::Equal => {
                self.stack.push(value);
                Ok(())
            }
            Ordering::Less => {
                log::error!("Trying to set a value out of the bounds of the stack.");
                Err(Error::StackOverflow)
            }
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
                ByteCode::SetGlobal(name, src) => {
                    let key = &program.constants[*name as usize];
                    let value = vm.stack[*src as usize].clone();
                    if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
                        global.1 = value;
                    } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
                        vm.globals.push((key.clone(), value));
                    } else {
                        return Err(Error::ExpectedName);
                    }
                }
                ByteCode::SetGlobalConstant(name, src) => {
                    let key = &program.constants[*name as usize];
                    let value = program.constants[*src as usize].clone();
                    if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
                        global.1 = value;
                    } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
                        vm.globals.push((key.clone(), value));
                    } else {
                        return Err(Error::ExpectedName);
                    }
                }
                ByteCode::SetGlobalInteger(key, value) => {
                    let key = &program.constants[*key as usize];
                    let value = (*value).into();
                    if let Some(global) = vm.globals.iter_mut().find(|global| global.0.eq(key)) {
                        global.1 = value;
                    } else if matches!(key, Value::String(_) | Value::ShortString(_)) {
                        vm.globals.push((key.clone(), value));
                    } else {
                        return Err(Error::ExpectedName);
                    }
                }
                ByteCode::SetGlobalGlobal(dst_name, src_name) => {
                    let dst_key = &program.constants[*dst_name as usize];
                    let src_key = &program.constants[*src_name as usize];
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
                ByteCode::LoadConstant(dst, key) => {
                    vm.stack
                        .insert(*dst as usize, program.constants[*key as usize].clone());
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
                ByteCode::NewTable(dst, array_initial_size, table_initial_size) => vm.stack.insert(
                    usize::from(*dst),
                    Value::Table(Rc::new(RefCell::new(Table::new(
                        usize::from(*array_initial_size),
                        usize::from(*table_initial_size),
                    )))),
                ),
                ByteCode::SetTable(table, key, value) => {
                    if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
                        let key = vm.stack[usize::from(*key)].clone();
                        let value = vm.stack[usize::from(*value)].clone();
                        let binary_search = (*table)
                            .borrow()
                            .table
                            .binary_search_by_key(&&key, |a| &a.0);
                        match binary_search {
                            Ok(i) => {
                                let mut table_borrow = table.borrow_mut();
                                let Some(table_value) = table_borrow.table.get_mut(i) else {
                                    unreachable!("Already tested existence of table value");
                                };
                                table_value.1 = value;
                            }
                            Err(i) => table.borrow_mut().table.insert(i, (key, value)),
                        }
                    } else {
                        return Err(Error::ExpectedTable);
                    }
                }
                ByteCode::SetField(table, key, value) => {
                    if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
                        let key = program.constants[usize::from(*key)].clone();
                        let value = vm.stack[usize::from(*value)].clone();
                        let binary_search = (*table)
                            .borrow()
                            .table
                            .binary_search_by_key(&&key, |a| &a.0);
                        match binary_search {
                            Ok(i) => {
                                let mut table_borrow = table.borrow_mut();
                                let Some(table_value) = table_borrow.table.get_mut(i) else {
                                    unreachable!("Already tested existence of table value");
                                };
                                table_value.1 = value;
                            }
                            Err(i) => table.borrow_mut().table.insert(i, (key, value)),
                        }
                    } else {
                        return Err(Error::ExpectedTable);
                    }
                }
                ByteCode::SetList(table, array_len) => {
                    let table_items_start = usize::from(*table) + 1;
                    if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
                        let values = vm.stack.drain(
                            table_items_start..(table_items_start + usize::from(*array_len)),
                        );
                        table.borrow_mut().array.extend(values);
                    } else {
                        return Err(Error::ExpectedTable);
                    }
                }
                ByteCode::GetTable(dst, table, src) => {
                    if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
                        let key = &vm.stack[usize::from(*src)];
                        let bin_search =
                            (*table).borrow().table.binary_search_by_key(&key, |a| &a.0);
                        let value = match bin_search {
                            Ok(i) => (*table).borrow().table[i].1.clone(),
                            Err(_) => Value::Nil,
                        };
                        vm.stack.insert(*dst as usize, value);
                    } else {
                        return Err(Error::ExpectedTable);
                    }
                }
                ByteCode::GetField(dst, table, key) => {
                    if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
                        let key = &program.constants[usize::from(*key)];
                        let bin_search =
                            (*table).borrow().table.binary_search_by_key(&key, |a| &a.0);
                        let value = match bin_search {
                            Ok(i) => (*table).borrow().table[i].1.clone(),
                            Err(_) => Value::Nil,
                        };
                        vm.stack.insert(*dst as usize, value);
                    } else {
                        return Err(Error::ExpectedTable);
                    }
                }
                ByteCode::GetInt(dst, table, index) => {
                    if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
                        let value = if index == &0 {
                            let bin_search = (*table)
                                .borrow()
                                .table
                                .binary_search_by_key(&&Value::Integer(0), |a| &a.0);
                            match bin_search {
                                Ok(i) => (*table).borrow().table[i].1.clone(),
                                Err(_) => Value::Nil,
                            }
                        } else {
                            (*table).borrow().array[usize::from(*index) - 1].clone()
                        };
                        vm.stack.insert(*dst as usize, value);
                    } else {
                        return Err(Error::ExpectedTable);
                    }
                }
                ByteCode::Not(dst, src) => {
                    let value = match &vm.stack[usize::from(*src)] {
                        Value::Boolean(false) | Value::Nil => Value::Boolean(true),
                        _ => Value::Boolean(false),
                    };
                    vm.set_stack(*dst, value)?;
                }
                ByteCode::Len(dst, src) => {
                    let value = match &vm.stack[usize::from(*src)] {
                        Value::String(string) => Value::Integer(i64::try_from(string.len())?),
                        Value::ShortString(string) => Value::Integer(i64::try_from(string.len())?),
                        _ => return Err(Error::InvalidLenOperand),
                    };
                    vm.set_stack(*dst, value)?;
                }
                ByteCode::Neg(dst, src) => {
                    let value = match vm.stack[usize::from(*src)] {
                        Value::Integer(integer) => Value::Integer(-integer),
                        Value::Float(float) => Value::Float(-float),
                        _ => return Err(Error::InvalidNegOperand),
                    };
                    vm.set_stack(*dst, value)?;
                }
                ByteCode::BitNot(dst, src) => {
                    let value = match vm.stack[usize::from(*src)] {
                        Value::Integer(integer) => Value::Integer(!integer),
                        _ => return Err(Error::InvalidBitNotOperand),
                    };
                    vm.set_stack(*dst, value)?;
                }
                ByteCode::Move(dst, src) => vm
                    .stack
                    .insert(*dst as usize, vm.stack[*src as usize].clone()),
                ByteCode::Call(func, _args) => {
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
