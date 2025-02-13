#![no_std]

mod byte_code;
mod closure;
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

use core::cmp::Ordering;

use alloc::vec::Vec;

use self::{byte_code::ByteCode, value::Value};

pub use {closure::Closure, error::Error, program::Program};

#[derive(Debug, Default)]
pub struct Lua {
    func_index: usize,
    program_counter: Vec<usize>,
    globals: Vec<(Value, Value)>,
    stack: Vec<Value>,
    return_stack: Vec<usize>,
}

impl Lua {
    fn new() -> Self {
        let globals = Vec::from([("print".into(), Value::Function(std::lib_print))]);

        Self {
            globals,
            program_counter: Vec::from([0]),
            ..Default::default()
        }
    }

    fn jump(&mut self, jump: isize) -> Result<(), Error> {
        let Some(pc) = self.program_counter.last_mut() else {
            unreachable!("Program counter stack of Vm should never be empty.");
        };
        if let Some(new_pc) = pc.checked_add_signed(jump) {
            *pc = new_pc;
            Ok(())
        } else {
            Err(Error::InvalidJump)
        }
    }

    fn set_stack(&mut self, dst: u8, value: Value) -> Result<(), Error> {
        let offset = self.return_stack.last().copied().unwrap_or(0);
        let dst = offset + usize::from(dst);
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
        let offset = self.get_return_stack();
        let src = offset + usize::from(src);
        Ok(&self.stack[src])
    }

    fn get_stack_mut(&mut self, src: u8) -> Result<&mut Value, Error> {
        let offset = self.get_return_stack();
        let src = offset + usize::from(src);
        Ok(&mut self.stack[src])
    }

    fn push_return_stack(&mut self, function_location: usize) {
        let last = self.return_stack.last().copied().unwrap_or(0);
        self.return_stack.push(last + function_location + 1);
    }

    fn get_return_stack(&self) -> usize {
        let last = self.return_stack.last().copied().unwrap_or(0);
        last
    }

    fn pop_return_stack(&mut self) {
        self.return_stack.pop();
    }

