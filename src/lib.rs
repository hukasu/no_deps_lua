#![no_std]

mod byte_code;
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

pub use {error::Error, program::Program};

#[derive(Debug)]
pub struct Lua {
    func_index: usize,
    program_counter: usize,
    globals: Vec<(Value, Value)>,
    stack: Vec<Value>,
}

impl Lua {
    fn new() -> Self {
        let globals = Vec::from([("print".into(), Value::Function(std::lib_print))]);

        Self {
            func_index: 0,
            program_counter: 0,
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

    fn read_bytecode<'a>(&mut self, program: &'a Program) -> Option<&'a ByteCode> {
        program
            .byte_codes
            .get(self.program_counter)
            .inspect(|_| self.program_counter += 1)
    }

    pub fn execute(program: &Program) -> Result<(), Error> {
        let mut vm = Self::new();

        loop {
            let Some(code) = vm.read_bytecode(program) else {
                break;
            };

            match code {
                move_bytecode @ ByteCode::Move(_, _) => {
                    move_bytecode.move_bytecode(&mut vm, program)?
                }
                load_int @ ByteCode::LoadInt(_, _) => load_int.load_int(&mut vm, program)?,
                load_float @ ByteCode::LoadFloat(_, _) => {
                    load_float.load_float(&mut vm, program)?
                }
                load_constant @ ByteCode::LoadConstant(_, _) => {
                    load_constant.load_constant(&mut vm, program)?
                }
                load_false @ ByteCode::LoadFalse(_) => load_false.load_false(&mut vm, program)?,
                load_true @ ByteCode::LoadTrue(_) => load_true.load_true(&mut vm, program)?,
                load_nil @ ByteCode::LoadNil(_) => load_nil.load_nil(&mut vm, program)?,
                get_global @ ByteCode::GetGlobal(_, _) => {
                    get_global.get_global(&mut vm, program)?
                }
                set_global @ ByteCode::SetGlobal(_, _) => {
                    set_global.set_global(&mut vm, program)?
                }
                get_table @ ByteCode::GetTable(_, _, _) => get_table.get_table(&mut vm, program)?,
                get_int @ ByteCode::GetInt(_, _, _) => get_int.get_int(&mut vm, program)?,
                get_field @ ByteCode::GetField(_, _, _) => get_field.get_field(&mut vm, program)?,
                set_table @ ByteCode::SetTable(_, _, _) => set_table.set_table(&mut vm, program)?,
                set_field @ ByteCode::SetField(_, _, _) => set_field.set_field(&mut vm, program)?,
                new_table @ ByteCode::NewTable(_, _, _) => new_table.new_table(&mut vm, program)?,
                add_integer @ ByteCode::AddInteger(_, _, _) => {
                    add_integer.add_integer(&mut vm, program)?
                }
                add_constant @ ByteCode::AddConstant(_, _, _) => {
                    add_constant.add_constant(&mut vm, program)?
                }
                mul_constant @ ByteCode::MulConstant(_, _, _) => {
                    mul_constant.mul_constant(&mut vm, program)?
                }
                add @ ByteCode::Add(_, _, _) => add.add(&mut vm, program)?,
                sub @ ByteCode::Sub(_, _, _) => sub.sub(&mut vm, program)?,
                mul @ ByteCode::Mul(_, _, _) => mul.mul(&mut vm, program)?,
                mod_bytecode @ ByteCode::Mod(_, _, _) => {
                    mod_bytecode.mod_bytecode(&mut vm, program)?
                }
                pow @ ByteCode::Pow(_, _, _) => pow.pow(&mut vm, program)?,
                div @ ByteCode::Div(_, _, _) => div.div(&mut vm, program)?,
                idiv @ ByteCode::Idiv(_, _, _) => idiv.idiv(&mut vm, program)?,
                bit_and @ ByteCode::BitAnd(_, _, _) => bit_and.bit_and(&mut vm, program)?,
                bit_or @ ByteCode::BitOr(_, _, _) => bit_or.bit_or(&mut vm, program)?,
                bit_xor @ ByteCode::BitXor(_, _, _) => bit_xor.bit_xor(&mut vm, program)?,
                shiftl @ ByteCode::ShiftL(_, _, _) => shiftl.shiftl(&mut vm, program)?,
                shiftr @ ByteCode::ShiftR(_, _, _) => shiftr.shiftr(&mut vm, program)?,
                neg @ ByteCode::Neg(_, _) => neg.neg(&mut vm, program)?,
                bit_not @ ByteCode::BitNot(_, _) => bit_not.bit_not(&mut vm, program)?,
                not @ ByteCode::Not(_, _) => not.not(&mut vm, program)?,
                len @ ByteCode::Len(_, _) => len.len(&mut vm, program)?,
                concat @ ByteCode::Concat(_, _, _) => concat.concat(&mut vm, program)?,
                jmp @ ByteCode::Jmp(_) => jmp.jmp(&mut vm, program)?,
                test @ ByteCode::Test(_, _) => test.test(&mut vm, program)?,
                call @ ByteCode::Call(_, _) => call.call(&mut vm, program)?,
                forloop @ ByteCode::ForLoop(_, _) => forloop.for_loop(&mut vm, program)?,
                forprep @ ByteCode::ForPrepare(_, _) => forprep.for_prepare(&mut vm, program)?,
                set_list @ ByteCode::SetList(_, _) => set_list.set_list(&mut vm, program)?,
                set_global_constant @ ByteCode::SetGlobalConstant(_, _) => {
                    set_global_constant.set_global_constant(&mut vm, program)?
                }
                set_global_integer @ ByteCode::SetGlobalInteger(_, _) => {
                    set_global_integer.set_global_integer(&mut vm, program)?
                }
                set_global_global @ ByteCode::SetGlobalGlobal(_, _) => {
                    set_global_global.set_global_global(&mut vm, program)?
                }
            }
        }

        Ok(())
    }
}
