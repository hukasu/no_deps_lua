use core::{cell::RefCell, fmt::Display};

use alloc::{rc::Rc, vec::Vec};

use crate::{Error, Lua, Program, function::Function, value::Value};

pub type NativeClosure = fn(&mut Lua) -> NativeClosureReturn;
pub type NativeClosureReturn = Result<usize, Error>;

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

    pub const fn new_native(function: NativeClosure, upvalues: Vec<Rc<RefCell<Upvalue>>>) -> Self {
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

    pub fn function(&self, function_index: usize) -> Result<Rc<Function>, ClosureFunctionError> {
        match &self.closure_type {
            FunctionType::Native(_) => Err(ClosureFunctionError {
                func_index: function_index,
                native: true,
                function_count: 0,
            }),
            FunctionType::Lua(function) => function
                .program()
                .functions
                .get(function_index)
                .ok_or_else(|| ClosureFunctionError {
                    func_index: function_index,
                    native: false,
                    function_count: function.program().functions.len(),
                })
                .cloned(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FunctionType {
    Native(NativeClosure),
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

#[derive(Debug, PartialEq, Eq)]
pub struct ClosureFunctionError {
    func_index: usize,
    native: bool,
    function_count: usize,
}

impl Display for ClosureFunctionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.native {
            write!(f, "Can't fetch a function from a native closure.")
        } else {
            write!(
                f,
                "Can't fetch function at position {}, there are only {} functions in closure's program.",
                self.func_index, self.function_count
            )
        }
    }
}

impl core::error::Error for ClosureFunctionError {}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::std;

    use super::*;

    #[test]
    fn closure_function() {
        let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
        let native = Closure::new_native(std::lib_print, vec![]);
        assert_eq!(
            native.function(0).unwrap_err(),
            ClosureFunctionError {
                func_index: 0,
                native: true,
                function_count: 0
            }
        );
        assert_eq!(
            native.function(1).unwrap_err(),
            ClosureFunctionError {
                func_index: 1,
                native: true,
                function_count: 0
            }
        );
        assert_eq!(
            native.function(1000).unwrap_err(),
            ClosureFunctionError {
                func_index: 1000,
                native: true,
                function_count: 0
            }
        );

        let program = Program::parse("function a() end").unwrap();
        let lua = Closure::new_lua(Rc::new(Function::new(program, 0, false)), vec![]);
        assert!(lua.function(0).is_ok());
        assert_eq!(
            lua.function(1).unwrap_err(),
            ClosureFunctionError {
                func_index: 1,
                native: false,
                function_count: 1
            }
        );
        assert_eq!(
            lua.function(1000).unwrap_err(),
            ClosureFunctionError {
                func_index: 1000,
                native: false,
                function_count: 1
            }
        );
    }
}
