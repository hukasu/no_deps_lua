use core::cell::RefCell;

use alloc::{rc::Rc, vec::Vec};

use crate::{Error, Lua, Program, function::Function, value::Value};

#[derive(Debug)]
pub struct Closure {
    closure_type: FunctionType,
    upvalues: Vec<Rc<RefCell<Upvalue>>>,
}

impl Closure {
    pub const fn new_lua(function: Rc<Function>, upvalues: Vec<Rc<RefCell<Upvalue>>>) -> Self {
        Self {
            closure_type: FunctionType::Lua(function),
            upvalues,
        }
    }

    pub const fn new_native(
        function: fn(&mut Lua) -> i32,
        upvalues: Vec<Rc<RefCell<Upvalue>>>,
    ) -> Self {
        Self {
            closure_type: FunctionType::Native(function),
            upvalues,
        }
    }

    pub fn closure_type(&self) -> &FunctionType {
        &self.closure_type
    }

    pub fn program(&self) -> &Program {
        match &self.closure_type {
            FunctionType::Native(_) => {
                unreachable!("It should not be possible to call `program` on a native closure.")
            }
            FunctionType::Lua(function) => function.program(),
        }
    }

    pub fn upvalue(&self, upvalue: usize) -> Result<Rc<RefCell<Upvalue>>, Error> {
        self.upvalues
            .get(upvalue)
            .ok_or(Error::UpvalueDoesNotExist)
            .cloned()
    }

    pub fn constant(&self, constant: usize) -> Result<Value, Error> {
        match &self.closure_type {
            FunctionType::Native(_) => Err(Error::ConstantDoesNotExist(constant, 0)),
            FunctionType::Lua(function) => function
                .program()
                .constants
                .get(constant)
                .ok_or_else(|| {
                    Error::ConstantDoesNotExist(constant, function.program().constants.len())
                })
                .cloned(),
        }
    }

    pub fn function(&self, function_index: usize) -> Result<Rc<Function>, Error> {
        match &self.closure_type {
            FunctionType::Native(_) => Err(Error::FunctionDoesNotExist(function_index, 0)),
            FunctionType::Lua(function) => function
                .program()
                .functions
                .get(function_index)
                .ok_or_else(|| {
                    Error::FunctionDoesNotExist(function_index, function.program().functions.len())
                })
                .cloned(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FunctionType {
    Native(fn(&mut Lua) -> i32),
    Lua(Rc<Function>),
}

#[derive(Debug)]
pub enum Upvalue {
    Open(usize),
    Closed(Value),
}

impl Upvalue {
    pub fn close(&mut self, lua: &Lua) {
        match self {
            Upvalue::Open(stack) => {
                let value = lua.stack[*stack].clone();
                *self = Upvalue::Closed(value);
            }
            Upvalue::Closed(_) => unreachable!("Called `close` on a already closed Upvalue."),
        }
    }
}
