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

use alloc::{format, rc::Rc, vec::Vec};

use self::{ext::FloatExt, program::ByteCode, table::Table, value::Value};

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
                ByteCode::Move(dst, src) => vm.set_stack(*dst, vm.stack[*src as usize].clone())?,
                ByteCode::LoadInt(dst, value) => {
                    vm.set_stack(*dst, Value::Integer(i64::from(*value)))?;
                }
                ByteCode::LoadFloat(dst, value) => {
                    vm.set_stack(*dst, Value::Float(*value as f64))?;
                }
                ByteCode::LoadConstant(dst, key) => {
                    vm.set_stack(*dst, program.constants[*key as usize].clone())?;
                }
                ByteCode::LoadFalse(dst) => {
                    vm.set_stack(*dst, Value::Boolean(false))?;
                }
                ByteCode::LoadTrue(dst) => {
                    vm.set_stack(*dst, Value::Boolean(true))?;
                }
                ByteCode::LoadNil(dst) => {
                    vm.set_stack(*dst, Value::Nil)?;
                }
                ByteCode::GetGlobal(dst, name) => {
                    let key = &program.constants[*name as usize];
                    if let Some(index) = vm.globals.iter().position(|global| global.0.eq(key)) {
                        vm.set_stack(*dst, vm.globals[index].1.clone())?;
                    } else {
                        vm.set_stack(*dst, Value::Nil)?;
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
                ByteCode::GetTable(dst, table, src) => {
                    if let Value::Table(table) = vm.stack[usize::from(*table)].clone() {
                        let key = &vm.stack[usize::from(*src)];
                        let bin_search =
                            (*table).borrow().table.binary_search_by_key(&key, |a| &a.0);
                        let value = match bin_search {
                            Ok(i) => (*table).borrow().table[i].1.clone(),
                            Err(_) => Value::Nil,
                        };
                        vm.set_stack(*dst, value)?;
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
                        vm.set_stack(*dst, value)?;
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
                        vm.set_stack(*dst, value)?;
                    } else {
                        return Err(Error::ExpectedTable);
                    }
                }
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
                ByteCode::NewTable(dst, array_initial_size, table_initial_size) => {
                    vm.set_stack(
                        *dst,
                        Value::Table(Rc::new(RefCell::new(Table::new(
                            usize::from(*array_initial_size),
                            usize::from(*table_initial_size),
                        )))),
                    )?;
                }
                ByteCode::AddInteger(_, _, _) => {
                    todo!("AddInteger")
                }
                ByteCode::AddConstant(_, _, _) => {
                    todo!("AddConstant")
                }
                ByteCode::Add(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
                        (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
                        (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 + r),
                        (Value::Float(l), Value::Integer(r)) => Value::Float(l + *r as f64),
                        (Value::Nil, _) => return Err(Error::NilArithmetic),
                        (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringArithmetic)
                        }
                        (Value::Table(_), _) => return Err(Error::TableArithmetic),
                        (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
                        (_, Value::Nil) => return Err(Error::NilArithmetic),
                        (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringArithmetic)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableArithmetic),
                        (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::Sub(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
                        (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
                        (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 - r),
                        (Value::Float(l), Value::Integer(r)) => Value::Float(l - *r as f64),
                        (Value::Nil, _) => return Err(Error::NilArithmetic),
                        (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringArithmetic)
                        }
                        (Value::Table(_), _) => return Err(Error::TableArithmetic),
                        (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
                        (_, Value::Nil) => return Err(Error::NilArithmetic),
                        (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringArithmetic)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableArithmetic),
                        (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::Mul(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
                        (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                        (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 * r),
                        (Value::Float(l), Value::Integer(r)) => Value::Float(l * *r as f64),
                        (Value::Nil, _) => return Err(Error::NilArithmetic),
                        (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringArithmetic)
                        }
                        (Value::Table(_), _) => return Err(Error::TableArithmetic),
                        (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
                        (_, Value::Nil) => return Err(Error::NilArithmetic),
                        (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringArithmetic)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableArithmetic),
                        (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::Mod(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l % r),
                        (Value::Float(l), Value::Float(r)) => Value::Float(l % r),
                        (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 % r),
                        (Value::Float(l), Value::Integer(r)) => Value::Float(l % *r as f64),
                        (Value::Nil, _) => return Err(Error::NilArithmetic),
                        (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringArithmetic)
                        }
                        (Value::Table(_), _) => return Err(Error::TableArithmetic),
                        (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
                        (_, Value::Nil) => return Err(Error::NilArithmetic),
                        (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringArithmetic)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableArithmetic),
                        (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::Pow(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => {
                            Value::Float((*l as f64).powf(*r as f64))
                        }
                        (Value::Float(l), Value::Float(r)) => Value::Float(l.powf(*r)),
                        (Value::Integer(l), Value::Float(r)) => Value::Float((*l as f64).powf(*r)),
                        (Value::Float(l), Value::Integer(r)) => Value::Float(l.powf(*r as f64)),
                        (Value::Nil, _) => return Err(Error::NilArithmetic),
                        (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringArithmetic)
                        }
                        (Value::Table(_), _) => return Err(Error::TableArithmetic),
                        (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
                        (_, Value::Nil) => return Err(Error::NilArithmetic),
                        (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringArithmetic)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableArithmetic),
                        (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::Div(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => {
                            Value::Float(*l as f64 / *r as f64)
                        }
                        (Value::Float(l), Value::Float(r)) => Value::Float(l / r),
                        (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 / r),
                        (Value::Float(l), Value::Integer(r)) => Value::Float(l / *r as f64),
                        (Value::Nil, _) => return Err(Error::NilArithmetic),
                        (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringArithmetic)
                        }
                        (Value::Table(_), _) => return Err(Error::TableArithmetic),
                        (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
                        (_, Value::Nil) => return Err(Error::NilArithmetic),
                        (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringArithmetic)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableArithmetic),
                        (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::Idiv(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l / r),
                        (Value::Float(l), Value::Float(r)) => Value::Float((l / r).trunc()),
                        (Value::Integer(l), Value::Float(r)) => {
                            Value::Float((*l as f64 / r).trunc())
                        }
                        (Value::Float(l), Value::Integer(r)) => {
                            Value::Float((l / *r as f64).trunc())
                        }
                        (Value::Nil, _) => return Err(Error::NilArithmetic),
                        (Value::Boolean(_), _) => return Err(Error::BoolArithmetic),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringArithmetic)
                        }
                        (Value::Table(_), _) => return Err(Error::TableArithmetic),
                        (Value::Function(_), _) => return Err(Error::FunctionArithmetic),
                        (_, Value::Nil) => return Err(Error::NilArithmetic),
                        (_, Value::Boolean(_)) => return Err(Error::BoolArithmetic),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringArithmetic)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableArithmetic),
                        (_, Value::Function(_)) => return Err(Error::FunctionArithmetic),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::BitAnd(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l & r),
                        (Value::Float(_), _) => return Err(Error::FloatBitwise),
                        (Value::Nil, _) => return Err(Error::NilBitwise),
                        (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringBitwise)
                        }
                        (Value::Table(_), _) => return Err(Error::TableBitwise),
                        (Value::Function(_), _) => return Err(Error::FunctionBitwise),
                        (_, Value::Float(_)) => return Err(Error::FloatBitwise),
                        (_, Value::Nil) => return Err(Error::NilBitwise),
                        (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringBitwise)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableBitwise),
                        (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::BitOr(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l | r),
                        (Value::Float(_), _) => return Err(Error::FloatBitwise),
                        (Value::Nil, _) => return Err(Error::NilBitwise),
                        (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringBitwise)
                        }
                        (Value::Table(_), _) => return Err(Error::TableBitwise),
                        (Value::Function(_), _) => return Err(Error::FunctionBitwise),
                        (_, Value::Float(_)) => return Err(Error::FloatBitwise),
                        (_, Value::Nil) => return Err(Error::NilBitwise),
                        (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringBitwise)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableBitwise),
                        (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::BitXor(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l ^ r),
                        (Value::Float(_), _) => return Err(Error::FloatBitwise),
                        (Value::Nil, _) => return Err(Error::NilBitwise),
                        (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringBitwise)
                        }
                        (Value::Table(_), _) => return Err(Error::TableBitwise),
                        (Value::Function(_), _) => return Err(Error::FunctionBitwise),
                        (_, Value::Float(_)) => return Err(Error::FloatBitwise),
                        (_, Value::Nil) => return Err(Error::NilBitwise),
                        (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringBitwise)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableBitwise),
                        (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::ShiftL(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l << r),
                        (Value::Float(_), _) => return Err(Error::FloatBitwise),
                        (Value::Nil, _) => return Err(Error::NilBitwise),
                        (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringBitwise)
                        }
                        (Value::Table(_), _) => return Err(Error::TableBitwise),
                        (Value::Function(_), _) => return Err(Error::FunctionBitwise),
                        (_, Value::Float(_)) => return Err(Error::FloatBitwise),
                        (_, Value::Nil) => return Err(Error::NilBitwise),
                        (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringBitwise)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableBitwise),
                        (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
                    };
                    vm.set_stack(*dst, res)?;
                }
                ByteCode::ShiftR(dst, lhs, rhs) => {
                    let res = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l >> r),
                        (Value::Float(l), Value::Float(r)) => {
                            if l.zero_frac() && r.zero_frac() {
                                Value::Integer((*l as i64) >> (*r as i64))
                            } else {
                                return Err(Error::FloatBitwise);
                            }
                        }
                        (Value::Float(l), Value::Integer(r)) => {
                            if l.zero_frac() {
                                Value::Integer((*l as i64) >> r)
                            } else {
                                return Err(Error::FloatBitwise);
                            }
                        }
                        (Value::Integer(l), Value::Float(r)) => {
                            if r.zero_frac() {
                                Value::Integer(l >> (*r as i64))
                            } else {
                                return Err(Error::FloatBitwise);
                            }
                        }
                        (Value::Nil, _) => return Err(Error::NilBitwise),
                        (Value::Boolean(_), _) => return Err(Error::BoolBitwise),
                        (Value::String(_) | Value::ShortString(_), _) => {
                            return Err(Error::StringBitwise)
                        }
                        (Value::Table(_), _) => return Err(Error::TableBitwise),
                        (Value::Function(_), _) => return Err(Error::FunctionBitwise),
                        (_, Value::Float(_)) => return Err(Error::FloatBitwise),
                        (_, Value::Nil) => return Err(Error::NilBitwise),
                        (_, Value::Boolean(_)) => return Err(Error::BoolBitwise),
                        (_, Value::String(_) | Value::ShortString(_)) => {
                            return Err(Error::StringBitwise)
                        }
                        (_, Value::Table(_)) => return Err(Error::TableBitwise),
                        (_, Value::Function(_)) => return Err(Error::FunctionBitwise),
                    };
                    vm.set_stack(*dst, res)?;
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
                ByteCode::Concat(dst, lhs, rhs) => {
                    let value = match (&vm.stack[usize::from(*lhs)], &vm.stack[usize::from(*rhs)]) {
                        (Value::Nil, _) => return Err(Error::NilConcat),
                        (Value::Boolean(_), _) => return Err(Error::BoolConcat),
                        (Value::Table(_), _) => return Err(Error::TableConcat),
                        (Value::Function(_), _) => return Err(Error::FunctionConcat),
                        (_, Value::Nil) => return Err(Error::NilConcat),
                        (_, Value::Boolean(_)) => return Err(Error::BoolConcat),
                        (_, Value::Table(_)) => return Err(Error::TableConcat),
                        (_, Value::Function(_)) => return Err(Error::FunctionConcat),
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            format!("{:?}{:?}", lhs, rhs).as_str().into()
                        }
                        (Value::Float(lhs), rhs) => format!("{:?}{}", lhs, rhs).as_str().into(),
                        (lhs, Value::Float(rhs)) => format!("{}{:?}", lhs, rhs).as_str().into(),
                        (lhs, rhs) => format!("{}{}", lhs, rhs).as_str().into(),
                    };
                    vm.set_stack(*dst, value)?;
                }
                ByteCode::Call(func, _args) => {
                    vm.func_index = *func as usize;
                    let func = &vm.stack[vm.func_index];
                    if let Value::Function(f) = func {
                        f(&mut vm);
                    } else {
                        return Err(Error::InvalidFunction(func.clone()));
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
            }
        }

        Ok(())
    }
}
