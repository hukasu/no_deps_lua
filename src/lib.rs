#![no_std]

mod bytecode;
mod closure;
mod error;
mod ext;
mod function;
mod lex;
mod parser;
mod program;
mod stack_frame;
mod stack_str;
mod std;
mod table;
mod value;

extern crate alloc;

use alloc::{rc::Rc, vec::Vec};
use core::{
    cell::RefCell,
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use self::{
    bytecode::Bytecode,
    closure::{Closure, Upvalue},
    function::Function,
    stack_frame::StackFrame,
    table::Table,
    value::{Value, ValueKey},
};
pub use self::{error::Error, program::Program};

#[derive(Debug, Default)]
pub struct Lua {
    stack: Vec<Value>,
    /// Stack frames
    stack_frame: Vec<StackFrame>,
}

impl Lua {
    /// Runs program with default environment
    pub fn run_program(main_program: Program) -> Result<(), Error> {
        let mut table = Table::new(0, 2);

        table.table.extend([(
            ValueKey("print".into()),
            Value::from(std::lib_print as fn(&mut Lua) -> i32),
        )]);

        Self::run_program_with_env(main_program, Rc::new(RefCell::new(table)))
    }

    /// Runs program with given environment
    pub fn run_program_with_env(
        main_program: Program,
        env: Rc<RefCell<Table>>,
    ) -> Result<(), Error> {
        log::trace!("Running program");

        let mut vm = Lua::default();

        vm.stack.push(Value::Closure(Rc::new(Closure::new_lua(
            Rc::new(Function::new(main_program, 0, true)),
            Vec::from_iter([Rc::new(RefCell::new(Upvalue::Closed(Value::Table(env))))]),
        ))));
        vm.prepare_new_stack_frame(0, 0, 0, 0);

        while let Some(code) = vm.read_bytecode() {
            code.execute(&mut vm)?;
        }

        Ok(())
    }

    fn jump(&mut self, jump: isize) -> Result<(), Error> {
        let top_stack = self.get_stack_frame_mut();

        let pc = &mut top_stack.program_counter;
        if let Some(new_pc) = pc.checked_add_signed(jump) {
            *pc = new_pc;
            Ok(())
        } else {
            Err(Error::InvalidJump)
        }
    }

    fn prepare_new_stack_frame(
        &mut self,
        func_index: usize,
        args: usize,
        out_params: usize,
        variadic_arguments: usize,
    ) {
        let (last_stack, last_variadics) = if !self.stack_frame.is_empty() {
            let top_stack = self.get_stack_frame();
            let last_stack = top_stack.stack_frame;
            let last_variadics = top_stack.variadic_arguments;
            (last_stack, last_variadics)
        } else {
            (0, 0)
        };

        let new_stack = StackFrame {
            function_index: func_index,
            program_counter: 0,
            stack_frame: last_stack + last_variadics + func_index + 1,
            variadic_arguments,
            out_params,
        };

        self.stack.resize(
            new_stack.stack_frame + args + variadic_arguments,
            Value::Nil,
        );

        self.stack_frame.push(new_stack);
    }

    fn drop_stack_frame(&mut self, return_start: usize, returns: usize) {
        let popped_stack = self.pop_stack_frame();

        let start = popped_stack.stack_frame + return_start;

        let returns = self
            .stack
            .drain(start..(start + returns))
            .collect::<Vec<_>>();

        if !self.stack_frame.is_empty() {
            let top_stack = self.get_stack_frame();
            self.stack.truncate(
                top_stack.stack_frame + top_stack.variadic_arguments + popped_stack.function_index,
            );
        } else {
            self.stack.clear();
        }
        self.stack.extend(returns);
    }

    fn set_stack(&mut self, dst: u8, value: Value) -> Result<(), Error> {
        let stack_frame = self.get_stack_frame();

        let offset = stack_frame.stack_frame;
        let variadics = stack_frame.variadic_arguments;
        let dst = offset + variadics + usize::from(dst);
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
                log::error!(
                    "Trying to set a value out of the bounds of the stack. {} {}",
                    dst,
                    self.stack.len()
                );
                Err(Error::StackOverflow)
            }
        }
    }

    fn get_stack(&self, src: u8) -> Result<&Value, Error> {
        let stack_frame = self.get_stack_frame();

        let offset = stack_frame.stack_frame;
        let variadics = stack_frame.variadic_arguments;
        let src = offset + variadics + usize::from(src);
        Ok(&self.stack[src])
    }

    fn get_stack_mut(&mut self, src: u8) -> Result<&mut Value, Error> {
        let stack_frame = self.get_stack_frame();

        let offset = stack_frame.stack_frame;
        let variadics = stack_frame.variadic_arguments;
        let src = offset + variadics + usize::from(src);
        Ok(&mut self.stack[src])
    }

    fn get_stack_frame(&self) -> &StackFrame {
        let Some(last) = self.stack_frame.last() else {
            unreachable!("Stack frames should never be empty.");
        };
        last
    }

    fn get_stack_frame_mut(&mut self) -> &mut StackFrame {
        let Some(last) = self.stack_frame.last_mut() else {
            unreachable!("Stack frames should never be empty.");
        };
        last
    }

    fn pop_stack_frame(&mut self) -> StackFrame {
        let Some(last) = self.stack_frame.pop() else {
            unreachable!("Stack frames should never be empty.");
        };
        last
    }

    fn get_upvalue(&self, upvalue: usize) -> Result<Value, Error> {
        let closure = self.get_running_closure();
        let upvalue = closure.upvalue(upvalue)?;
        let upvalue_borrow = upvalue.as_ref().borrow();
        match upvalue_borrow.deref() {
            Upvalue::Open(register) => self.get_stack(u8::try_from(*register)?).cloned(),
            Upvalue::Closed(value) => Ok(value).cloned(),
        }
    }

    fn set_upvalue(&mut self, upvalue: usize, value: impl Into<Value>) -> Result<(), Error> {
        let closure = self.get_running_closure();
        let upvalue = closure.upvalue(upvalue)?;
        let value = value.into();

        match upvalue.as_ref().borrow_mut().deref_mut() {
            Upvalue::Open(dst) => {
                self.set_stack(u8::try_from(*dst)?, value)?;
            }
            Upvalue::Closed(upvalue) => {
                *upvalue = value;
            }
        }
        Ok(())
    }

    fn read_bytecode(&mut self) -> Option<Bytecode> {
        if self.stack_frame.is_empty() {
            return None;
        }

        let stack_frame = self.get_stack_frame_mut();
        let pc = &mut stack_frame.program_counter;

        let old = *pc;
        *pc += 1;

        let closure = self.get_running_closure();

        let program = closure.program();
        program.read_bytecode(old)
    }

    fn get_running_closure(&self) -> &Closure {
        self.get_running_closure_of_stack_frame(self.get_stack_frame())
    }

    fn get_running_closure_of_stack_frame(&self, stack_frame: &StackFrame) -> &Closure {
        let func_index = stack_frame.stack_frame - 1;

        match &self.stack[func_index] {
            Value::Closure(closure) => closure,
            other => unreachable!(
                "Value at {} should be a closure, but was {:?}",
                func_index, other
            ),
        }
    }

    fn find_upvalue(&self, upvalue: &str) -> Result<Rc<RefCell<Upvalue>>, Error> {
        // for now only global environment
        let Value::Closure(main_program) = &self.stack[0] else {
            unreachable!("Main program was not a closure");
        };
        assert_eq!(upvalue, "_ENV");

        main_program.upvalue(0)
    }
}
