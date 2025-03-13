use alloc::boxed::Box;

use crate::{
    bytecode::OpCode,
    ext::{FloatExt, Unescape},
    value::Value,
};

use super::{
    binops::Binop,
    compile_context::CompileContext,
    helper_types::{ExpList, TableFields, TableKey},
    Bytecode, Error, Proto,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExpDesc<'a> {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    Name(&'a str),
    Unop(fn(u8, u8) -> Bytecode, Box<ExpDesc<'a>>),
    Binop(Binop, Box<ExpDesc<'a>>, Box<ExpDesc<'a>>),
    Local(usize),
    Global(usize),
    Upvalue(usize),
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
    MethodCall(Box<ExpDesc<'a>>, Box<ExpDesc<'a>>, ExpList<'a>),
    VariadicArguments,
}

impl<'a> ExpDesc<'a> {
    pub fn discharge(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
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
            ExpDesc::Binop(Binop::Add, _, _) => {
                self.discharge_binop_add(dst, program, compile_context)
            }
            ExpDesc::Binop(Binop::Sub, _, _) => {
                self.discharge_binop_sub(dst, program, compile_context)
            }
            ExpDesc::Binop(Binop::Concat, _, _) => {
                self.discharge_binop_concat(dst, program, compile_context)
            }
            ExpDesc::Binop(_, _, _) => self.discharge_binop(dst, program, compile_context),
            ExpDesc::Local(_) => self.discharge_local(dst, program, compile_context),
            ExpDesc::Global(_) => self.discharge_global(dst, program, compile_context),
            ExpDesc::Upvalue(_) => self.discharge_upvalue(dst, program, compile_context),
            ExpDesc::Table(_) => self.discharge_table(dst, program, compile_context),
            ExpDesc::TableAccess {
                table: _,
                key: _,
                record: false,
            } => self.discharge_general_table_access(dst, program, compile_context),
            ExpDesc::TableAccess {
                table: _,
                key: _,
                record: true,
            } => self.discharge_record_table_access(dst, program, compile_context),
            ExpDesc::Closure(_) => self.discharge_closure(dst, program),
            ExpDesc::FunctionCall(_, _) => {
                self.discharge_function_call(dst, program, compile_context)
            }
            ExpDesc::MethodCall(_, _, _) => {
                self.discharge_method_call(dst, program, compile_context)
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

    fn discharge_nil(&self, dst: &ExpDesc<'a>, program: &mut Proto) -> Result<(), Error> {
        let ExpDesc::Nil = &self else {
            unreachable!("Should never be anything other than Nil");
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;
                program.byte_codes.push(Bytecode::load_nil(dst, 0));

                Ok(())
            }
            Self::Global(key) => Self::push_value_to_global(program, *key, ()),
            other => unreachable!("Nil can't be discharged in {:?}.", other),
        }
    }

    fn discharge_boolean(&self, dst: &ExpDesc<'a>, program: &mut Proto) -> Result<(), Error> {
        let ExpDesc::Boolean(boolean) = &self else {
            unreachable!("Should never be anything other than Boolean");
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                if *boolean {
                    program.byte_codes.push(Bytecode::load_true(dst));
                } else {
                    program.byte_codes.push(Bytecode::load_false(dst));
                }

                Ok(())
            }
            Self::Global(key) => Self::push_value_to_global(program, *key, *boolean),
            other => unreachable!("Boolean can't be discharged in {:?}.", other),
        }
    }

    fn discharge_integer(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Integer(integer) = &self else {
            unreachable!("Should never be anything other than Integer");
        };

        match dst {
            Self::Name(name) => match compile_context.find_name(name) {
                Some(local) => self.discharge(&local, program, compile_context),
                None => {
                    let key = program.push_constant(*name)?;

                    if name.len() > 32 {
                        let dst = compile_context.stack_top;
                        let key_loc = compile_context.stack_top + 1;

                        let int_constant = program.push_constant(*integer)?;

                        program.byte_codes.push(Bytecode::get_upvalue(dst, 0));
                        program
                            .byte_codes
                            .push(Bytecode::load_constant(key_loc, key));
                        program.byte_codes.push(Bytecode::set_table(
                            dst,
                            key_loc,
                            u8::try_from(int_constant)?,
                            1,
                        ));

                        Ok(())
                    } else {
                        let global = ExpDesc::Global(usize::try_from(key)?);
                        self.discharge(&global, program, compile_context)
                    }
                }
            },
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                let code = if let Ok(i) = i16::try_from(*integer) {
                    Bytecode::load_integer(dst, i32::from(i))
                } else {
                    let position = program.push_constant(*integer)?;
                    Bytecode::load_constant(dst, position)
                };

                program.byte_codes.push(code);

                Ok(())
            }
            Self::Global(key) => Self::push_value_to_global(program, *key, *integer),
            Self::TableAccess {
                table,
                key,
                record: true,
            } => {
                let table_loc = match table.as_ref() {
                    ExpDesc::Local(local) => *local,
                    name_exp @ ExpDesc::Name(name) => {
                        match compile_context.find_name(name) {
                            Some(ExpDesc::Local(local)) => local,
                            Some(other) => {
                                unreachable!("CompileContext::find_name only returns ExpDesc::Local, was {:?}.", other)
                            }
                            None => {
                                let (stack_loc, stack_top) = compile_context.reserve_stack_top();
                                name_exp.discharge(&stack_top, program, compile_context)?;
                                compile_context.stack_top -= 1;

                                usize::from(stack_loc)
                            }
                        }
                    }
                    other => unimplemented!("Only Local or Name for now, was {:?}.", other),
                };

                match key.as_ref() {
                    ExpDesc::Name(name) => {
                        let key_constant = program.push_constant(*name)?;
                        let int_constant = program.push_constant(*integer)?;
                        program.byte_codes.push(Bytecode::set_field(
                            u8::try_from(table_loc)?,
                            u8::try_from(key_constant)?,
                            u8::try_from(int_constant)?,
                            1,
                        ));

                        Ok(())
                    }
                    other => Err(Error::TableRecordAccess(other.static_type_name())),
                }
            }
            other => unreachable!("Integer can't be discharged in {:?}.", other),
        }
    }

    fn discharge_float(&self, dst: &ExpDesc<'a>, program: &mut Proto) -> Result<(), Error> {
        let ExpDesc::Float(float) = &self else {
            unreachable!("Should never be anything other than Float");
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                match float.to_sbx() {
                    Some(i) => {
                        program.byte_codes.push(Bytecode::load_float(dst, i));
                        Ok(())
                    }
                    _ => {
                        let position = program.push_constant(*float)?;
                        program
                            .byte_codes
                            .push(Bytecode::load_constant(dst, position));
                        Ok(())
                    }
                }
            }
            Self::Global(key) => Self::push_value_to_global(program, *key, *float),
            other => unreachable!("Float can't be discharged in {:?}.", other),
        }
    }

    fn discharge_string(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::String(string) = &self else {
            unreachable!("Should never be anything other than String");
        };

        match dst {
            Self::Name(name) => {
                let dst = match compile_context.find_name(name) {
                    Some(local) => local,
                    None => ExpDesc::Global(usize::try_from(program.push_constant(*name)?)?),
                };

                self.discharge(&dst, program, compile_context)
            }
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;
                let string = string.unescape()?;
                let position = program.push_constant(string.as_str())?;

                program
                    .byte_codes
                    .push(Bytecode::load_constant(dst, position));

                Ok(())
            }
            Self::Global(key) => {
                let string = string.unescape()?;
                Self::push_value_to_global(program, *key, string.as_str())
            }
            Self::TableAccess {
                table,
                key,
                record: false,
            } => {
                let table = match table.as_ref() {
                    ExpDesc::Local(table) => table,
                    other => unimplemented!("Can't discharge string to {:?}.", other),
                };

                let key_stack = match key.as_ref() {
                    ExpDesc::Name(name) => {
                        if let Some(ExpDesc::Local(local)) = compile_context.find_name(name) {
                            local
                        } else {
                            unimplemented!("only local");
                        }
                    }
                    other => {
                        unimplemented!("only name for now, was {:?}", other);
                    }
                };
                let string = string.unescape()?;
                let string_constant = program.push_constant(string.as_str())?;

                program.byte_codes.push(Bytecode::set_table(
                    u8::try_from(*table)?,
                    u8::try_from(key_stack)?,
                    u8::try_from(string_constant)?,
                    1,
                ));

                Ok(())
            }
            Self::TableAccess {
                table,
                key,
                record: true,
            } => {
                let ExpDesc::Name(key) = key.as_ref() else {
                    unreachable!("Table Record should always have Name key.");
                };

                let table = match table.as_ref() {
                    ExpDesc::Local(table) => table,
                    other => unimplemented!("Can't discharge string to {:?}.", other),
                };

                let key_constant = program.push_constant(*key)?;
                let string = string.unescape()?;
                let string_constant = program.push_constant(string.as_str())?;

                program.byte_codes.push(Bytecode::set_field(
                    u8::try_from(*table)?,
                    u8::try_from(key_constant)?,
                    u8::try_from(string_constant)?,
                    1,
                ));

                Ok(())
            }
            other => unreachable!("String can't be discharged in {:?}.", other),
        }
    }

    fn discharge_name(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Name(name) = &self else {
            unreachable!("Should never be anything other than Name");
        };

        match dst {
            Self::Name(dst) => {
                if name != dst {
                    let rhs = match compile_context.find_name(name) {
                        Some(local) => local,
                        None => ExpDesc::Global(usize::try_from(program.push_constant(*name)?)?),
                    };
                    let lhs = match compile_context.find_name(dst) {
                        Some(local) => local,
                        None => ExpDesc::Global(usize::try_from(program.push_constant(*dst)?)?),
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
                            .push(Bytecode::move_bytecode(dst, u8::try_from(pos)?));
                    }
                    Some(other) => {
                        unreachable!("Local will always be a `Local`, but was {:?}.", other)
                    }
                    None => {
                        let upvalue = program.push_upvalue("_ENV")?;
                        let constant = program.push_constant(*name)?;

                        if name.len() > 32 {
                            let name_loc = dst + 1;

                            program.byte_codes.push(Bytecode::get_upvalue(dst, upvalue));
                            program
                                .byte_codes
                                .push(Bytecode::load_constant(name_loc, constant));
                            program
                                .byte_codes
                                .push(Bytecode::get_table(dst, dst, name_loc));
                        } else {
                            program.byte_codes.push(Bytecode::get_uptable(
                                dst,
                                upvalue,
                                u8::try_from(constant)?,
                            ));
                        }
                    }
                }

                Ok(())
            }
            table_exp @ Self::TableAccess {
                table,
                key,
                record: _,
            } => match compile_context.find_name(name) {
                Some(local @ ExpDesc::Local(_)) => {
                    local.discharge(table_exp, program, compile_context)?;
                    Ok(())
                }
                Some(upvalue @ ExpDesc::Upvalue(_)) => {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    upvalue.discharge(&stack_top, program, compile_context)?;

                    stack_top.discharge(table_exp, program, compile_context)?;

                    compile_context.stack_top -= 1;

                    Ok(())
                }
                Some(other) => unreachable!(
                    "CompileContenxt::find_name should be Local or Upvalue, but was {:?}.",
                    other
                ),
                None => {
                    let (table_loc, stack_top) = compile_context.reserve_stack_top();
                    table.discharge(&stack_top, program, compile_context)?;

                    match key.as_ref() {
                        ExpDesc::Name(key) => {
                            let key = program.push_constant(*key)?;

                            let (name_loc, name_stack_top) = compile_context.reserve_stack_top();
                            self.discharge(&name_stack_top, program, compile_context)?;

                            program.byte_codes.push(Bytecode::set_field(
                                table_loc,
                                u8::try_from(key)?,
                                name_loc,
                                0,
                            ));

                            compile_context.stack_top -= 2;
                        }
                        other => {
                            unreachable!("Only Name keys are supported, but was {:?}.", other);
                        }
                    }

                    Ok(())
                }
            },
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
                program.byte_codes.push(Bytecode::jump(0));
                compile_context.jumps_to_end.push(jump);

                Ok(())
            }
            other => unreachable!("Name can't be discharged in {:?}.", other),
        }
    }

    fn discharge_unop(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
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

    fn discharge_binop_add(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Binop(Binop::Add, lhs, rhs) = self else {
            unreachable!("Should always be BinopAdd, but was {:?}.", self)
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                if rhs.is_i8_integer() {
                    let ExpDesc::Integer(integer) = rhs.as_ref() else {
                        unreachable!("Exp should be Integer, but was {:?}.", rhs);
                    };
                    let Ok(integer) = i8::try_from(*integer) else {
                        unreachable!("Integer should fit into i8, but was {}.", integer);
                    };

                    Self::discharge_binop_integer(
                        Bytecode::add_integer,
                        lhs,
                        integer,
                        dst,
                        program,
                        compile_context,
                    )
                } else if lhs.is_i8_integer() {
                    let ExpDesc::Integer(integer) = lhs.as_ref() else {
                        unreachable!("Exp should be Integer, but was {:?}.", rhs);
                    };
                    let Ok(integer) = i8::try_from(*integer) else {
                        unreachable!("Integer should fit into i8, but was {}.", integer);
                    };

                    Self::discharge_binop_integer(
                        Bytecode::add_integer,
                        rhs,
                        integer,
                        dst,
                        program,
                        compile_context,
                    )
                } else {
                    Self::discharge_generic_binop(
                        Bytecode::add,
                        lhs,
                        rhs,
                        dst,
                        program,
                        compile_context,
                    )
                }
            }
            other => unreachable!("Can only discharge BinopAdd to local, but was {:?}.", other),
        }
    }

    fn discharge_binop_sub(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Binop(Binop::Sub, lhs, rhs) = self else {
            unreachable!("Should always be BinopSub, but was {:?}.", self)
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                if rhs.is_i8_integer() {
                    let ExpDesc::Integer(integer) = rhs.as_ref() else {
                        unreachable!("Exp should be Integer, but was {:?}.", rhs);
                    };
                    let Ok(integer) = i8::try_from(*integer) else {
                        unreachable!("Integer should fit into i8, but was {}.", integer);
                    };

                    Self::discharge_binop_integer(
                        Bytecode::add_integer,
                        lhs,
                        -integer,
                        dst,
                        program,
                        compile_context,
                    )
                } else if lhs.is_i8_integer() {
                    let ExpDesc::Integer(integer) = lhs.as_ref() else {
                        unreachable!("Exp should be Integer, but was {:?}.", rhs);
                    };
                    let Ok(integer) = i8::try_from(*integer) else {
                        unreachable!("Integer should fit into i8, but was {}.", integer);
                    };

                    Self::discharge_binop_integer(
                        Bytecode::add_integer,
                        rhs,
                        -integer,
                        dst,
                        program,
                        compile_context,
                    )
                } else {
                    Self::discharge_generic_binop(
                        Bytecode::sub,
                        lhs,
                        rhs,
                        dst,
                        program,
                        compile_context,
                    )
                }
            }
            other => unreachable!("Can only discharge BinopSub to local, but was {:?}.", other),
        }
    }

    fn discharge_binop_concat(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Binop(Binop::Concat, lhs, rhs) = &self else {
            unreachable!("Should never be anything other than Binop");
        };

        match dst {
            local_exp @ ExpDesc::Local(local) => {
                let dst = u8::try_from(*local)?;

                lhs.discharge(local_exp, program, compile_context)?;

                let (_, stack_top) = compile_context.reserve_stack_top();
                rhs.discharge(&stack_top, program, compile_context)?;

                if program
                    .byte_codes
                    .last()
                    .filter(|bytecode| bytecode.get_opcode() == OpCode::Concat)
                    .is_some()
                {
                    let Some(concat) = program.byte_codes.pop() else {
                        unreachable!("Proto bytecodes should not be empty.");
                    };
                    let (_, count, _, _) = concat.decode_abck();
                    program.byte_codes.push(Bytecode::concat(dst, count + 1));
                } else {
                    program.byte_codes.push(Bytecode::concat(dst, 2));
                }

                compile_context.stack_top -= 1;

                Ok(())
            }
            other => {
                unreachable!(
                    "Can only discharge BinopConcat to Local, but was {:?}.",
                    other
                )
            }
        }
    }

    fn discharge_binop(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Binop(_, _, _) = &self else {
            unreachable!("Should never be anything other than Binop");
        };

        fn discharge_binop_constant(
            bytecode: fn(u8, u8, u8) -> Bytecode,
            lhs: &ExpDesc,
            constant: u8,
            dst: u8,
            program: &mut Proto,
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
            bytecode: fn(u8, u8, u8) -> Bytecode,
            lhs: &ExpDesc,
            rhs: &ExpDesc,
            dst: u8,
            test: bool,
            program: &mut Proto,
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
            program.byte_codes.push(Bytecode::jump(0));
            compile_context.jumps_to_false.push(jump);

            Ok(())
        }

        match (self, dst) {
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
                    program.byte_codes.push(Bytecode::test(dst, 0));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    compile_context.jumps_to_end.push(jump);
                }
                for jump in compile_context.jumps_to_block.drain(jump_to_block_count..) {
                    program.byte_codes[jump] = Bytecode::jump(
                        i32::try_from((after_lhs + 2) - jump - 1).map_err(|_| Error::LongJump)?,
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

                program.byte_codes.push(Bytecode::test(dst, 1));
                let jump = program.byte_codes.len();
                program.byte_codes.push(Bytecode::jump(0));
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
                        .push(Bytecode::equal_integer(lhs_loc, integer, 0));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
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

                    program.byte_codes.push(Bytecode::equal_constant(
                        lhs_loc,
                        u8::try_from(constant)?,
                        0,
                    ));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    compile_context.jumps_to_false.push(jump);
                } else {
                    unimplemented!("eq")
                }

                Ok(())
            }
            (Self::Binop(Binop::LessThan, lhs, rhs), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                discharge_relational_binop_into_local(
                    Bytecode::less_than,
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
                    .push(Bytecode::greater_than_integer(lhs_loc, integer, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(Bytecode::jump(0));
                compile_context.jumps_to_false.push(jump);

                Ok(())
            }
            (Self::Binop(Binop::GreaterThan, lhs, rhs), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                discharge_relational_binop_into_local(
                    Bytecode::less_than,
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
                    Bytecode::less_equal,
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
                    Bytecode::less_equal,
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
                    program.byte_codes[jump] = Bytecode::jump(
                        i32::try_from(after_lhs_cond - jump - 1).map_err(|_| Error::LongJump)?,
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

                program.byte_codes.push(Bytecode::equal_constant(
                    lhs_loc,
                    u8::try_from(constant)?,
                    *cond as u8,
                ));
                let jump = program.byte_codes.len();
                program.byte_codes.push(Bytecode::jump(0));
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
                    .push(Bytecode::less_than(rhs_loc, lhs_loc, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(Bytecode::jump(0));
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
                    .push(Bytecode::less_equal(lhs_loc, rhs_loc, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(Bytecode::jump(0));
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

                program.byte_codes.push(Bytecode::greater_equal_integer(
                    lhs_loc,
                    integer,
                    *cond as u8,
                ));
                let jump = program.byte_codes.len();
                program.byte_codes.push(Bytecode::jump(0));
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
                        Binop::Mul => Bytecode::mul_constant,
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

                discharge_binop_constant(
                    bytecode,
                    lhs,
                    u8::try_from(constant)?,
                    dst,
                    program,
                    compile_context,
                )
            }
            (Self::Binop(binop, lhs, rhs), Self::Local(dst))
                if binop.arithmetic_binary_operator() =>
            {
                let dst = u8::try_from(*dst)?;

                let bytecode = match binop {
                        Binop::Add => Bytecode::add,
                        Binop::Sub => Bytecode::sub,
                        Binop::Mul => Bytecode::mul,
                        Binop::Mod => Bytecode::mod_bytecode,
                        Binop::Pow => Bytecode::pow,
                        Binop::Div => Bytecode::div,
                        Binop::Idiv => Bytecode::idiv,
                        Binop::ShiftLeft => Bytecode::shift_left,
                        Binop::ShiftRight => Bytecode::shift_right,
                        Binop::BitAnd => Bytecode::bit_and,
                        Binop::BitOr => Bytecode::bit_or,
                        Binop::BitXor => Bytecode::bit_xor,
                        other => unreachable!("Match guard garantees this is an arithmetic binary operator, but was {:?}.", other),
                    };

                Self::discharge_generic_binop(bytecode, lhs, rhs, dst, program, compile_context)
            }
            (Self::Binop(binop, lhs, rhs), Self::Local(dst))
                if binop.relational_binary_operator() =>
            {
                let dst = u8::try_from(*dst)?;

                let (bytecode, lhs, rhs): (fn(u8,u8,u8) -> Bytecode, _ ,_) = match binop {
                        Binop::Equal => unimplemented!("eq"),
                        Binop::NotEqual => unimplemented!("neq"),
                        Binop::GreaterThan => (Bytecode::less_than, rhs, lhs),
                        Binop::LessEqual => (Bytecode::less_equal, lhs, rhs),
                        Binop::GreaterEqual => (Bytecode::less_equal, rhs, lhs),
                        other => unreachable!("Match guard garantees this is an relational binary operator, but was {:?}.", other),
                    };

                Self::discharge_generic_binop(bytecode, lhs, rhs, dst, program, compile_context)
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
                            program.byte_codes[jump] = Bytecode::jump(
                                i32::try_from(after_lhs_cond - jump - 1)
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
                            .push(Bytecode::less_equal(lhs_loc, rhs_loc, 1));
                    }
                    Binop::GreaterEqual => match (lhs.as_ref(), rhs.as_ref()) {
                        (lhs, ExpDesc::Integer(integer)) => {
                            let (lhs_loc, stack_top) = compile_context.reserve_stack_top();
                            lhs.discharge(&stack_top, program, compile_context)?;

                            if let Ok(integer) = i8::try_from(*integer) {
                                program
                                    .byte_codes
                                    .push(Bytecode::greater_equal_integer(lhs_loc, integer, 0));

                                compile_context.stack_top -= 1;
                            } else {
                                let (rhs_loc, stack_top) = compile_context.reserve_stack_top();
                                rhs.discharge(&stack_top, program, compile_context)?;

                                program
                                    .byte_codes
                                    .push(Bytecode::less_equal(rhs_loc, lhs_loc, 1));

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

                            program.byte_codes.push(Bytecode::equal_constant(
                                lhs_loc,
                                u8::try_from(string)?,
                                1,
                            ));

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
        program: &mut Proto,
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

                    program.byte_codes.push(Bytecode::move_bytecode(dst, src));

                    Ok(())
                }
            }
            Self::Global(key) => {
                let src = u8::try_from(*src)?;
                let key = u8::try_from(*key)?;

                program
                    .byte_codes
                    .push(Bytecode::set_uptable(0, key, src, 0));

                Ok(())
            }
            Self::Name(name) => {
                let dst = match compile_context.find_name(name) {
                    Some(local) => local,
                    None => {
                        let global = program.push_constant(*name)?;
                        Self::Global(usize::try_from(global)?)
                    }
                };

                self.discharge(&dst, program, compile_context)
            }
            Self::TableAccess {
                table,
                key,
                record: false,
            } => {
                let mut used_stack = 0;

                let table_loc = match table.as_ref() {
                    ExpDesc::Local(local) => u8::try_from(*local)?,
                    name @ ExpDesc::Name(_) => {
                        let (table_loc, stack_top) = compile_context.reserve_stack_top();
                        name.discharge(&stack_top, program, compile_context)?;
                        used_stack += 1;

                        table_loc
                    }
                    other => unreachable!("Can't discharge to {:?}.", other),
                };

                match key.as_ref() {
                    ExpDesc::String(key) => {
                        let constant = program.push_constant(*key)?;

                        program.byte_codes.push(Bytecode::set_field(
                            table_loc,
                            u8::try_from(constant)?,
                            u8::try_from(*src)?,
                            0,
                        ));
                    }
                    other => unimplemented!("Only String keys, but was {:?}.", other),
                }

                compile_context.stack_top -= used_stack;

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
                        program.byte_codes.push(Bytecode::set_field(
                            table_loc,
                            u8::try_from(constant)?,
                            u8::try_from(*src)?,
                            0,
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
                        program.byte_codes.push(Bytecode::set_field(
                            u8::try_from(*table_loc)?,
                            u8::try_from(constant)?,
                            u8::try_from(*src)?,
                            0,
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
                    .push(Bytecode::test(src, *condition as u8));

                Ok(())
            }
            other => unreachable!("Local can't be discharged in {:?}.", other),
        }
    }

    fn discharge_global(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Global(key) = &self else {
            unreachable!("Should never be anything other than Global");
        };

        match dst {
            Self::Local(dst) => {
                let key = u8::try_from(*key)?;
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(Bytecode::get_uptable(dst, 0, key));

                Ok(())
            }
            Self::Global(dst_key) => {
                let src_key = u8::try_from(*key)?;
                let dst_key = u8::try_from(*dst_key)?;

                let (stack_loc, _) = compile_context.reserve_stack_top();

                program
                    .byte_codes
                    .push(Bytecode::get_uptable(stack_loc, 0, src_key));
                program
                    .byte_codes
                    .push(Bytecode::set_uptable(0, dst_key, stack_loc, 0));

                compile_context.stack_top -= 1;

                Ok(())
            }
            Self::Condition(condition) => {
                let (dst, stack_top) = compile_context.reserve_stack_top();
                self.discharge(&stack_top, program, compile_context)?;

                program
                    .byte_codes
                    .push(Bytecode::test(dst, *condition as u8));

                compile_context.stack_top -= 1;

                Ok(())
            }
            other => unreachable!("Global can't be discharged in {:?}.", other),
        }
    }

    fn discharge_upvalue(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        _compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Upvalue(upvalue) = &self else {
            unreachable!("Should never be anything other than Table");
        };

        match dst {
            ExpDesc::Local(local) => {
                program.byte_codes.push(Bytecode::get_upvalue(
                    u8::try_from(*local)?,
                    u8::try_from(*upvalue)?,
                ));
                Ok(())
            }
            other => {
                unimplemented!("Can't discharge Upvalue to {:?}.", other)
            }
        }
    }

    fn discharge_table(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::Table(fields) = &self else {
            unreachable!("Should never be anything other than Table");
        };

        match dst {
            Self::Name(name) => match compile_context.find_name(name) {
                Some(local) => self.discharge(&local, program, compile_context),
                None => {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    let constant = program.push_constant(*name)?;

                    self.discharge(&stack_top, program, compile_context)?;
                    stack_top.discharge(
                        &ExpDesc::Global(usize::try_from(constant)?),
                        program,
                        compile_context,
                    )?;

                    compile_context.stack_top -= 1;

                    Ok(())
                }
            },
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                let last_field_is_variadic =
                    Some((TableKey::Array, ExpDesc::VariadicArguments)) == fields.last().cloned();

                let array_items = u8::try_from(
                    fields
                        .iter()
                        .filter(|(key, _)| matches!(key, TableKey::Array))
                        .count(),
                )?;
                let table_items = u8::try_from(fields.len())? - array_items;
                let adjusted_array_items = if last_field_is_variadic {
                    array_items - 1
                } else {
                    array_items
                };

                let table = dst;

                program.byte_codes.push(Bytecode::new_table(
                    dst,
                    table_items,
                    adjusted_array_items,
                ));

                for (i, (key, val)) in fields.iter().enumerate() {
                    match key {
                        TableKey::Array => {
                            let (_, stack_top) = compile_context.reserve_stack_top();
                            val.discharge(&stack_top, program, compile_context)?;
                        }
                        TableKey::Record(key) => {
                            val.discharge(
                                &ExpDesc::TableAccess {
                                    table: Box::new(ExpDesc::Local(usize::from(table))),
                                    key: key.clone(),
                                    record: true,
                                },
                                program,
                                compile_context,
                            )?;
                        }
                        TableKey::General(key) => {
                            val.discharge(
                                &ExpDesc::TableAccess {
                                    table: Box::new(ExpDesc::Local(usize::from(table))),
                                    key: key.clone(),
                                    record: false,
                                },
                                program,
                                compile_context,
                            )?;
                        }
                    }

                    if i != fields.len() - 1
                        && program
                            .byte_codes
                            .last_mut()
                            .filter(|bytecode| bytecode.get_opcode() == OpCode::VariadicArguments)
                            .is_some()
                    {
                        let Some(variadic) = program.byte_codes.pop() else {
                            unreachable!("Proto bytecodes should not be empty.");
                        };
                        let (register, _, _, _) = variadic.decode_abck();
                        program
                            .byte_codes
                            .push(Bytecode::variadic_arguments(register, 2));
                    }
                }

                if adjusted_array_items > 0 {
                    let count = if last_field_is_variadic {
                        0
                    } else {
                        adjusted_array_items
                    };

                    program.byte_codes.push(Bytecode::set_list(table, count, 0));
                }

                compile_context.stack_top -= array_items;

                Ok(())
            }
            table_exp @ ExpDesc::TableAccess {
                table: _,
                key: _,
                record: _,
            } => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                self.discharge(&stack_top, program, compile_context)?;

                stack_top.discharge(table_exp, program, compile_context)?;

                compile_context.stack_top -= 1;

                Ok(())
            }
            other => unreachable!("Table can't be discharged in {:?}.", other),
        }
    }

    fn discharge_general_table_access(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::TableAccess {
            table,
            key,
            record: false,
        } = &self
        else {
            unreachable!("Should never be anything other than general TableAccess");
        };

        match dst {
            Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                let table_loc =
                    table.get_local_or_discharge_at_location(program, dst, compile_context)?;

                match key.as_ref() {
                    ExpDesc::Integer(integer) => {
                        if let Ok(index) = u8::try_from(*integer) {
                            program
                                .byte_codes
                                .push(Bytecode::get_index(dst, table_loc, index));
                            // TODO proper conversion
                        } else if let Ok(index) = i32::try_from(*integer) {
                            program
                                .byte_codes
                                .push(Bytecode::load_integer(compile_context.stack_top, index));
                            program.byte_codes.push(Bytecode::get_table(
                                dst,
                                table_loc,
                                compile_context.stack_top,
                            ));
                        } else {
                            let constant = program.push_constant(*integer)?;
                            program.byte_codes.push(Bytecode::get_field(
                                dst,
                                table_loc,
                                u8::try_from(constant)?,
                            ));
                        }
                    }
                    ExpDesc::String(string) => {
                        let constant = program.push_constant(*string)?;
                        program.byte_codes.push(Bytecode::get_field(
                            dst,
                            table_loc,
                            u8::try_from(constant)?,
                        ));
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
            lhs @ Self::TableAccess {
                table: _,
                key: _,
                record: _,
            } => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                self.discharge(&stack_top, program, compile_context)?;

                stack_top.discharge(lhs, program, compile_context)?;
                compile_context.stack_top -= 1;

                Ok(())
            }

            other => unreachable!("General TableAccess can't be discharged in {:?}.", other),
        }
    }

    fn discharge_record_table_access(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::TableAccess {
            table,
            key,
            record: true,
        } = &self
        else {
            unreachable!("Should never be anything other than record TableAccess");
        };

        match dst {
            local @ Self::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                let table_loc = match table.as_ref() {
                    ExpDesc::Local(local) => u8::try_from(*local)?,
                    ExpDesc::Name(_) => {
                        table.get_local_or_discharge_at_location(program, dst, compile_context)?
                    }
                    table @ ExpDesc::TableAccess {
                        table: _,
                        key: _,
                        record: _,
                    } => {
                        table.discharge(local, program, compile_context)?;
                        dst
                    }
                    other => unreachable!(
                        "Table can only be located on Local or Name, but was {:?}.",
                        other
                    ),
                };

                match key.as_ref() {
                    ExpDesc::Name(name) => {
                        let constant = program.push_constant(*name)?;
                        program.byte_codes.push(Bytecode::get_field(
                            dst,
                            table_loc,
                            u8::try_from(constant)?,
                        ));

                        Ok(())
                    }
                    other => Err(Error::TableRecordAccess(other.static_type_name())),
                }
            }
            Self::TableAccess {
                table: dst_table,
                key: dst_key,
                record: _,
            } => {
                let (table_loc, stack_top) = compile_context.reserve_stack_top();
                dst_table.discharge(&stack_top, program, compile_context)?;

                match dst_key.as_ref() {
                    ExpDesc::Name(name) => {
                        let key = program.push_constant(*name)?;

                        let (src_loc, src_stack_top) = compile_context.reserve_stack_top();
                        self.discharge(&src_stack_top, program, compile_context)?;

                        program.byte_codes.push(Bytecode::set_field(
                            table_loc,
                            u8::try_from(key)?,
                            src_loc,
                            0,
                        ));

                        compile_context.stack_top -= 2;
                    }
                    ExpDesc::TableAccess {
                        table: _,
                        key: _,
                        record: _,
                    } => {
                        todo!();
                    }
                    other => unreachable!(
                        "Table key should always be Name or TableAccess, but was {:?}.",
                        other
                    ),
                }
                Ok(())
            }
            other => unreachable!("TableAccess can't be discharged in {:?}.", other),
        }
    }

    fn discharge_closure(&self, dst: &ExpDesc<'a>, program: &mut Proto) -> Result<(), Error> {
        let ExpDesc::Closure(index) = &self else {
            unreachable!("Should never be anything other than TableAccess");
        };

        match dst {
            ExpDesc::Local(dst) => {
                let dst = u8::try_from(*dst)?;

                program
                    .byte_codes
                    .push(Bytecode::closure(dst, u32::from(*index)));

                Ok(())
            }
            other => unreachable!("TableAccess can't be discharged in {:?}.", other),
        }
    }

    fn discharge_function_call(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
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

                for (i, arg) in args.iter().enumerate() {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    arg.discharge(&stack_top, program, compile_context)?;

                    if program
                        .byte_codes
                        .last_mut()
                        .filter(|bytecode| bytecode.get_opcode() == OpCode::VariadicArguments)
                        .is_some()
                    {
                        let Some(variadic) = program.byte_codes.pop() else {
                            unreachable!()
                        };
                        let (register, _, _, _) = variadic.decode_abck();
                        let count = if i == args.len() - 1 { 0 } else { 2 };
                        program
                            .byte_codes
                            .push(Bytecode::variadic_arguments(register, count));
                    }
                }

                if program
                    .byte_codes
                    .last_mut()
                    .filter(|bytecode| bytecode.get_opcode() == OpCode::Call)
                    .is_some()
                {
                    let Some(call) = program.byte_codes.pop() else {
                        unreachable!()
                    };
                    let (func_index, inputs, _, _) = call.decode_abck();
                    program
                        .byte_codes
                        .push(Bytecode::call(func_index, inputs, 0));
                }

                let after_args = program.byte_codes.len();

                for jump in compile_context.jumps_to_block.drain(jumps_to_block_count..) {
                    program.byte_codes[jump] = Bytecode::jump(
                        i32::try_from(after_args - jump - 1).map_err(|_| Error::LongJump)?,
                    );
                }
                for jump in compile_context.jumps_to_end.drain(jumps_to_end_count..) {
                    program.byte_codes[jump] = Bytecode::jump(
                        i32::try_from(after_args - jump - 1).map_err(|_| Error::LongJump)?,
                    );
                }
                if compile_context.jumps_to_false.len() > jumps_to_false_count {
                    let false_bytecode = program.byte_codes.len();
                    program
                        .byte_codes
                        .push(Bytecode::load_false_skip(func_index + 1));

                    let Some(last) = compile_context.jumps_to_false.pop() else {
                        unreachable!("Should always have at least one jump on the list.");
                    };
                    program.byte_codes[last] = Bytecode::jump(1);
                    if program.byte_codes[last - 1].get_opcode().is_relational() {
                        program.byte_codes[last - 1].flip_test();
                    }

                    program.byte_codes.push(Bytecode::load_true(func_index + 1));
                    for jump in compile_context.jumps_to_false.drain(jumps_to_false_count..) {
                        program.byte_codes[jump] = Bytecode::jump(
                            i32::try_from(false_bytecode - jump - 1)
                                .map_err(|_| Error::LongJump)?,
                        );
                    }
                }

                // TODO do proper bytecode
                let args_len = if let Some(ExpDesc::FunctionCall(_, _)) = args.last() {
                    0
                } else if program
                    .byte_codes
                    .last()
                    .filter(|bytecode| bytecode.get_opcode() == OpCode::VariadicArguments)
                    .is_some()
                {
                    0
                } else {
                    u8::try_from(args.len())? + 1
                };

                program
                    .byte_codes
                    .push(Bytecode::call(func_index, args_len, 1));

                compile_context.stack_top -= u8::try_from(args.len())?;

                Ok(())
            }
            ExpDesc::Global(global) => {
                let (local_loc, stack_top) = compile_context.reserve_stack_top();
                self.discharge(&stack_top, program, compile_context)?;

                program.byte_codes.push(Bytecode::set_uptable(
                    0,
                    u8::try_from(*global)?,
                    local_loc,
                    0,
                ));
                compile_context.stack_top -= 1;

                Ok(())
            }
            ExpDesc::Name(name) => {
                let name = match compile_context.find_name(name) {
                    Some(local) => local,
                    None => {
                        let global = program.push_constant(*name)?;
                        ExpDesc::Global(usize::try_from(global)?)
                    }
                };
                self.discharge(&name, program, compile_context)
            }
            other => unreachable!("FunctionCall can't be discharged in {:?}.", other),
        }
    }

    fn discharge_method_call(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let ExpDesc::MethodCall(prefix, name, args) = self else {
            unreachable!("discharge_method_call should never be called with anything other than ExpDesc::MethodCall, but was {:?}.", self);
        };

        let ExpDesc::Local(dst) = dst else {
            unreachable!(
                "Method call can only be discharged into Local, but was {:?}.",
                dst
            );
        };
        let dst_u8 = u8::try_from(*dst)?;

        prefix.discharge(&ExpDesc::Local(*dst), program, compile_context)?;

        let ExpDesc::Name(name) = name.as_ref() else {
            unreachable!("Method name should always be Name, but was {:?}.", name);
        };
        let constant = program.push_constant(*name)?;
        program.byte_codes.push(Bytecode::table_self(
            dst_u8,
            dst_u8,
            u8::try_from(constant)?,
        ));
        // TableSelf uses 2 locations on the stack
        compile_context.stack_top += 1;

        for arg in args.iter() {
            let (_, stack_top) = compile_context.reserve_stack_top();
            arg.discharge(&stack_top, program, compile_context)?;
        }

        program
            .byte_codes
            .push(Bytecode::call(dst_u8, u8::try_from(args.len() + 2)?, 1));

        compile_context.stack_top -= u8::try_from(args.len() + 1)?;

        Ok(())
    }

    fn discharge_variable_arguments(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Proto,
        _compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match dst {
            ExpDesc::Local(dst) => {
                let dst = u8::try_from(*dst)?;
                program
                    .byte_codes
                    .push(Bytecode::variadic_arguments(dst, 0));
            }
            _ => unimplemented!(
                "Discharging Variable arguments into {:?} is not yet supported.",
                dst
            ),
        }
        Ok(())
    }

    fn discharge_generic_binop(
        bytecode: fn(u8, u8, u8) -> Bytecode,
        lhs: &ExpDesc,
        rhs: &ExpDesc,
        dst: u8,
        program: &mut Proto,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        let (lhs_loc, used_dst) = if let name @ ExpDesc::Name(_) = lhs {
            let local = name.get_local_or_discharge_at_location(program, dst, compile_context)?;
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
        bytecode: fn(u8, u8, i8) -> Bytecode,
        lhs: &ExpDesc,
        integer: i8,
        dst: u8,
        program: &mut Proto,
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

    fn push_value_to_global(
        program: &mut Proto,
        key: usize,
        value: impl Into<Value>,
    ) -> Result<(), Error> {
        let key = u8::try_from(key)?;
        let constant = program.push_constant(value)?;

        program
            .byte_codes
            .push(Bytecode::set_uptable(0, key, u8::try_from(constant)?, 1));

        Ok(())
    }

    pub fn get_local_or_discharge_at_location(
        &self,
        program: &mut Proto,
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
            Self::Upvalue(_) => "upvalue",
            Self::Table(_) => "table",
            Self::TableAccess {
                table: _,
                key: _,
                record: _,
            } => "table_access",
            Self::Condition(_) => "condition",
            Self::Closure(_) => "closure",
            Self::FunctionCall(_, _) => "function_call",
            Self::MethodCall(_, _, _) => "method_call",
            Self::VariadicArguments => "variadic arguments",
        }
    }
}
