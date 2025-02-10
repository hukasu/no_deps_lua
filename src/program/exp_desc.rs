use alloc::boxed::Box;

use crate::ext::{FloatExt, Unescape};

use super::{compile_context::CompileContext, ByteCode, Error, Program};

#[derive(Debug, Clone, PartialEq)]
pub enum ExpDesc<'a> {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    Unop(fn(u8, u8) -> ByteCode, usize),
    BinopConstant(fn(u8, u8, u8) -> ByteCode, usize, Box<ExpDesc<'a>>),
    Binop(fn(u8, u8, u8) -> ByteCode, usize, usize),
    Local(usize),
    Global(usize),
    TableLocal(usize, Box<ExpDesc<'a>>),
    TableGlobal(usize, Box<ExpDesc<'a>>),
    IfCondition(usize, bool),
    OrCondition(Box<ExpDesc<'a>>, Box<ExpDesc<'a>>),
    AndCondition(Box<ExpDesc<'a>>, Box<ExpDesc<'a>>),
}

impl<'a> ExpDesc<'a> {
    pub fn discharge(
        &self,
        dst: &ExpDesc<'a>,
        program: &mut Program,
        compile_context: &mut CompileContext,
    ) -> Result<(), Error> {
        match (&self, &dst) {
            (Self::Nil, Self::Local(dst)) => u8::try_from(*dst).map_err(Error::from).map(|dst| {
                program.byte_codes.push(ByteCode::LoadNil(dst));
            }),
            (Self::Nil, Self::Global(key)) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(())?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

                Ok(())
            }
            (Self::Boolean(boolean), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;

                if *boolean {
                    program.byte_codes.push(ByteCode::LoadTrue(dst));
                } else {
                    program.byte_codes.push(ByteCode::LoadFalse(dst));
                }

                Ok(())
            }
            (Self::Boolean(boolean), Self::Global(key)) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(*boolean)?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

                Ok(())
            }
            (Self::Integer(integer), Self::Local(dst)) => {
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
            (Self::Integer(integer), Self::Global(key)) => {
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
            (Self::Integer(integer), Self::TableGlobal(table, exp)) => {
                let table = u8::try_from(*table)?;

                match exp.as_ref() {
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;

                        if let Ok(i) = i16::try_from(*integer) {
                            program
                                .byte_codes
                                .push(ByteCode::GetGlobal(compile_context.stack_top, table));
                            program
                                .byte_codes
                                .push(ByteCode::LoadInt(compile_context.stack_top + 1, i));
                            program.byte_codes.push(ByteCode::SetField(
                                compile_context.stack_top,
                                constant,
                                compile_context.stack_top + 1,
                            ));

                            Ok(())
                        } else {
                            todo!()
                        }
                    }
                    ExpDesc::Local(local) => {
                        let local = u8::try_from(*local)?;

                        if let Ok(i) = i16::try_from(*integer) {
                            program
                                .byte_codes
                                .push(ByteCode::GetGlobal(compile_context.stack_top, table));
                            program
                                .byte_codes
                                .push(ByteCode::LoadInt(compile_context.stack_top + 1, i));
                            program.byte_codes.push(ByteCode::SetField(
                                compile_context.stack_top,
                                local,
                                compile_context.stack_top + 1,
                            ));

                            Ok(())
                        } else {
                            todo!()
                        }
                    }
                    _ => {
                        log::error!("Global table assign.");
                        Err(Error::Unimplemented)
                    }
                }
            }
            (Self::Float(float), Self::Local(dst)) => {
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
            (Self::Float(float), Self::Global(key)) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(*float)?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

                Ok(())
            }
            (Self::String(string), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                let string = string.unescape()?;
                let position = program.push_constant(string.as_str())?;

                program
                    .byte_codes
                    .push(ByteCode::LoadConstant(dst, position));

                Ok(())
            }
            (Self::String(string), Self::Global(key)) => {
                let key = u8::try_from(*key)?;
                let string = string.unescape()?;
                let constant = program.push_constant(string.as_str())?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

                Ok(())
            }
            (Self::Unop(bytecode, src), Self::Local(dst)) => {
                let src = u8::try_from(*src)?;
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(bytecode(dst, src));

                Ok(())
            }
            (Self::Binop(bytecode, lhs, rhs), Self::Local(dst)) => {
                let lhs = u8::try_from(*lhs)?;
                let rhs = u8::try_from(*rhs)?;
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(bytecode(dst, lhs, rhs));

                Ok(())
            }
            (Self::BinopConstant(bytecode, lhs, rhs), Self::Local(dst)) => {
                let lhs = u8::try_from(*lhs)?;
                let dst = u8::try_from(*dst)?;

                let rhs = match rhs.as_ref() {
                    ExpDesc::Nil => program.push_constant(()),
                    ExpDesc::Boolean(boolean) => program.push_constant(*boolean),
                    ExpDesc::Integer(integer) => program.push_constant(*integer),
                    ExpDesc::Float(float) => program.push_constant(*float),
                    _ => unreachable!("Only this values should be available for BinopConstant"),
                }?;

                program.byte_codes.push(bytecode(dst, lhs, rhs));

                Ok(())
            }
            (Self::Local(src), Self::Local(dst)) => {
                if src == dst {
                    Ok(())
                } else {
                    let src = u8::try_from(*src)?;
                    let dst = u8::try_from(*dst)?;

                    program.byte_codes.push(ByteCode::Move(dst, src));

                    Ok(())
                }
            }
            (Self::Local(src), Self::Global(key)) => {
                let src = u8::try_from(*src)?;
                let key = u8::try_from(*key)?;

                program.byte_codes.push(ByteCode::SetGlobal(key, src));

                Ok(())
            }
            (Self::Local(src), Self::TableLocal(table, exp)) => {
                let src = u8::try_from(*src)?;
                let table = u8::try_from(*table)?;

                match exp.as_ref() {
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;

                        program
                            .byte_codes
                            .push(ByteCode::SetField(table, constant, src));

                        Ok(())
                    }
                    _ => {
                        log::error!("Local({:?}) TableGlobal({:?}).", src, (table, exp));
                        Err(Error::Unimplemented)
                    }
                }
            }
            (Self::Local(src), Self::TableGlobal(table, exp)) => {
                let src = u8::try_from(*src)?;
                let table = u8::try_from(*table)?;

                match exp.as_ref() {
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;

                        program
                            .byte_codes
                            .push(ByteCode::GetGlobal(compile_context.stack_top, table));
                        program.byte_codes.push(ByteCode::SetField(
                            compile_context.stack_top,
                            constant,
                            src,
                        ));

                        Ok(())
                    }
                    _ => {
                        log::error!("Local({:?}) TableGlobal({:?}).", src, (table, exp));
                        Err(Error::Unimplemented)
                    }
                }
            }
            (Self::Local(src), Self::IfCondition(_, condition)) => {
                let src = u8::try_from(*src)?;
                program
                    .byte_codes
                    .push(ByteCode::Test(src, *condition as u8));

                compile_context.jumps_to_end.push(program.byte_codes.len());
                program.byte_codes.push(ByteCode::Jmp(0));

                Ok(())
            }
            (Self::Global(key), Self::Local(dst)) => {
                let key = u8::try_from(*key)?;
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(ByteCode::GetGlobal(dst, key));

                Ok(())
            }
            (Self::Global(src_key), Self::Global(dst_key)) => {
                let src_key = u8::try_from(*src_key)?;
                let dst_key = u8::try_from(*dst_key)?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalGlobal(dst_key, src_key));

                Ok(())
            }
            (Self::Global(key), Self::TableGlobal(table, exp)) => {
                let key = u8::try_from(*key)?;
                let table = u8::try_from(*table)?;

                match exp.as_ref() {
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;

                        program
                            .byte_codes
                            .push(ByteCode::GetGlobal(compile_context.stack_top, key));
                        program
                            .byte_codes
                            .push(ByteCode::GetGlobal(compile_context.stack_top + 1, table));
                        program.byte_codes.push(ByteCode::SetField(
                            compile_context.stack_top + 1,
                            constant,
                            compile_context.stack_top,
                        ));

                        Ok(())
                    }
                    _ => {
                        log::error!("Global({:?}) TableGlobal({:?}).", key, (table, exp));
                        Err(Error::Unimplemented)
                    }
                }
            }
            (src @ Self::Global(_), Self::IfCondition(dst, condition)) => {
                src.discharge(&ExpDesc::Local(*dst), program, compile_context)?;

                let dst = u8::try_from(*dst)?;
                program
                    .byte_codes
                    .push(ByteCode::Test(dst, *condition as u8));

                compile_context.jumps_to_end.push(program.byte_codes.len());
                program.byte_codes.push(ByteCode::Jmp(0));

                Ok(())
            }
            (Self::TableLocal(table, exp), Self::Local(dst)) => {
                let table = u8::try_from(*table)?;
                let dst = u8::try_from(*dst)?;

                match exp.as_ref() {
                    ExpDesc::Integer(integer) => {
                        if let Ok(i) = u8::try_from(*integer) {
                            program.byte_codes.push(ByteCode::GetInt(dst, table, i));
                            Ok(())
                        } else {
                            let constant = program.push_constant(*integer)?;

                            program
                                .byte_codes
                                .push(ByteCode::GetField(dst, table, constant));

                            Ok(())
                        }
                    }
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;

                        program
                            .byte_codes
                            .push(ByteCode::GetField(dst, table, constant));

                        Ok(())
                    }
                    ExpDesc::Local(src) => {
                        let src = u8::try_from(*src)?;

                        program.byte_codes.push(ByteCode::GetTable(dst, table, src));

                        Ok(())
                    }
                    _ => {
                        log::error!("TableLocal({:?}) Local({:?})", (table, exp), dst);
                        Err(Error::Unimplemented)
                    }
                }
            }
            (Self::TableGlobal(table, exp), Self::Local(dst)) => {
                let table = u8::try_from(*table)?;
                let dst = u8::try_from(*dst)?;

                match exp.as_ref() {
                    ExpDesc::Integer(integer) => {
                        if let Ok(i) = u8::try_from(*integer) {
                            program.byte_codes.push(ByteCode::GetGlobal(dst, table));
                            program.byte_codes.push(ByteCode::GetInt(dst, dst, i));
                            Ok(())
                        } else {
                            let constant = program.push_constant(*integer)?;

                            program.byte_codes.push(ByteCode::GetGlobal(dst, table));
                            program
                                .byte_codes
                                .push(ByteCode::GetField(dst, dst, constant));

                            Ok(())
                        }
                    }
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;

                        program.byte_codes.push(ByteCode::GetGlobal(dst, table));
                        program
                            .byte_codes
                            .push(ByteCode::GetField(dst, dst, constant));

                        Ok(())
                    }
                    ExpDesc::Local(src) => {
                        let src = u8::try_from(*src)?;

                        program.byte_codes.push(ByteCode::GetGlobal(dst, table));
                        program.byte_codes.push(ByteCode::GetTable(dst, dst, src));

                        Ok(())
                    }
                    _ => {
                        log::error!("TableGlobal({:?}) Local({:?})", (table, exp), dst);
                        Err(Error::Unimplemented)
                    }
                }
            }
            (src @ Self::TableGlobal(_, _), dst @ Self::TableGlobal(_, _)) => {
                let (_, dst_local) = compile_context.reserve_stack_top();

                src.discharge(&dst_local, program, compile_context)?;
                dst_local.discharge(dst, program, compile_context)?;

                compile_context.stack_top -= 1;

                Ok(())
            }
            (Self::OrCondition(lhs, rhs), Self::IfCondition(top_index, _)) => {
                let mut jump_cache = core::mem::take(&mut compile_context.jumps_to_end);

                lhs.discharge(
                    &ExpDesc::IfCondition(*top_index, true),
                    program,
                    compile_context,
                )?;
                match lhs.as_ref() {
                    ExpDesc::Local(_) | ExpDesc::Global(_) => {
                        assert_eq!(compile_context.jumps_to_end.len(), 1, "When OrCondition's `lhs` is a Local or Global, there should be only 1 jump.");
                        let Some(jump) = compile_context.jumps_to_end.pop() else {
                            unreachable!("OrCondition's `lhs` will always have 1 item.");
                        };
                        compile_context.jumps_to_block.push(jump);
                    }
                    _ => (),
                }

                compile_context.last_rhs_was_or = matches!(
                    rhs.as_ref(),
                    ExpDesc::Local(_) | ExpDesc::Global(_) | ExpDesc::OrCondition(_, _)
                );
                rhs.discharge(
                    &ExpDesc::IfCondition(*top_index, true),
                    program,
                    compile_context,
                )?;

                core::mem::swap(&mut compile_context.jumps_to_end, &mut jump_cache);
                compile_context.jumps_to_end.extend(jump_cache);

                Ok(())
            }
            (Self::AndCondition(lhs, rhs), Self::IfCondition(top_index, _)) => {
                lhs.discharge(
                    &ExpDesc::IfCondition(*top_index, false),
                    program,
                    compile_context,
                )?;
                if let ExpDesc::OrCondition(_, _) = lhs.as_ref() {
                    let end_of_ors = program.byte_codes.len() - 1;
                    for jump in compile_context.jumps_to_block.drain(..) {
                        program.byte_codes[jump] = ByteCode::Jmp(i16::try_from(end_of_ors - jump)?);
                    }
                }
                if compile_context.last_rhs_was_or {
                    program.invert_last_test();
                }

                if let ExpDesc::OrCondition(_, _) = rhs.as_ref() {
                    compile_context.last_rhs_was_or = true;
                } else {
                    compile_context.last_rhs_was_or = false;
                }
                rhs.discharge(
                    &ExpDesc::IfCondition(*top_index, false),
                    program,
                    compile_context,
                )?;

                Ok(())
            }
            _ => {
                log::error!(
                    "Unimplemented discharge between Src({:?}) and Dst({:?})",
                    self,
                    dst
                );
                Err(Error::Unimplemented)?
            }
        }
    }
}