    fn read_bytecode<'a>(&mut self, program: &'a Program) -> Option<&'a ByteCode> {
        let Some(pc) = self.program_counter.last_mut() else {
            unreachable!("Program counter stack of Vm should never be empty.");
        };

        let next_bytecode = program.byte_codes.get(*pc);
        *pc += 1;

        next_bytecode
    }

    pub fn execute(program: &Program) -> Result<(), Error> {
        Self::new().run_program(program)
    }

    pub fn run_program(&mut self, program: &Program) -> Result<(), Error> {
        log::trace!("Running program");
        loop {
            let Some(code) = self.read_bytecode(program) else {
                break;
            };

            match code {
                move_bytecode @ ByteCode::Move(_, _) => {
                    move_bytecode.move_bytecode(self, program)?
                }
                load_int @ ByteCode::LoadInt(_, _) => load_int.load_int(self, program)?,
                load_float @ ByteCode::LoadFloat(_, _) => load_float.load_float(self, program)?,
                load_constant @ ByteCode::LoadConstant(_, _) => {
                    load_constant.load_constant(self, program)?
                }
                load_false @ ByteCode::LoadFalse(_) => load_false.load_false(self, program)?,
                load_false_skip @ ByteCode::LoadFalseSkip(_) => {
                    load_false_skip.load_false_skip(self, program)?
                }
                load_true @ ByteCode::LoadTrue(_) => load_true.load_true(self, program)?,
                load_nil @ ByteCode::LoadNil(_) => load_nil.load_nil(self, program)?,
                get_global @ ByteCode::GetGlobal(_, _) => get_global.get_global(self, program)?,
                set_global @ ByteCode::SetGlobal(_, _) => set_global.set_global(self, program)?,
                get_table @ ByteCode::GetTable(_, _, _) => get_table.get_table(self, program)?,
                get_int @ ByteCode::GetInt(_, _, _) => get_int.get_int(self, program)?,
                get_field @ ByteCode::GetField(_, _, _) => get_field.get_field(self, program)?,
                set_table @ ByteCode::SetTable(_, _, _) => set_table.set_table(self, program)?,
                set_field @ ByteCode::SetField(_, _, _) => set_field.set_field(self, program)?,
                new_table @ ByteCode::NewTable(_, _, _) => new_table.new_table(self, program)?,
                add_integer @ ByteCode::AddInteger(_, _, _) => {
                    add_integer.add_integer(self, program)?
                }
                add_constant @ ByteCode::AddConstant(_, _, _) => {
                    add_constant.add_constant(self, program)?
                }
                mul_constant @ ByteCode::MulConstant(_, _, _) => {
                    mul_constant.mul_constant(self, program)?
                }
                add @ ByteCode::Add(_, _, _) => add.add(self, program)?,
                sub @ ByteCode::Sub(_, _, _) => sub.sub(self, program)?,
                mul @ ByteCode::Mul(_, _, _) => mul.mul(self, program)?,
                mod_bytecode @ ByteCode::Mod(_, _, _) => {
                    mod_bytecode.mod_bytecode(self, program)?
                }
                pow @ ByteCode::Pow(_, _, _) => pow.pow(self, program)?,
                div @ ByteCode::Div(_, _, _) => div.div(self, program)?,
                idiv @ ByteCode::Idiv(_, _, _) => idiv.idiv(self, program)?,
                bit_and @ ByteCode::BitAnd(_, _, _) => bit_and.bit_and(self, program)?,
                bit_or @ ByteCode::BitOr(_, _, _) => bit_or.bit_or(self, program)?,
                bit_xor @ ByteCode::BitXor(_, _, _) => bit_xor.bit_xor(self, program)?,
                shiftl @ ByteCode::ShiftL(_, _, _) => shiftl.shiftl(self, program)?,
                shiftr @ ByteCode::ShiftR(_, _, _) => shiftr.shiftr(self, program)?,
                neg @ ByteCode::Neg(_, _) => neg.neg(self, program)?,
                bit_not @ ByteCode::BitNot(_, _) => bit_not.bit_not(self, program)?,
                not @ ByteCode::Not(_, _) => not.not(self, program)?,
                len @ ByteCode::Len(_, _) => len.len(self, program)?,
                concat @ ByteCode::Concat(_, _, _) => concat.concat(self, program)?,
                jmp @ ByteCode::Jmp(_) => jmp.jmp(self, program)?,
                lt @ ByteCode::LessThan(_, _, _) => lt.less_than(self, program)?,
                le @ ByteCode::LessEqual(_, _, _) => le.less_equal(self, program)?,
                eqk @ ByteCode::EqualConstant(_, _, _) => eqk.equal_constant(self, program)?,
                gti @ ByteCode::GreaterThanInteger(_, _, _) => {
                    gti.greater_than_integer(self, program)?
                }
                gei @ ByteCode::GreaterEqualInteger(_, _, _) => {
                    gei.greater_equal_integer(self, program)?
                }
                test @ ByteCode::Test(_, _) => test.test(self, program)?,
                call @ ByteCode::Call(_, _) => call.call(self, program)?,
                ByteCode::Return => unimplemented!("return bytecode"),
                zero_return @ ByteCode::ZeroReturn => {
                    zero_return.zero_return(self, program)?;
                    break;
                }
                one_return @ ByteCode::OneReturn(_) => {
                    one_return.one_return(self, program)?;
                    break;
                }
                forloop @ ByteCode::ForLoop(_, _) => forloop.for_loop(self, program)?,
                forprep @ ByteCode::ForPrepare(_, _) => forprep.for_prepare(self, program)?,
                set_list @ ByteCode::SetList(_, _) => set_list.set_list(self, program)?,
                set_global_constant @ ByteCode::SetGlobalConstant(_, _) => {
                    set_global_constant.set_global_constant(self, program)?
                }
                set_global_integer @ ByteCode::SetGlobalInteger(_, _) => {
                    set_global_integer.set_global_integer(self, program)?
                }
                set_global_global @ ByteCode::SetGlobalGlobal(_, _) => {
                    set_global_global.set_global_global(self, program)?
                }
                closure @ ByteCode::Closure(_, _) => closure.closure(self, program)?,
            }
        }

        Ok(())
    }
}
