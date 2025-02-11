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
    RelationalOp(
        fn(u8, u8, u8) -> ByteCode,
        Box<ExpDesc<'a>>,
        Box<ExpDesc<'a>>,
    ),
    RelationalOpInteger(
        fn(u8, i8, u8) -> ByteCode,
        Box<ExpDesc<'a>>,
        Box<ExpDesc<'a>>,
    ),
    RelationalOpConstant(
        fn(u8, u8, u8) -> ByteCode,
        Box<ExpDesc<'a>>,
        Box<ExpDesc<'a>>,
    ),
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
            (Self::OrCondition(lhs, rhs), Self::Local(top_index)) => {
                lhs.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                program
                    .byte_codes
                    .push(ByteCode::Test(u8::try_from(*top_index)?, 1));
                let lhs_jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));

                compile_context.last_expdesc_was_or = true;
                rhs.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                let after_rhs = program.byte_codes.len() - 1;

                program.byte_codes[lhs_jump] = ByteCode::Jmp(i16::try_from(after_rhs - lhs_jump)?);

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

                compile_context.last_expdesc_was_or = matches!(
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
            (Self::AndCondition(lhs, rhs), Self::Local(top_index)) => {
                let top_index_u8 = u8::try_from(*top_index)?;

                lhs.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                let after_lhs = program.byte_codes.len() - 2;

                let maybe_jump = if !compile_context.last_expdesc_was_relational {
                    program.byte_codes.push(ByteCode::Test(top_index_u8, 0));
                    let lhs_jump = program.byte_codes.len();
                    program.byte_codes.push(ByteCode::Jmp(0));

                    if compile_context.last_expdesc_was_or {
                        program.byte_codes[after_lhs] =
                            ByteCode::Jmp(i16::try_from(lhs_jump - after_lhs)?);
                    }

                    Some(lhs_jump)
                } else {
                    None
                };

                compile_context.last_expdesc_was_or = false;
                compile_context.last_expdesc_was_relational = false;
                rhs.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                let after_rhs = program.byte_codes.len() - 1;

                if let Some(lhs_jump) = maybe_jump {
                    program.byte_codes[lhs_jump] =
                        ByteCode::Jmp(i16::try_from(after_rhs - lhs_jump)?);
                }

                Ok(())
            }
            (Self::AndCondition(lhs, rhs), Self::IfCondition(top_index, _)) => {
                lhs.discharge(
                    &ExpDesc::IfCondition(*top_index, false),
                    program,
                    compile_context,
                )?;
                if compile_context.last_expdesc_was_or {
                    let end_of_ors = program.byte_codes.len() - 1;
                    for jump in compile_context.jumps_to_block.drain(..) {
                        program.byte_codes[jump] = ByteCode::Jmp(i16::try_from(end_of_ors - jump)?);
                    }
                    program.invert_last_test();
                }

                compile_context.last_expdesc_was_or = false;
                compile_context.last_expdesc_was_relational = false;
                rhs.discharge(
                    &ExpDesc::IfCondition(*top_index, false),
                    program,
                    compile_context,
                )?;

                Ok(())
            }
            (ExpDesc::RelationalOp(bytecode, lhs, rhs), ExpDesc::Local(top_index)) => {
                let top_index_u8 = u8::try_from(*top_index)?;
                let (lhs, lhs_used_top_index) = match lhs.as_ref() {
                    ExpDesc::Local(lhs) => (*lhs, 0),
                    other => {
                        other.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                        (*top_index, 1)
                    }
                };
                let rhs = match rhs.as_ref() {
                    ExpDesc::Local(rhs) => *rhs,
                    ExpDesc::Integer(integer) => {
                        if let Ok(integer) = i16::try_from(*integer) {
                            program.byte_codes.push(ByteCode::LoadInt(
                                top_index_u8 + lhs_used_top_index,
                                integer,
                            ));
                            top_index + usize::from(lhs_used_top_index)
                        } else {
                            let constant = program.push_constant(*integer)?;
                            program.byte_codes.push(ByteCode::LoadConstant(
                                top_index_u8 + lhs_used_top_index,
                                constant,
                            ));
                            top_index + usize::from(lhs_used_top_index)
                        }
                    }
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;
                        program.byte_codes.push(ByteCode::LoadConstant(
                            top_index_u8 + lhs_used_top_index,
                            constant,
                        ));
                        top_index + usize::from(lhs_used_top_index)
                    }
                    other => {
                        other.discharge(
                            &ExpDesc::Local(top_index + usize::from(lhs_used_top_index)),
                            program,
                            compile_context,
                        )?;
                        top_index + usize::from(lhs_used_top_index)
                    }
                };

                program
                    .byte_codes
                    .push(bytecode(u8::try_from(lhs)?, u8::try_from(rhs)?, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jump_to_false.push(jump);

                compile_context.last_expdesc_was_relational = true;

                Ok(())
            }
            (
                ExpDesc::RelationalOp(bytecode, lhs, rhs),
                ExpDesc::IfCondition(top_index, test_cond),
            ) => {
                let top_index_u8 = u8::try_from(*top_index)?;
                let (lhs, lhs_used_top_index) = match lhs.as_ref() {
                    ExpDesc::Local(lhs) => (*lhs, 0),
                    other => {
                        other.discharge(
                            &ExpDesc::IfCondition(*top_index, *test_cond),
                            program,
                            compile_context,
                        )?;
                        (*top_index, 1)
                    }
                };
                let rhs = match rhs.as_ref() {
                    ExpDesc::Local(rhs) => *rhs,
                    ExpDesc::Integer(integer) => {
                        if let Ok(integer) = i16::try_from(*integer) {
                            program.byte_codes.push(ByteCode::LoadInt(
                                top_index_u8 + lhs_used_top_index,
                                integer,
                            ));
                            top_index + usize::from(lhs_used_top_index)
                        } else {
                            let constant = program.push_constant(*integer)?;
                            program.byte_codes.push(ByteCode::LoadConstant(
                                top_index_u8 + lhs_used_top_index,
                                constant,
                            ));
                            top_index + usize::from(lhs_used_top_index)
                        }
                    }
                    ExpDesc::String(string) => {
                        let string = string.unescape()?;
                        let constant = program.push_constant(string.as_str())?;
                        program.byte_codes.push(ByteCode::LoadConstant(
                            top_index_u8 + lhs_used_top_index,
                            constant,
                        ));
                        top_index + usize::from(lhs_used_top_index)
                    }
                    other => {
                        other.discharge(
                            &ExpDesc::Local(top_index + usize::from(lhs_used_top_index)),
                            program,
                            compile_context,
                        )?;
                        top_index + usize::from(lhs_used_top_index)
                    }
                };

                program.byte_codes.push(bytecode(
                    u8::try_from(lhs)?,
                    u8::try_from(rhs)?,
                    *test_cond as u8,
                ));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_end.push(jump);

                compile_context.last_expdesc_was_relational = true;

                Ok(())
            }
            (ExpDesc::RelationalOpInteger(bytecode, lhs, rhs), ExpDesc::Local(top_index)) => {
                let lhs = match lhs.as_ref() {
                    ExpDesc::Local(lhs) => *lhs,
                    other => {
                        other.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                        *top_index
                    }
                };

                if let ExpDesc::Integer(integer) = rhs.as_ref() {
                    if let Ok(integer) = i8::try_from(*integer) {
                        program
                            .byte_codes
                            .push(bytecode(u8::try_from(lhs)?, integer, 0));
                    } else {
                        let integer_const = program.push_constant(*integer)?;
                        let rhs = u8::try_from(top_index + 1)?;
                        program
                            .byte_codes
                            .push(ByteCode::LoadConstant(rhs, integer_const));
                        program
                            .byte_codes
                            .push(ByteCode::LessThan(u8::try_from(lhs)?, rhs, 0));
                    }
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(ByteCode::Jmp(0));
                    compile_context.jump_to_false.push(jump);

                    compile_context.last_expdesc_was_relational = true;

                    Ok(())
                } else {
                    panic!(
                        "Rhs must be integer on RelationalOpInteger, but was {:?}",
                        rhs
                    );
                }
            }
            (
                ExpDesc::RelationalOpInteger(bytecode, lhs, rhs),
                ExpDesc::IfCondition(top_index, test),
            ) => {
                let lhs = match lhs.as_ref() {
                    ExpDesc::Local(lhs) => *lhs,
                    other => {
                        other.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                        *top_index
                    }
                };

                if let ExpDesc::Integer(integer) = rhs.as_ref() {
                    if let Ok(integer) = i8::try_from(*integer) {
                        program
                            .byte_codes
                            .push(bytecode(u8::try_from(lhs)?, integer, *test as u8));
                    } else {
                        let integer_const = program.push_constant(*integer)?;
                        let rhs = u8::try_from(top_index + 1)?;
                        program
                            .byte_codes
                            .push(ByteCode::LoadConstant(rhs, integer_const));
                        program
                            .byte_codes
                            .push(ByteCode::LessThan(u8::try_from(lhs)?, rhs, 0));
                    }
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(ByteCode::Jmp(0));
                    compile_context.jumps_to_end.push(jump);

                    compile_context.last_expdesc_was_relational = true;

                    Ok(())
                } else {
                    panic!(
                        "Rhs must be integer on RelationalOpInteger, but was {:?}",
                        rhs
                    );
                }
            }
            (ExpDesc::RelationalOpConstant(bytecode, lhs, rhs), ExpDesc::Local(top_index)) => {
                let lhs = match lhs.as_ref() {
                    ExpDesc::Local(lhs) => *lhs,
                    other => {
                        other.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                        *top_index
                    }
                };

                let constant = match rhs.as_ref() {
                    ExpDesc::Integer(integer) => program.push_constant(*integer)?,
                    ExpDesc::Float(float) => program.push_constant(*float)?,
                    ExpDesc::String(string) => {
                        program.push_constant(string.unescape()?.as_str())?
                    }
                    _ => panic!("{:?} can't be used on RelationalOpConstant.", rhs),
                };
                program
                    .byte_codes
                    .push(bytecode(u8::try_from(lhs)?, constant, 0));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jump_to_false.push(jump);

                compile_context.last_expdesc_was_relational = true;

                Ok(())
            }
            (
                ExpDesc::RelationalOpConstant(bytecode, lhs, rhs),
                ExpDesc::IfCondition(top_index, test),
            ) => {
                let lhs = match lhs.as_ref() {
                    ExpDesc::Local(lhs) => *lhs,
                    other => {
                        other.discharge(&ExpDesc::Local(*top_index), program, compile_context)?;
                        *top_index
                    }
                };

                let constant = match rhs.as_ref() {
                    ExpDesc::Integer(integer) => program.push_constant(*integer)?,
                    ExpDesc::Float(float) => program.push_constant(*float)?,
                    ExpDesc::String(string) => {
                        program.push_constant(string.unescape()?.as_str())?
                    }
                    _ => panic!("{:?} can't be used on RelationalOpConstant.", rhs),
                };
                program
                    .byte_codes
                    .push(bytecode(u8::try_from(lhs)?, constant, *test as u8));
                let jump = program.byte_codes.len();
                program.byte_codes.push(ByteCode::Jmp(0));
                compile_context.jumps_to_end.push(jump);

                compile_context.last_expdesc_was_relational = true;

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
