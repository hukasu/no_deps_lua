#![no_std]

mod error;
mod lex;
mod program;
mod std;
mod value;

extern crate alloc;

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use program::ByteCode;
use value::Value;

pub use {error::Error, program::Program};

pub struct Lua {
    globals: Vec<(String, Value)>,
    stack: Vec<Value>,
}

impl Lua {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), Error> {
        for code in program.byte_codes.iter() {
            match code {
                ByteCode::GetGlobal(dst, name) => {
                    let name = &program.constants[*name as usize];
                    if let Value::String(key) = name {
                        let bin_search = self.globals.binary_search_by(|(a, _)| a.cmp(key));
                        if let Ok(index) = bin_search {
                            self.stack
                                .insert(*dst as usize, self.globals[index].1.clone());
                        } else {
                            self.stack.insert(*dst as usize, Value::Nil);
                        }
                    } else {
                        return Err(Error::InvalidGlobalKey(name.clone()));
                    }
                }
                ByteCode::LoadConstant(dst, key) => {
                    self.stack
                        .insert(*dst as usize, program.constants[*key as usize].clone());
                }
                ByteCode::Call(func, _) => {
                    let func = &self.stack[*func as usize];
                    if let Value::Function(f) = func {
                        f(self);
                    } else {
                        return Err(Error::InvalidFunction(func.clone()));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for Lua {
    #[allow(clippy::vec_init_then_push)]
    fn default() -> Self {
        let mut globals = Vec::new();
        globals.push(("print".to_owned(), Value::Function(std::lib_print)));

        Self {
            globals,
            stack: Vec::new(),
        }
    }
}
