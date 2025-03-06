#![no_std]

mod bytecode;
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

use core::{cell::RefCell, cmp::Ordering};

use alloc::{rc::Rc, vec::Vec};
use stack_frame::FunctionIndex;
use table::Table;
use value::ValueKey;

use self::{bytecode::Bytecode, stack_frame::StackFrame, value::Value};
pub use self::{error::Error, program::Program};

#[derive(Debug, Default)]
pub struct Lua {
    upvalues: Vec<(Value, Value)>,
    stack: Vec<Value>,
    /// Stack frames
    stack_frame: Vec<StackFrame>,
}

impl Lua {
    fn new() -> Self {
        let mut table = Table::new(0, 2);
        table.table.extend([
            (
                ValueKey("print".into()),
                Value::NativeFunction(std::lib_print),
            ),
            (
                ValueKey("type".into()),
                Value::NativeFunction(std::lib_type),
            ),
        ]);

        let upvalues = Vec::from([("_ENV".into(), Value::Table(Rc::new(RefCell::new(table))))]);

        Self {
            upvalues,
            ..Default::default()
        }
    }

    pub fn execute(program: &Program) -> Result<(), Error> {
        Self::new().run_program(program)
    }

    fn run_program(&mut self, main_program: &Program) -> Result<(), Error> {
        log::trace!("Running program");

        self.stack_frame.push(StackFrame {
            function_index: FunctionIndex::Main,
            program_counter: 0,
            stack_frame: 0,
            variadic_arguments: 0,
            out_params: 0,
        });

        while let Some(code) = self.read_bytecode(main_program) {
            code.execute(self, main_program)?;
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

    fn prepare_new_function_stack(
        &mut self,
        func_index: usize,
        args: usize,
        out_params: usize,
        variadic_arguments: usize,
    ) {
        let top_stack = self.get_stack_frame();
        let last_stack = top_stack.stack_frame;
        let last_variadics = top_stack.variadic_arguments;

        let new_stack = StackFrame {
            function_index: FunctionIndex::Closure(func_index),
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

        if let FunctionIndex::Closure(func_index) = popped_stack.function_index {
            let top_stack = self.get_stack_frame();
            self.stack
                .truncate(top_stack.stack_frame + top_stack.variadic_arguments + func_index);
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

    fn get_up_table(&self, uptable: usize) -> Option<&Value> {
        self.upvalues.get(uptable).map(|(_, value)| value)
    }

    fn read_bytecode(&mut self, main_program: &Program) -> Option<Bytecode> {
        if self.stack_frame.is_empty() {
            return None;
        }

        let stack_frame = self.get_stack_frame_mut();
        let pc = &mut stack_frame.program_counter;

        let old = *pc;
        *pc += 1;

        self.get_running_closure(main_program).read_bytecode(old)
    }

    fn get_running_closure<'a>(&'a self, main_program: &'a Program) -> &'a Program {
        let stack_frame = self.get_stack_frame();

        if let FunctionIndex::Closure(_) = stack_frame.function_index {
            match &self.stack[stack_frame.stack_frame - 1] {
                Value::Function(func) => func.program(),
                other => unreachable!(
                    "Value at {} should be a closure, but was {:?}",
                    stack_frame.stack_frame, other
                ),
            }
        } else {
            main_program
        }
    }
}
