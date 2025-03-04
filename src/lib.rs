#![no_std]

mod byte_code;
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

use self::{byte_code::ByteCode, stack_frame::StackFrame, value::Value};
pub use self::{error::Error, function::Function, program::Program};

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
            match code {
                move_bytecode @ ByteCode::Move(_, _) => {
                    move_bytecode.move_bytecode(self, main_program)?
                }
                load_int @ ByteCode::LoadInt(_, _) => load_int.load_int(self, main_program)?,
                load_float @ ByteCode::LoadFloat(_, _) => {
                    load_float.load_float(self, main_program)?
                }
                load_constant @ ByteCode::LoadConstant(_, _) => {
                    load_constant.load_constant(self, main_program)?
                }
                load_false @ ByteCode::LoadFalse(_) => load_false.load_false(self, main_program)?,
                load_false_skip @ ByteCode::LoadFalseSkip(_) => {
                    load_false_skip.load_false_skip(self, main_program)?
                }
                load_true @ ByteCode::LoadTrue(_) => load_true.load_true(self, main_program)?,
                load_nil @ ByteCode::LoadNil(_, _) => load_nil.load_nil(self, main_program)?,
                get_up_value @ ByteCode::GetUpValue(_, _) => {
                    get_up_value.get_up_value(self, main_program)?
                }
                get_up_table @ ByteCode::GetUpTable(_, _, _) => {
                    get_up_table.get_up_table(self, main_program)?
                }
                get_table @ ByteCode::GetTable(_, _, _) => {
                    get_table.get_table(self, main_program)?
                }
                get_int @ ByteCode::GetInt(_, _, _) => get_int.get_int(self, main_program)?,
                get_field @ ByteCode::GetField(_, _, _) => {
                    get_field.get_field(self, main_program)?
                }
                set_up_table @ ByteCode::SetUpTable(_, _, _) => {
                    set_up_table.set_up_table(self, main_program)?
                }
                set_up_table_constant @ ByteCode::SetUpTableConstant(_, _, _) => {
                    set_up_table_constant.set_up_table_constant(self, main_program)?
                }
                set_table @ ByteCode::SetTable(_, _, _) => {
                    set_table.set_table(self, main_program)?
                }
                set_table_constant @ ByteCode::SetTableConstant(_, _, _) => {
                    set_table_constant.set_table_constant(self, main_program)?
                }
                set_field @ ByteCode::SetField(_, _, _) => {
                    set_field.set_field(self, main_program)?
                }
                set_field_constant @ ByteCode::SetFieldConstant(_, _, _) => {
                    set_field_constant.set_field_constant(self, main_program)?
                }
                new_table @ ByteCode::NewTable(_, _, _) => {
                    new_table.new_table(self, main_program)?
                }
                table_self @ ByteCode::TableSelf(_, _, _) => {
                    table_self.table_self(self, main_program)?
                }
                add_integer @ ByteCode::AddInteger(_, _, _) => {
                    add_integer.add_integer(self, main_program)?
                }
                add_constant @ ByteCode::AddConstant(_, _, _) => {
                    add_constant.add_constant(self, main_program)?
                }
                mul_constant @ ByteCode::MulConstant(_, _, _) => {
                    mul_constant.mul_constant(self, main_program)?
                }
                add @ ByteCode::Add(_, _, _) => add.add(self, main_program)?,
                sub @ ByteCode::Sub(_, _, _) => sub.sub(self, main_program)?,
                mul @ ByteCode::Mul(_, _, _) => mul.mul(self, main_program)?,
                mod_bytecode @ ByteCode::Mod(_, _, _) => {
                    mod_bytecode.mod_bytecode(self, main_program)?
                }
                pow @ ByteCode::Pow(_, _, _) => pow.pow(self, main_program)?,
                div @ ByteCode::Div(_, _, _) => div.div(self, main_program)?,
                idiv @ ByteCode::Idiv(_, _, _) => idiv.idiv(self, main_program)?,
                bit_and @ ByteCode::BitAnd(_, _, _) => bit_and.bit_and(self, main_program)?,
                bit_or @ ByteCode::BitOr(_, _, _) => bit_or.bit_or(self, main_program)?,
                bit_xor @ ByteCode::BitXor(_, _, _) => bit_xor.bit_xor(self, main_program)?,
                shiftl @ ByteCode::ShiftLeft(_, _, _) => shiftl.shiftl(self, main_program)?,
                shiftr @ ByteCode::ShiftRight(_, _, _) => shiftr.shiftr(self, main_program)?,
                neg @ ByteCode::Neg(_, _) => neg.neg(self, main_program)?,
                bit_not @ ByteCode::BitNot(_, _) => bit_not.bit_not(self, main_program)?,
                not @ ByteCode::Not(_, _) => not.not(self, main_program)?,
                len @ ByteCode::Len(_, _) => len.len(self, main_program)?,
                concat @ ByteCode::Concat(_, _) => concat.concat(self, main_program)?,
                jmp @ ByteCode::Jmp(_) => jmp.jmp(self, main_program)?,
                lt @ ByteCode::LessThan(_, _, _) => lt.less_than(self, main_program)?,
                le @ ByteCode::LessEqual(_, _, _) => le.less_equal(self, main_program)?,
                eqk @ ByteCode::EqualConstant(_, _, _) => eqk.equal_constant(self, main_program)?,
                eqi @ ByteCode::EqualInteger(_, _, _) => eqi.equal_integer(self, main_program)?,
                gti @ ByteCode::GreaterThanInteger(_, _, _) => {
                    gti.greater_than_integer(self, main_program)?
                }
                gei @ ByteCode::GreaterEqualInteger(_, _, _) => {
                    gei.greater_equal_integer(self, main_program)?
                }
                test @ ByteCode::Test(_, _) => test.test(self, main_program)?,
                call @ ByteCode::Call(_, _, _) => call.call(self, main_program)?,
                tail_call @ ByteCode::TailCall(_, _, _) => {
                    tail_call.tail_call(self, main_program)?
                }
                return_bytecode @ ByteCode::Return(_, _, _) => {
                    return_bytecode.return_bytecode(self, main_program)?
                }
                zero_return @ ByteCode::ZeroReturn => {
                    zero_return.zero_return(self, main_program)?
                }
                one_return @ ByteCode::OneReturn(_) => one_return.one_return(self, main_program)?,
                forloop @ ByteCode::ForLoop(_, _) => forloop.for_loop(self, main_program)?,
                forprep @ ByteCode::ForPrepare(_, _) => forprep.for_prepare(self, main_program)?,
                set_list @ ByteCode::SetList(_, _, _) => set_list.set_list(self, main_program)?,
                closure @ ByteCode::Closure(_, _) => closure.closure(self, main_program)?,
                variadic_args @ ByteCode::VariadicArguments(_, _) => {
                    variadic_args.variadic_arguments(self, main_program)?
                }
                ByteCode::VariadicArgumentPrepare(_) => {
                    // Nothing is done here as this functionality is already dealt with
                }
            }
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

    fn read_bytecode(&mut self, main_program: &Program) -> Option<ByteCode> {
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
