use alloc::boxed::Box;

use crate::ext::{FloatExt, Unescape};

use super::{
    binops::Binop,
    compile_context::CompileContext,
    helper_types::{ExpList, TableFields, TableKey},
    ByteCode, Error, Program,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExpDesc<'a> {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    Name(&'a str),
    Unop(fn(u8, u8) -> ByteCode, Box<ExpDesc<'a>>),
    Binop(Binop, Box<ExpDesc<'a>>, Box<ExpDesc<'a>>),
    Local(usize),
    Global(usize),
    Table(TableFields<'a>),
    /// Access to a table
    ///
    /// `table`: The table being accessed  
    /// `key`: Key into table  
    /// `record`: Whether it is a record access or general access
    TableAccess {
        table: Box<ExpDesc<'a>>,
        key: Box<ExpDesc<'a>>,
        record: bool,
    },
    Condition(bool),
    Closure(u8),
    FunctionCall(Box<ExpDesc<'a>>, ExpList<'a>),
    VariadicArguments,
}

impl<'a> ExpDesc<'a> {
    pub fn discharge(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match self {
            ExpDesc::Nil => self.discharge_nil(dst, program),
            ExpDesc::Boolean(_) => self.discharge_boolean(dst, program),
            ExpDesc::Integer(_) => self.discharge_integer(dst, program, compile_context),
            ExpDesc::Float(_) => self.discharge_float(dst, program),
            ExpDesc::String(_) => self.discharge_string(dst, program, compile_context),
            ExpDesc::Name(_) => self.discharge_name(dst, program, compile_context),
            ExpDesc::Unop(_, _) => self.discharge_unop(dst, program, compile_context),
            ExpDesc::Binop(_, _, _) => self.discharge_binop(dst, program, compile_context),
            ExpDesc::Local(_) => self.discharge_local(dst, program, compile_context),
            ExpDesc::Global(_) => self.discharge_global(dst, program, compile_context),
            ExpDesc::Table(_) => self.discharge_table(dst, program, compile_context),
            ExpDesc::TableAccess {
                table: _,
                key: _,
                record: _,
            } => self.discharge_table_access(dst, program, compile_context),
            ExpDesc::Closure(_) => self.discharge_closure(dst, program),
            ExpDesc::FunctionCall(_, _) => {
                self.discharge_function_call(dst, program, compile_context)
            }
            ExpDesc::VariadicArguments => {
                self.discharge_variable_arguments(dst, program, compile_context)
            }
            _ => {
                unimplemented!(
                    "Unimplemented discharge between Src({:?}) and Dst({:?})",
                    self,
                    dst
                );
            }
        }
    }

    fn discharge_nil(&self, dst: &ExpDesc<'a>, program: &mut Program) -> Result<(), Error> {
        let ExpDesc::Nil = &self else {
            unreachable!("Should never be anything other than Nil");
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;
                program.byte_codes.push(ByteCode::LoadNil(dst));
            }
            Self::Global(key) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(())?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));
            }
            other => unreachable!("Nil can't be discharged in {:?}.", other),
        }

        Ok(())
    }

    fn discharge_boolean(&self, dst: &ExpDesc<'a>, program: &mut Program) -> Result<(), Error> {
        let ExpDesc::Boolean(boolean) = &self else {
            unreachable!("Should never be anything other than Boolean");
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                if *boolean {
                    program.byte_codes.push(ByteCode::LoadTrue(dst));
                } else {
                    program.byte_codes.push(ByteCode::LoadFalse(dst));
                }
            }
            Self::Global(key) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(*boolean)?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));
            }
            other => unreachable!("Boolean can't be discharged in {:?}.", other),
        }

        Ok(())
    }

    fn discharge_integer(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Integer(integer) = &self else {
            unreachable!("Should never be anything other than Integer");
        };

        match dst {
            Self::Name(name) => {
                let name = match compile_context.find_name(name) {
                    Some(local) => local,
                    None => ExpDesc::Global(usize::from(program.push_constant(*name)?)),
                };

                self.discharge(&name, program, compile_context)
            }
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                let code = if let Ok(i) = i16::try_from(*integer) {
                    ByteCode::LoadInt(dst, i)
                } else {
                    let position = program.push_constant(*integer)?;
                    ByteCode::LoadConstant(dst, position)
                };

                program.byte_codes.push(code);

                Ok(())
            }
            Self::Global(key) => {
                let key = u8::try_from(*key)?;

                let code = if let Ok(i) = i16::try_from(*integer) {
                    ByteCode::SetGlobalInteger(key, i)
                } else {
                    let constant = program.push_constant(*integer)?;
                    ByteCode::SetGlobalConstant(key, constant)
                };

                program.byte_codes.push(code);

                Ok(())
            }
            Self::TableAccess {
                table,
                key,
                record: true,
            } => {
                let (table_loc, stack_top) = compile_context.reserve_stack_top();
                table.discharge(&stack_top, program, compile_context)?;

                let (integer_loc, stack_top) = compile_context.reserve_stack_top();
                self.discharge(&stack_top, program, compile_context)?;

                compile_context.stack_top -= 2;

                match key.as_ref() {
                    ExpDesc::Name(name) => {
                        let constant = program.push_constant(*name)?;
                        program.byte_codes.push(ByteCode::SetField(
                            table_loc,
                            constant,
                            integer_loc,
                        ));

                        Ok(())
                    }
                    other => Err(Error::TableRecordAccess(other.static_type_name())),
                }
            }
            other => unreachable!("Integer can't be discharged in {:?}.", other),
        }
    }

    fn discharge_float(&self, dst: &ExpDesc<'a>, program: &mut Program) -> Result<(), Error> {
        let ExpDesc::Float(float) = &self else {
            unreachable!("Should never be anything other than Float");
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                match float.to_i16() {
                    Some(i) => {
                        program.byte_codes.push(ByteCode::LoadFloat(dst, i));
                        Ok(())
                    }
                    _ => {
                        let position = program.push_constant(*float)?;
                        program
                            .byte_codes
                            .push(ByteCode::LoadConstant(dst, position));
                        Ok(())
                    }
                }
            }
            Self::Global(key) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(*float)?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

                Ok(())
            }
            other => unreachable!("Float can't be discharged in {:?}.", other),
        }
    }

    fn discharge_string(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::String(string) = &self else {
            unreachable!("Should never be anything other than String");
        };

        match dst {
            Self::Name(name) => {
                let dst = match compile_context.find_name(name) {
                    Some(local) => local,
                    None => ExpDesc::Global(usize::from(program.push_constant(*name)?)),
                };

                self.discharge(&dst, program, compile_context)
            }
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;
                let string = string.unescape()?;
                let position = program.push_constant(string.as_str())?;

                program
                    .byte_codes
                    .push(ByteCode::LoadConstant(dst, position));

                Ok(())
            }
            Self::Global(key) => {
                let key = u8::try_from(*key)?;
                let string = string.unescape()?;
                let constant = program.push_constant(string.as_str())?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

                Ok(())
            }
            other => unreachable!("String can't be discharged in {:?}.", other),
        }
    }

    fn discharge_name(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Name(name) = &self else {
            unreachable!("Should never be anything other than Name");
        };

        match dst {
            Self::Name(lhs) => {
                if name != lhs {
                    let rhs = match compile_context.find_name(name) {
                        Some(local) => local,
                        None => ExpDesc::Global(usize::from(program.push_constant(*name)?)),
                    };
                    let lhs = match compile_context.find_name(lhs) {
                        Some(local) => local,
                        None => ExpDesc::Global(usize::from(program.push_constant(*lhs)?)),
                    };
                    rhs.discharge(&lhs, program, compile_context)
                } else {
                    Ok(())
                }
            }
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                match compile_context.find_name(name) {
                    Some(ExpDesc::Local(pos)) => {
                        program
                            .byte_codes
                            .push(ByteCode::Move(dst, u8::try_from(pos)?));
                    }
                    Some(_) => unreachable!("Local will always be a `Local`."),
                    None => {
                        let constant = program.push_constant(*name)?;
                        program.byte_codes.push(ByteCode::GetGlobal(dst, constant));
                    }
                }

                Ok(())
            }
            table @ Self::TableAccess {
                table: _,
                key: _,
                record: _,
            } => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                self.discharge(&stack_top, program, compile_context)?;

                stack_top.discharge(table, program, compile_context)?;

                compile_context.stack_top -= 1;

                Ok(())
            }
            cond @ Self::Condition(_) => {
                let (name_loc, stack_top) = compile_context.reserve_stack_top();
                let var_loc =
                    self.get_local_or_discharge_at_location(program, name_loc, compile_context)?;

                if var_loc == name_loc {
                    stack_top.discharge(cond, program, compile_context)?;
                    compile_context.stack_top -= 1;
                } else {
                    compile_context.stack_top -= 1;
                    ExpDesc::Local(usize::from(var_loc)).discharge(
                        cond,
                        program,
                        compile_context,
                    )?;
                }

                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_end.push(jump);

                Ok(())
            }
            other => unreachable!("Name can't be discharged in {:?}.", other),
        }
    }

    fn discharge_unop(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Unop(unop, exp) = &self else {
            unreachable!("Should never be anything other than Unop");
        };

        match dst {
            name @ ExpDesc::Name(_) => {
                let (name_loc, stack_top) = compile_context.reserve_stack_top();
                let local =
                    name.get_local_or_discharge_at_location(program, name_loc, compile_context)?;

                if name_loc == local {
                    self.discharge(&stack_top, program, compile_context)?;
                    compile_context.stack_top -= 1;
                } else {
                    compile_context.stack_top -= 1;
                    self.discharge(
                        &ExpDesc::Local(usize::from(local)),
                        program,
                        compile_context,
                    )?;
                }
                Ok(())
            }
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                let src_loc = match exp.as_ref() {
                    name @ ExpDesc::Name(_) => {
                        name.get_local_or_discharge_at_location(program, dst, compile_context)?
                    }
                    ExpDesc::Local(local) => u8::try_from(*local)?,
                    other => {
                        let (src_loc, src_expdesc) = compile_context.reserve_stack_top();
                        other.discharge(&src_expdesc, program, compile_context)?;
                        compile_context.stack_top -= 1;
                        src_loc
                    }
                };

                program.byte_codes.push(unop(dst, src_loc));

                Ok(())
            }
            other => unreachable!("Unop can't be discharged in {:?}.", other),
        }
    }

    fn discharge_binop(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Binop(_, _, _) = &self else {
            unreachable!("Should never be anything other than Binop");
        };

        fn discharge_binop(
            bytecode: fn(u8, u8, u8) -> ByteCode,
            lhs: &ExpDesc,
            rhs: &ExpDesc,
            dst: u8,
            program: &mut Program,
            compile_context: &mut CompileContext,
        ) -> Result<(), Error> {
            let (lhs_loc, used_dst) = if let name @ ExpDesc::Name(_) = lhs {
                let local =
                    name.get_local_or_discharge_at_location(program, dst, compile_context)?;
                (local, local == dst)
            } else {
                lhs.discharge(&ExpDesc::Local(usize::from(dst)), program, compile_context)?;
                (dst, true)
            };

            let (rhs_loc, stack_top) = if used_dst {
                compile_context.reserve_stack_top()
            } else {
                (dst, ExpDesc::Local(usize::from(dst)))
            };

            let rhs_loc = if let name @ ExpDesc::Name(_) = rhs {
                name.get_local_or_discharge_at_location(program, rhs_loc, compile_context)?
            } else {
                rhs.discharge(&stack_top, program, compile_context)?;
                rhs_loc
            };

            if used_dst {
                compile_context.stack_top -= 1;
            }

            program.byte_codes.push(bytecode(dst, lhs_loc, rhs_loc));

            Ok(())
        }

        fn discharge_binop_integer(
            bytecode: fn(u8, u8, i8) -> ByteCode,
            lhs: &ExpDesc,
            integer: i8,
            dst: u8,
            program: &mut Program,
            compile_context: &mut CompileContext,
        ) -> Result<(), Error> {
            let lhs_loc = if let name @ ExpDesc::Name(_) = lhs {
                name.get_local_or_discharge_at_location(program, dst, compile_context)?
            } else {
                lhs.discharge(&ExpDesc::Local(usize::from(dst)), program, compile_context)?;
                dst
            };

            program.byte_codes.push(bytecode(dst, lhs_loc, integer));

            Ok(())
        }

        fn discharge_binop_constant(
            bytecode: fn(u8, u8, u8) -> ByteCode,
            lhs: &ExpDesc,
            constant: u8,
            dst: u8,
            program: &mut Program,
            compile_context: &mut CompileContext,
        ) -> Result<(), Error> {
            let lhs_loc = if let name @ ExpDesc::Name(_) = lhs {
                name.get_local_or_discharge_at_location(program, dst, compile_context)?
            } else {
                lhs.discharge(&ExpDesc::Local(usize::from(dst)), program, compile_context)?;
                dst
            };

            program.byte_codes.push(bytecode(dst, lhs_loc, constant));

            Ok(())
        }

        fn discharge_relational_binop_into_local(
            bytecode: fn(u8, u8, u8) -> ByteCode,
            lhs: &ExpDesc,
            rhs: &ExpDesc,
            dst: u8,
            test: bool,
            program: &mut Program,
            compile_context: &mut CompileContext,
        ) -> Result<(), Error> {
            let (lhs_loc, used_dst) = if let name @ ExpDesc::Name(_) = lhs {
                let local =
                    name.get_local_or_discharge_at_location(program, dst, compile_context)?;
                (local, local == dst)
            } else {
                lhs.discharge(&ExpDesc::Local(usize::from(dst)), program, compile_context)?;
                (dst, true)
            };

            let (rhs_loc, stack_top) = if used_dst {
                compile_context.reserve_stack_top()
            } else {
                (dst, ExpDesc::Local(usize::from(dst)))
            };

            let rhs_loc = if let name @ ExpDesc::Name(_) = rhs {
                name.get_local_or_discharge_at_location(program, rhs_loc, compile_context)?
            } else {
                rhs.discharge(&stack_top, program, compile_context)?;
                rhs_loc
            };

            if used_dst {
                compile_context.stack_top -= 1;
            }

            program
                .byte_codes
                .push(bytecode(lhs_loc, rhs_loc, test as u8));
            let jump = program.byte_codes.len();
            program.byte_codes.push(ByteCode::Jmp(0));
            compile_context.jumps_to_false.push(jump);

            Ok(())
        }

        match (self, dst) {
            (Self::Binop(Binop::Add, lhs, rhs), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;

                if rhs.is_i8_integer() {
                    let ExpDesc::Integer(integer) = rhs.as_ref() else {
                        unreachable!("Exp should be Integer, but was {:?}.", rhs);
                    };
                    let Ok(integer) = i8::try_from(*integer) else {
                        unreachable!("Integer should fit into i8, but was {}.", integer);
                    };

                    discharge_binop_integer(
                        ByteCode::AddInteger,
                        lhs,
                        integer,
                        dst,
                        program,
                        compile_context,
                    )
                } else {
                    discharge_binop(ByteCode::Add, lhs, rhs, dst, program, compile_context)
                }
            }
            (Self::Binop(Binop::And, lhs, rhs), local @ Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;

                let jump_to_block_count = compile_context.jumps_to_block.len();
                if let name @ ExpDesc::Name(_) = lhs.as_ref() {
                    name.get_local_or_discharge_at_location(program, dst, compile_context)?;
                } else {
                    lhs.discharge(local, program, compile_context)?;
                };
                let after_lhs = program.byte_codes.len();

                if !lhs.is_relational() {
                    program.byte_codes.push(ByteCode::Test(dst, 0));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(ByteCode::Jmp(0));
                    compile_context.jumps_to_end.push(jump);
                }
                for jump in compile_context.jumps_to_block.drain(jump_to_block_count..) {
                    program.byte_codes[jump] = ByteCode::Jmp(
                        i16::try_from((after_lhs + 2) - jump - 1).map_err(|_| Error::LongJump)?,
                    );
                }

                rhs.discharge(local, program, compile_context)?;
                Ok(())
            }
            (Self::Binop(Binop::Or, lhs, rhs), local @ Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;

                if let name @ ExpDesc::Name(_) = lhs.as_ref() {
                    name.get_local_or_discharge_at_location(program, dst, compile_context)?;
                } else {
                    lhs.discharge(local, program, compile_context)?;
                };

                program.byte_codes.push(ByteCode::Test(dst, 1));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_block.push(jump);

                rhs.discharge(local, program, compile_context)?;
                Ok(())
            }
            (Self::Binop(Binop::Equal, lhs, rhs), local @ Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;

                if rhs.is_i8_integer() {
                    let lhs_loc = if let name @ ExpDesc::Name(_) = lhs.as_ref() {
                        name.get_local_or_discharge_at_location(program, dst, compile_context)?
                    } else {
                        lhs.discharge(local, program, compile_context)?;
                        dst
                    };

                    let ExpDesc::Integer(integer) = rhs.as_ref() else {
                        unreachable!(
                            "Match guard guarantees rhs is a Integer, but was {:?}.",
                            rhs
                        );
                    };
                    let Ok(integer) = i8::try_from(*integer) else {
                        unreachable!("Match guard garantees this is a i8, but was {:?}.", integer);
                    };

                    program
                        .byte_codes
                        .push(ByteCode::EqualInteger(lhs_loc, integer, 0));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(ByteCode::Jmp(0));
                    compile_context.jumps_to_false.push(jump);
                } else if rhs.is_integer() || rhs.is_float() || rhs.is_string() {
                    let lhs_loc = if let name @ ExpDesc::Name(_) = lhs.as_ref() {
                        name.get_local_or_discharge_at_location(program, dst, compile_context)?
                    } else {
                        lhs.discharge(local, program, compile_context)?;
                        dst
                    };

                    let constant = match rhs.as_ref() {
                        ExpDesc::Integer(integer) => program.push_constant(*integer)?,
                        ExpDesc::Float(float) => program.push_constant(*float)?,
                        ExpDesc::String(string) => program.push_constant(*string)?,
                        other => unreachable!(
                            "Match guard guarantees rhs is a a constant, but was {:?}.",
                            other
                        ),
                    };

                    program
                        .byte_codes
                        .push(ByteCode::EqualConstant(lhs_loc, constant, 0));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(ByteCode::Jmp(0));
                    compile_context.jumps_to_false.push(jump);
                } else {
                    unimplemented!("eq")
                }

                Ok(())
            }
            (Self::Binop(Binop::LessThan, lhs, rhs), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                discharge_relational_binop_into_local(
                    ByteCode::LessThan,
                    lhs,
                    rhs,
                    dst,
                    false,
                    program,
                    compile_context,
                )
            }
            (Self::Binop(Binop::GreaterThan, lhs, rhs), local @ Self::Local(dst))
                if rhs.is_i8_integer() =>
            {
                let dst = u8::try_from(*dst)?;

                let lhs_loc = if let name @ ExpDesc::Name(_) = lhs.as_ref() {
                    name.get_local_or_discharge_at_location(program, dst, compile_context)?
                } else {
                    lhs.discharge(local, program, compile_context)?;
                    dst
                };

                let ExpDesc::Integer(integer) = rhs.as_ref() else {
                    unreachable!(
                        "Match guard guarantees rhs is a Integer, but was {:?}.",
                        rhs
                    );
                };
                let Ok(integer) = i8::try_from(*integer) else {
                    unreachable!("Match guard garantees this is a i8, but was {:?}.", integer);
                };

                program
                    .byte_codes
                    .push(ByteCode::GreaterThanInteger(lhs_loc, integer, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_false.push(jump);

                Ok(())
            }
            (Self::Binop(Binop::GreaterThan, lhs, rhs), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                discharge_relational_binop_into_local(
                    ByteCode::LessThan,
                    rhs,
                    lhs,
                    dst,
                    false,
                    program,
                    compile_context,
                )
            }
            (Self::Binop(Binop::LessEqual, lhs, rhs), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                discharge_relational_binop_into_local(
                    ByteCode::LessEqual,
                    lhs,
                    rhs,
                    dst,
                    false,
                    program,
                    compile_context,
                )
            }
            (Self::Binop(Binop::GreaterEqual, lhs, rhs), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                discharge_relational_binop_into_local(
                    ByteCode::LessEqual,
                    rhs,
                    lhs,
                    dst,
                    false,
                    program,
                    compile_context,
                )
            }
            (Self::Binop(Binop::And, lhs, rhs), Self::Condition(_)) => {
                let jump_to_block_count = compile_context.jumps_to_block.len();

                lhs.discharge(&Self::Condition(false), program, compile_context)?;
                let after_lhs_cond = program.byte_codes.len();
                for jump in compile_context.jumps_to_block.drain(jump_to_block_count..) {
                    program.byte_codes[jump] = ByteCode::Jmp(
                        i16::try_from(after_lhs_cond - jump - 1).map_err(|_| Error::LongJump)?,
                    );
                }

                rhs.discharge(&Self::Condition(false), program, compile_context)?;

                Ok(())
            }
            (Self::Binop(Binop::Equal, lhs, rhs), Self::Condition(cond))
                if rhs.is_integer() || rhs.is_float() || rhs.is_string() =>
            {
                let (lhs_loc, stack_top) = compile_context.reserve_stack_top();
                let lhs_loc = if let name @ ExpDesc::Name(_) = lhs.as_ref() {
                    name.get_local_or_discharge_at_location(program, lhs_loc, compile_context)?
                } else {
                    lhs.discharge(&stack_top, program, compile_context)?;
                    lhs_loc
                };

                let constant = match rhs.as_ref() {
                    ExpDesc::Integer(integer) => program.push_constant(*integer)?,
                    ExpDesc::Float(float) => program.push_constant(*float)?,
                    ExpDesc::String(string) => program.push_constant(*string)?,
                    other => unreachable!(
                        "Match guard guarantees rhs is a a constant, but was {:?}.",
                        other
                    ),
                };

                program
                    .byte_codes
                    .push(ByteCode::EqualConstant(lhs_loc, constant, *cond as u8));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_end.push(jump);

                compile_context.stack_top -= 1;

                Ok(())
            }
            (Self::Binop(Binop::GreaterThan, lhs, rhs), Self::Condition(_)) => {
                let (stack_loc, stack_top) = compile_context.reserve_stack_top();
                let mut reserved_stack = 1;
                let (lhs_loc, used_stack_top) = if matches!(lhs.as_ref(), ExpDesc::Name(_)) {
                    let loc = lhs.get_local_or_discharge_at_location(
                        program,
                        stack_loc,
                        compile_context,
                    )?;
                    (loc, loc == stack_loc)
                } else {
                    lhs.discharge(&stack_top, program, compile_context)?;
                    (stack_loc, true)
                };

                let (rhs_loc, stack_top) = if used_stack_top {
                    reserved_stack += 1;
                    compile_context.reserve_stack_top()
                } else {
                    (stack_loc, stack_top)
                };
                let rhs_loc = if matches!(rhs.as_ref(), ExpDesc::Name(_)) {
                    rhs.get_local_or_discharge_at_location(program, rhs_loc, compile_context)?
                } else {
                    rhs.discharge(&stack_top, program, compile_context)?;
                    rhs_loc
                };

                program
                    .byte_codes
                    .push(ByteCode::LessThan(rhs_loc, lhs_loc, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_end.push(jump);

                compile_context.stack_top -= reserved_stack;

                Ok(())
            }
            (Self::Binop(Binop::LessEqual, lhs, rhs), Self::Condition(_)) => {
                let (stack_loc, stack_top) = compile_context.reserve_stack_top();
                let mut reserved_stack = 1;
                let (lhs_loc, used_stack_top) = if matches!(lhs.as_ref(), ExpDesc::Name(_)) {
                    let loc = lhs.get_local_or_discharge_at_location(
                        program,
                        stack_loc,
                        compile_context,
                    )?;
                    (loc, loc == stack_loc)
                } else {
                    lhs.discharge(&stack_top, program, compile_context)?;
                    (stack_loc, true)
                };

                let (rhs_loc, stack_top) = if used_stack_top {
                    reserved_stack += 1;
                    compile_context.reserve_stack_top()
                } else {
                    (stack_loc, stack_top)
                };
                let rhs_loc = if matches!(rhs.as_ref(), ExpDesc::Name(_)) {
                    rhs.get_local_or_discharge_at_location(program, rhs_loc, compile_context)?
                } else {
                    rhs.discharge(&stack_top, program, compile_context)?;
                    rhs_loc
                };

                program
                    .byte_codes
                    .push(ByteCode::LessEqual(lhs_loc, rhs_loc, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_end.push(jump);

                compile_context.stack_top -= reserved_stack;

                Ok(())
            }
            (Self::Binop(Binop::GreaterEqual, lhs, rhs), Self::Condition(cond))
                if rhs.is_i8_integer() =>
            {
                let (lhs_loc, _) = compile_context.reserve_stack_top();
                let lhs_loc =
                    lhs.get_local_or_discharge_at_location(program, lhs_loc, compile_context)?;

                let ExpDesc::Integer(integer) = rhs.as_ref() else {
                    unreachable!(
                        "Match guard garantees this is an Integer, but was {:?}.",
                        rhs
                    );
                };
                let Ok(integer) = i8::try_from(*integer) else {
                    unreachable!(
                        "Match guard garantees Integer can be turned into a i8, but was {:?}.",
                        integer
                    );
                };

                program.byte_codes.push(ByteCode::GreaterEqualInteger(
                    lhs_loc,
                    integer,
                    *cond as u8,
                ));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_end.push(jump);

                compile_context.stack_top -= 1;

                Ok(())
            }
            (Self::Binop(binop, lhs, rhs), Self::Local(dst))
                if rhs.is_float() && binop.arithmetic_binary_operator_constant() =>
            {
                let dst = u8::try_from(*dst)?;

                let bytecode = match binop {
                        Binop::Add => unimplemented!("addk"),
                        Binop::Sub => unimplemented!("subk"),
                        Binop::Mul => ByteCode::MulConstant,
                        Binop::Mod => unimplemented!("modk"),
                        Binop::Pow => unimplemented!("powk"),
                        Binop::Div => unimplemented!("divk"),
                        Binop::Idiv => unimplemented!("idivk"),
                        Binop::BitAnd => unimplemented!("bandk"),
                        Binop::BitOr => unimplemented!("bork"),
                        Binop::BitXor => unimplemented!("bxork"),
                        other => unreachable!("Match guard garantees this is a binary operator that has a constant bytecode, but was {:?}.", other),
                    };

                let ExpDesc::Float(float) = rhs.as_ref() else {
                    unreachable!("Match guard garantees this is a Float, but was {:?}.", rhs);
                };
                let constant = program.push_constant(*float)?;

                discharge_binop_constant(bytecode, lhs, constant, dst, program, compile_context)
            }
            (Self::Binop(binop, lhs, rhs), Self::Local(dst))
                if binop.arithmetic_binary_operator() =>
            {
                let dst = u8::try_from(*dst)?;

                let bytecode = match binop {
                        Binop::Add => ByteCode::Add,
                        Binop::Sub => ByteCode::Sub,
                        Binop::Mul => ByteCode::Mul,
                        Binop::Mod => ByteCode::Mod,
                        Binop::Pow => ByteCode::Pow,
                        Binop::Div => ByteCode::Div,
                        Binop::Idiv => ByteCode::Idiv,
                        Binop::ShiftLeft => ByteCode::ShiftLeft,
                        Binop::ShiftRight => ByteCode::ShiftRight,
                        Binop::BitAnd => ByteCode::BitAnd,
                        Binop::BitOr => ByteCode::BitOr,
                        Binop::BitXor => ByteCode::BitXor,
                        Binop::Concat => ByteCode::Concat,
                        other => unreachable!("Match guard garantees this is an arithmetic binary operator, but was {:?}.", other),
                    };

                discharge_binop(bytecode, lhs, rhs, dst, program, compile_context)
            }
            (Self::Binop(binop, lhs, rhs), Self::Local(dst))
                if binop.relational_binary_operator() =>
            {
                let dst = u8::try_from(*dst)?;

                let (bytecode, lhs, rhs): (fn(u8,u8,u8) -> ByteCode, _ ,_) = match binop {
                        Binop::Equal => unimplemented!("eq"),
                        Binop::NotEqual => unimplemented!("neq"),
                        Binop::GreaterThan => (ByteCode::LessThan, rhs, lhs),
                        Binop::LessEqual => (ByteCode::LessEqual, lhs, rhs),
                        Binop::GreaterEqual => (ByteCode::LessEqual, rhs, lhs),
                        other => unreachable!("Match guard garantees this is an relational binary operator, but was {:?}.", other),
                    };

                discharge_binop(bytecode, lhs, rhs, dst, program, compile_context)
            }
            (Self::Binop(binop, lhs, rhs), Self::Condition(cond)) => {
                match binop {
                    Binop::Or => {
                        lhs.discharge(&Self::Condition(true), program, compile_context)?;
                        let Some(last_jump) = compile_context.jumps_to_end.pop() else {
                            unreachable!("There will be always at least one jump to end.");
                        };
                        compile_context.jumps_to_block.push(last_jump);

                        rhs.discharge(&Self::Condition(*cond), program, compile_context)?;
                    }
                    Binop::And => {
                        let jump_to_block_count = compile_context.jumps_to_block.len();

                        lhs.discharge(&Self::Condition(false), program, compile_context)?;
                        let after_lhs_cond = program.byte_codes.len();
                        for jump in compile_context.jumps_to_block.drain(jump_to_block_count..) {
                            program.byte_codes[jump] = ByteCode::Jmp(
                                i16::try_from(after_lhs_cond - jump - 1)
                                    .map_err(|_| Error::LongJump)?,
                            );
                        }

                        rhs.discharge(&Self::Condition(false), program, compile_context)?;
                    }
                    Binop::LessEqual => {
                        let (lhs_loc, stack_top) = compile_context.reserve_stack_top();
                        lhs.discharge(&stack_top, program, compile_context)?;

                        let (rhs_loc, stack_top) = compile_context.reserve_stack_top();
                        rhs.discharge(&stack_top, program, compile_context)?;

                        program
                            .byte_codes
                            .push(ByteCode::LessEqual(lhs_loc, rhs_loc, 1));
                    }
                    Binop::GreaterEqual => match (lhs.as_ref(), rhs.as_ref()) {
                        (lhs, ExpDesc::Integer(integer)) => {
                            let (lhs_loc, stack_top) = compile_context.reserve_stack_top();
                            lhs.discharge(&stack_top, program, compile_context)?;

                            if let Ok(integer) = i8::try_from(*integer) {
                                program
                                    .byte_codes
                                    .push(ByteCode::GreaterEqualInteger(lhs_loc, integer, 0));

                                compile_context.stack_top -= 1;
                            } else {
                                let (rhs_loc, stack_top) = compile_context.reserve_stack_top();
                                rhs.discharge(&stack_top, program, compile_context)?;

                                program
                                    .byte_codes
                                    .push(ByteCode::LessEqual(rhs_loc, lhs_loc, 1));

                                compile_context.stack_top -= 2;
                            }
                        }
                        (lhs, rhs) => unimplemented!(
                            "GreaterEqual taking {:?} and {:?} is still unimplemented.",
                            lhs,
                            rhs
                        ),
                    },
                    Binop::Equal => match (lhs.as_ref(), rhs.as_ref()) {
                        (lhs, ExpDesc::String(string)) => {
                            let (lhs_loc, stack_top) = compile_context.reserve_stack_top();
                            lhs.discharge(&stack_top, program, compile_context)?;

                            let string = program.push_constant(*string)?;

                            program
                                .byte_codes
                                .push(ByteCode::EqualConstant(lhs_loc, string, 1));

                            compile_context.stack_top -= 2;
                        }
                        (lhs, rhs) => unimplemented!(
                            "GreaterEqual taking {:?} and {:?} is still unimplemented.",
                            lhs,
                            rhs
                        ),
                    },
                    other => unimplemented!("Condition with {:?} is unimplemented.", other),
                }

                Ok(())
            }
            other => unreachable!("Unop can't be discharged in {:?}.", other),
        }
    }

    fn discharge_local(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Local(src) = &self else {
            unreachable!("Should never be anything other than Local");
        };

        match dst {
            Self::Local(dst) => {
                if src == dst {
                    Ok(())
                } else {
                    let src = u8::try_from(*src)?;
                    let dst = u8::try_from(*dst)?;

                    program.byte_codes.push(ByteCode::Move(dst, src));

                    Ok(())
                }
            }
            Self::Global(key) => {
                let src = u8::try_from(*src)?;
                let key = u8::try_from(*key)?;

                program.byte_codes.push(ByteCode::SetGlobal(key, src));

                Ok(())
            }
            Self::TableAccess {
                table,
                key,
                record: true,
            } if table.is_name() => {
                let (table_loc, _) = compile_context.reserve_stack_top();
                let table_loc = table.get_local_or_discharge_at_location(
                    program,
                    table_loc,
                    compile_context,
                )?;
                compile_context.stack_top -= 1;

                match key.as_ref() {
                    ExpDesc::Name(name) => {
                        let constant = program.push_constant(*name)?;
                        program.byte_codes.push(ByteCode::SetField(
                            table_loc,
                            constant,
                            u8::try_from(*src)?,
                        ));

                        Ok(())
                    }
                    other => Err(Error::TableRecordAccess(other.static_type_name())),
                }
            }
            Self::TableAccess {
                table,
                key,
                record: true,
            } if table.is_local() => {
                let ExpDesc::Local(table_loc) = table.as_ref() else {
                    unreachable!("Should be a Local, but was {:?}", table);
                };

                match key.as_ref() {
                    ExpDesc::Name(name) => {
                        let constant = program.push_constant(*name)?;
                        program.byte_codes.push(ByteCode::SetField(
                            u8::try_from(*table_loc)?,
                            constant,
                            u8::try_from(*src)?,
                        ));

                        Ok(())
                    }
                    other => Err(Error::TableRecordAccess(other.static_type_name())),
                }
            }
            Self::TableAccess {
                table,
                key,
                record: true,
            } if table.is_global() => {
                let ExpDesc::Global(_) = table.as_ref() else {
                    unreachable!("Should be a Global, but was {:?}", table);
                };

                let (_, stack_top) = compile_context.reserve_stack_top();

                table.discharge(&stack_top, program, compile_context)?;
                self.discharge(
                    &ExpDesc::TableAccess {
                        table: Box::new(stack_top),
                        key: key.clone(),
                        record: true,
                    },
                    program,
                    compile_context,
                )?;

                compile_context.stack_top -= 1;

                Ok(())
            }
            Self::Condition(condition) => {
                let src = u8::try_from(*src)?;
                program
                    .byte_codes
                    .push(ByteCode::Test(src, *condition as u8));

                Ok(())
            }
            other => unreachable!("Local can't be discharged in {:?}.", other),
        }
    }

    fn discharge_global(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Global(key) = &self else {
            unreachable!("Should never be anything other than Global");
        };

        match dst {
            Self::Local(dst) => {
                let key = u8::try_from(*key)?;
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(ByteCode::GetGlobal(dst, key));

                Ok(())
            }
            Self::Global(dst_key) => {
                let src_key = u8::try_from(*key)?;
                let dst_key = u8::try_from(*dst_key)?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalGlobal(dst_key, src_key));

                Ok(())
            }
            Self::Condition(condition) => {
                let (dst, stack_top) = compile_context.reserve_stack_top();
                self.discharge(&stack_top, program, compile_context)?;

                program
                    .byte_codes
                    .push(ByteCode::Test(dst, *condition as u8));

                compile_context.stack_top -= 1;

                Ok(())
            }
            other => unreachable!("Global can't be discharged in {:?}.", other),
        }
    }

    fn discharge_table(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Table(fields) = &self else {
            unreachable!("Should never be anything other than Table");
        };

        match dst {
            Self::Name(name) => {
                let dest = match compile_context.find_name(name) {
                    Some(local) => local,
                    None => ExpDesc::Global(usize::from(program.push_constant(*name)?)),
                };
                self.discharge(&dest, program, compile_context)
            }
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                let array_items = u8::try_from(
                    fields
                        .iter()
                        .filter(|(key, _)| matches!(key, TableKey::Array))
                        .count(),
                )?;
                let table_items = u8::try_from(fields.len())? - array_items;

                let table = dst;

                program
                    .byte_codes
                    .push(ByteCode::NewTable(dst, array_items, table_items));

                for (key, val) in fields {
                    match key {
                        TableKey::Array => {
                            let (_, stack_top) = compile_context.reserve_stack_top();
                            val.discharge(&stack_top, program, compile_context)?;
                        }
                        TableKey::Record(key) => {
                            let ExpDesc::Name(name) = key.as_ref() else {
                                unreachable!("`TableRecordField`'s key should always be a `Name`.");
                            };
                            let constant = program.push_constant(*name)?;

                            let (val_loc, stack_top) = compile_context.reserve_stack_top();
                            val.discharge(&stack_top, program, compile_context)?;
                            compile_context.stack_top -= 1;

                            program
                                .byte_codes
                                .push(ByteCode::SetField(table, constant, val_loc));
                        }
                        TableKey::General(key) => {
                            let (key_loc, stack_top) = compile_context.reserve_stack_top();
                            key.discharge(&stack_top, program, compile_context)?;

                            let (val_loc, stack_top) = compile_context.reserve_stack_top();
                            val.discharge(&stack_top, program, compile_context)?;

                            compile_context.stack_top -= 2;

                            program
                                .byte_codes
                                .push(ByteCode::SetTable(table, key_loc, val_loc));
                        }
                    }
                }

                if array_items > 0 {
                    program
                        .byte_codes
                        .push(ByteCode::SetList(table, array_items));
                }

                compile_context.stack_top -= array_items;

                Ok(())
            }
            Self::Global(dst) => {
                let dst = u8::try_from(*dst)?;

                let array_items = u8::try_from(
                    fields
                        .iter()
                        .filter(|(key, _)| matches!(key, TableKey::Array))
                        .count(),
                )?;
                let table_items = u8::try_from(fields.len())? - array_items;

                let (table_loc, _) = compile_context.reserve_stack_top();
                program
                    .byte_codes
                    .push(ByteCode::NewTable(table_loc, array_items, table_items));

                for (key, val) in fields {
                    match key {
                        TableKey::Array => {
                            let (_, stack_top) = compile_context.reserve_stack_top();
                            val.discharge(&stack_top, program, compile_context)?;
                        }
                        TableKey::Record(key) => {
                            let ExpDesc::Name(name) = key.as_ref() else {
                                unreachable!("`TableRecordField`'s key should always be a `Name`.");
                            };
                            let constant = program.push_constant(*name)?;

                            let (val_loc, stack_top) = compile_context.reserve_stack_top();
                            val.discharge(&stack_top, program, compile_context)?;
                            compile_context.stack_top -= 1;

                            program
                                .byte_codes
                                .push(ByteCode::SetField(table_loc, constant, val_loc));
                        }
                        TableKey::General(key) => {
                            let (key_loc, stack_top) = compile_context.reserve_stack_top();
                            key.discharge(&stack_top, program, compile_context)?;

                            let (val_loc, stack_top) = compile_context.reserve_stack_top();
                            val.discharge(&stack_top, program, compile_context)?;

                            compile_context.stack_top -= 2;

                            program
                                .byte_codes
                                .push(ByteCode::SetTable(table_loc, key_loc, val_loc));
                        }
                    }
                }

                if array_items > 0 {
                    program
                        .byte_codes
                        .push(ByteCode::SetList(table_loc, array_items));
                }

                program.byte_codes.push(ByteCode::SetGlobal(dst, table_loc));

                compile_context.stack_top -= array_items + 1;

                Ok(())
            }
            other => unreachable!("Table can't be discharged in {:?}.", other),
        }
    }

    fn discharge_table_access(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::TableAccess {
            table: _,
            key: _,
            record: _,
        } = &self
        else {
            unreachable!("Should never be anything other than TableAccess");
        };

        match (self, dst) {
            (
                Self::TableAccess {
                    table,
                    key,
                    record: false,
                },
                Self::Local(dst),
            ) => {
                let dst = u8::try_from(*dst)?;

                log::trace!("{:?}", table);
                let table_loc =
                    table.get_local_or_discharge_at_location(program, dst, compile_context)?;

                match key.as_ref() {
                    ExpDesc::Integer(integer) => {
                        if let Ok(index) = u8::try_from(*integer) {
                            program
                                .byte_codes
                                .push(ByteCode::GetInt(dst, table_loc, index));
                        } else {
                            let constant = program.push_constant(*integer)?;
                            program
                                .byte_codes
                                .push(ByteCode::GetField(dst, table_loc, constant));
                        }
                    }
                    ExpDesc::String(string) => {
                        let constant = program.push_constant(*string)?;
                        program
                            .byte_codes
                            .push(ByteCode::GetField(dst, table_loc, constant));
                    }
                    other => {
                        unimplemented!(
                            "Only integer and string general access is implemented for now. {:?}",
                            other
                        )
                    }
                }

                Ok(())
            }
            (
                Self::TableAccess {
                    table,
                    key,
                    record: true,
                },
                Self::Local(dst),
            ) => {
                let dst = u8::try_from(*dst)?;

                let table_loc =
                    table.get_local_or_discharge_at_location(program, dst, compile_context)?;

                match key.as_ref() {
                    ExpDesc::Name(name) => {
                        let constant = program.push_constant(*name)?;
                        program
                            .byte_codes
                            .push(ByteCode::GetField(dst, table_loc, constant));

                        Ok(())
                    }
                    other => Err(Error::TableRecordAccess(other.static_type_name())),
                }
            }
            (
                rhs @ Self::TableAccess {
                    table: _,
                    key: _,
                    record: _,
                },
                lhs @ Self::TableAccess {
                    table: _,
                    key: _,
                    record: _,
                },
            ) => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                log::trace!("{:?}", stack_top);
                rhs.discharge(&stack_top, program, compile_context)?;

                stack_top.discharge(lhs, program, compile_context)?;
                compile_context.stack_top -= 1;

                Ok(())
            }

            other => unreachable!("TableAccess can't be discharged in {:?}.", other),
        }
    }

    fn discharge_closure(&self, dst: &ExpDesc<'a>, program: &mut Program) -> Result<(), Error> {
        let ExpDesc::Closure(index) = &self else {
            unreachable!("Should never be anything other than TableAccess");
        };

        match dst {
            ExpDesc::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(ByteCode::Closure(dst, *index));

                Ok(())
            }
            other => unreachable!("TableAccess can't be discharged in {:?}.", other),
        }
    }

    fn discharge_function_call(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::FunctionCall(func, args) = &self else {
            unreachable!("Should never be anything other than FunctionCall");
        };

        match dst {
            stack_top @ ExpDesc::Local(func_index) => {
                let jumps_to_block_count = compile_context.jumps_to_block.len();
                let jumps_to_end_count = compile_context.jumps_to_end.len();
                let jumps_to_false_count = compile_context.jumps_to_false.len();

                let func_index = u8::try_from(*func_index)?;
                func.discharge(stack_top, program, compile_context)?;

                for arg in args.iter() {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    arg.discharge(&stack_top, program, compile_context)?;
                }

                if let Some(ByteCode::Call(_, _, out)) = program.byte_codes.last_mut() {
                    *out = 0;
                }

                let after_args = program.byte_codes.len();

                for jump in compile_context.jumps_to_block.drain(jumps_to_block_count..) {
                    program.byte_codes[jump] = ByteCode::Jmp(
                        i16::try_from(after_args - jump - 1).map_err(|_| Error::LongJump)?,
                    );
                }
                for jump in compile_context.jumps_to_end.drain(jumps_to_end_count..) {
                    program.byte_codes[jump] = ByteCode::Jmp(
                        i16::try_from(after_args - jump - 1).map_err(|_| Error::LongJump)?,
                    );
                }
                if compile_context.jumps_to_false.len() > jumps_to_false_count {
                    let false_bytecode = program.byte_codes.len();
                    program
                        .byte_codes
                        .push(ByteCode::LoadFalseSkip(func_index + 1));

                    let Some(last) = compile_context.jumps_to_false.pop() else {
                        unreachable!("Should always have at least one jump on the list.");
                    };
                    program.byte_codes[last] = ByteCode::Jmp(1);
                    match &mut program.byte_codes[last - 1] {
                        ByteCode::EqualConstant(_, _, test)
                        | ByteCode::EqualInteger(_, _, test)
                        | ByteCode::LessThan(_, _, test)
                        | ByteCode::LessEqual(_, _, test)
                        | ByteCode::GreaterThanInteger(_, _, test)
                        | ByteCode::GreaterEqualInteger(_, _, test) => *test ^= 1,
                        other => unreachable!(
                            "Last test should always be a relational test, but was {:?}.",
                            other
                        ),
                    }

                    program.byte_codes.push(ByteCode::LoadTrue(func_index + 1));
                    for jump in compile_context.jumps_to_false.drain(jumps_to_false_count..) {
                        program.byte_codes[jump] = ByteCode::Jmp(
                            i16::try_from(false_bytecode - jump - 1)
                                .map_err(|_| Error::LongJump)?,
                        );
                    }
                }

                // TODO do proper bytecode
                let args_len = if let Some(ExpDesc::FunctionCall(_, _)) = args.last() {
                    0
                } else if let Some(ByteCode::VariadicArguments(_, _)) = program.byte_codes.last() {
                    0
                } else {
                    u8::try_from(args.len())? + 1
                };

                program
                    .byte_codes
                    .push(ByteCode::Call(func_index, args_len, 1));

                compile_context.stack_top -= u8::try_from(args.len())?;

                Ok(())
            }
            other => unreachable!("FunctionCall can't be discharged in {:?}.", other),
        }
    }

    fn discharge_variable_arguments(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        _compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match dst {
            ExpDesc::Local(dst) => {
                let dst = u8::try_from(*dst)?;
                program.byte_codes.push(ByteCode::VariadicArguments(dst, 0));
            }
            _ => unimplemented!(
                "Discharging Variable arguments into {:?} is not yet supported.",
                dst
            ),
        }
        Ok(())
    }

    pub fn get_local_or_discharge_at_location(
        &self,
        program: &mut Program,
        location: u8,
        compile_context: &mut CompileContext,
    ) -> Result<u8, Error> {
        match self {
            ExpDesc::Name(table_name) => match compile_context.find_name(table_name) {
                Some(ExpDesc::Local(pos)) => u8::try_from(pos).map_err(Error::from),
                Some(other) => unreachable!(
                    "CompileContext::find_name should always return ExpDesc::Local, but returned {:?}.",
                    other
                ),
                None => {
                    self.discharge(
                        &ExpDesc::Local(usize::from(location)),
                        program,
                        compile_context,
                    )?;
                    Ok(location)
                }
            },
            other => unreachable!("Should always be called with a ExpDesc::Name, but was called with {:?}.", other),
        }
    }

    fn is_integer(&self) -> bool {
        matches!(self, ExpDesc::Integer(_))
    }

    fn is_i8_integer(&self) -> bool {
        matches!(self, ExpDesc::Integer(integer) if i8::try_from(*integer).is_ok())
    }

    fn is_float(&self) -> bool {
        matches!(self, ExpDesc::Float(_))
    }

    fn is_string(&self) -> bool {
        matches!(self, ExpDesc::String(_))
    }

    fn is_name(&self) -> bool {
        matches!(self, ExpDesc::Name(_))
    }

    fn is_local(&self) -> bool {
        matches!(self, ExpDesc::Local(_))
    }

    fn is_global(&self) -> bool {
        matches!(self, ExpDesc::Global(_))
    }

    fn is_relational(&self) -> bool {
        matches!(self, Self::Binop(binop, _, _) if matches!(binop, Binop::Equal | Binop::NotEqual | Binop::LessThan | Binop::GreaterThan | Binop::LessEqual | Binop::GreaterEqual))
    }

    fn static_type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Name(_) => "name",
            Self::Unop(_, _) => "unop",
            Self::Binop(_, _, _) => "binop",
            Self::Local(_) => "local",
            Self::Global(_) => "global",
            Self::Table(_) => "table",
            Self::TableAccess {
                table: _,
                key: _,
                record: _,
            } => "table_access",
            Self::Condition(_) => "condition",
            Self::Closure(_) => "closure",
            Self::FunctionCall(_, _) => "function_call",
            Self::VariadicArguments => "variadic arguments",
        }
    }
}
