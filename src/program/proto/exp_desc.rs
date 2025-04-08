use alloc::{boxed::Box, vec::Vec};

use crate::{
    bytecode::{ext::BytecodeExt, OpCode},
    ext::Unescape,
};

use super::{
    binops::Binop,
    compile_context::CompileContext,
    helper_types::{TableFields, TableKey},
    Bytecode, Error, ExpList, Proto,
};

const SHORT_STRING_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq)]
pub enum ExpDesc<'a> {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    Name(&'a str),
    LongName(&'a str),
    Unop(fn(u8, u8) -> Bytecode, Box<ExpDesc<'a>>),
    Binop(Binop, Box<ExpDesc<'a>>, Box<ExpDesc<'a>>),
    Local(usize),
    Global(usize),
    Upvalue(usize),
    ExpList(Vec<ExpDesc<'a>>),
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
    /// A condition found on `if`s, `while`, `for`s, etc
    Condition {
        /// If `true` means a jump to after the block, or to the start if the conditional
        /// is of a `repeat`. `false` means a jump to the block, as a shortcircuit of an
        /// `or`.
        jump_to_end: bool,
        /// Tests if the expression of the condition is `true` or `false`.
        if_condition: bool,
    },
    Closure(usize),
    FunctionCall(Box<ExpDesc<'a>>, ExpList<'a>),
    MethodCall(Box<ExpDesc<'a>>, Box<ExpDesc<'a>>, ExpList<'a>),
    VariadicArguments,
}

impl<'a> ExpDesc<'a> {
    pub fn discharge(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        match self {
            Self::Name(_) => self.discharge_into_name(src, program, compile_context),
            Self::LongName(_) => self.discharge_into_long_name(src, program, compile_context),
            Self::Local(_) => self.discharge_into_local(src, program, compile_context),
            Self::Global(_) => self.discharge_into_global(src, program, compile_context),
            Self::Upvalue(_) => self.discharge_into_upvalue(src, program, compile_context),
            Self::ExpList(_) => self.discharge_into_explist(src, program, compile_context),
            Self::TableAccess {
                table: _,
                key: _,
                record: _,
            } => self.discharge_into_table_access(src, program, compile_context),
            Self::Condition {
                jump_to_end: _,
                if_condition: _,
            } => self.discharge_into_condition(src, program, compile_context),
            _ => {
                unimplemented!(
                    "Unimplemented discharge between Src({:?}) and Dst({:?})",
                    src,
                    self,
                );
            }
        }
    }

    fn discharge_into_name(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let Self::Name(name) = self else {
            unreachable!(
                "Destination of `discharge_into_name` must be `ExpDesc::Name`, but was {:?}.",
                self
            );
        };

        let name = Self::find_name(name, program, compile_context)?;
        name.discharge(src, program, compile_context)
    }

    fn discharge_into_long_name(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let Self::LongName(long_name) = self else {
            unreachable!(
                "Destination of `discharge_into_long_name` must be `ExpDesc::LongName`, but was {:?}.",
                self
            );
        };

        let env = program.push_upvalue("_ENV");

        let (_, env_top) = compile_context.reserve_stack_top();
        env_top.discharge(&Self::Upvalue(env), program, compile_context)?;

        let (_, key_top) = compile_context.reserve_stack_top();
        key_top.discharge(&Self::String(long_name), program, compile_context)?;

        let env_table = Self::TableAccess {
            table: Box::new(env_top),
            key: Box::new(key_top),
            record: false,
        };
        env_table.discharge(src, program, compile_context)?;

        compile_context.stack_top -= 2;

        Ok(())
    }

    fn discharge_into_local(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let ExpDesc::Local(dst) = &self else {
            unreachable!(
                "Destination of `discharge_into_local` must be `ExpDesc::Local`, but was {:?}.",
                self
            );
        };
        let dst = u8::try_from(*dst)?;

        match src {
            Self::Nil => {
                program.byte_codes.push(Bytecode::load_nil(dst, 0));
                Ok(())
            }
            Self::Boolean(boolean) => {
                let bytecode = if *boolean {
                    Bytecode::load_true(dst)
                } else {
                    Bytecode::load_false(dst)
                };
                program.byte_codes.push(bytecode);
                Ok(())
            }
            Self::Integer(integer) => {
                if let Some(integer) = integer.to_sbx() {
                    program
                        .byte_codes
                        .push(Bytecode::load_integer(dst, integer));
                } else {
                    let constant = program.push_constant(*integer)?;
                    program
                        .byte_codes
                        .push(Bytecode::load_constant(dst, constant));
                }
                Ok(())
            }
            Self::Float(float) => {
                if let Some(float) = float.to_sbx() {
                    program.byte_codes.push(Bytecode::load_float(dst, float));
                    Ok(())
                } else {
                    let constant = program.push_constant(*float)?;
                    program
                        .byte_codes
                        .push(Bytecode::load_constant(dst, constant));
                    Ok(())
                }
            }
            Self::String(string) => {
                let unescaped_string = string.unescape()?;
                let constant = program.push_constant(unescaped_string.as_str())?;
                program
                    .byte_codes
                    .push(Bytecode::load_constant(dst, constant));
                Ok(())
            }
            Self::Name(name) => {
                let name = Self::find_name(name, program, compile_context)?;
                self.discharge(&name, program, compile_context)
            }
            Self::LongName(long_name) => {
                // Reaching here already means that it is a global
                let env = program.push_upvalue("_ENV");

                self.discharge(&Self::Upvalue(env), program, compile_context)?;
                let (_, stack_top) = compile_context.reserve_stack_top();
                stack_top.discharge(&Self::String(long_name), program, compile_context)?;
                self.discharge(
                    &Self::TableAccess {
                        table: Box::new(self.clone()),
                        key: Box::new(stack_top),
                        record: false,
                    },
                    program,
                    compile_context,
                )?;
                compile_context.stack_top -= 1;

                Ok(())
            }
            Self::Unop(op, var) => match var.as_ref() {
                Self::Name(name) => {
                    let name = Self::find_name(name, program, compile_context)?;
                    self.discharge(&Self::Unop(*op, Box::new(name)), program, compile_context)
                }
                Self::Local(local) => {
                    program.byte_codes.push(op(dst, u8::try_from(*local)?));
                    Ok(())
                }
                global @ Self::Global(_) => {
                    self.discharge(global, program, compile_context)?;
                    self.discharge(
                        &Self::Unop(*op, Box::new(self.clone())),
                        program,
                        compile_context,
                    )
                }
                other => unimplemented!("Can't execute unary operation on {:?}.", other),
            },
            Self::Binop(op, lhs, rhs) => match (op, lhs.as_ref(), rhs.as_ref()) {
                (
                    Binop::Mul
                    | Binop::Mod
                    | Binop::Pow
                    | Binop::Div
                    | Binop::Idiv
                    | Binop::BitAnd
                    | Binop::BitOr
                    | Binop::BitXor
                    | Binop::ShiftLeft
                    | Binop::ShiftRight
                    | Binop::Or
                    | Binop::And
                    | Binop::LessThan
                    | Binop::GreaterThan
                    | Binop::LessEqual
                    | Binop::GreaterEqual
                    | Binop::Equal
                    | Binop::NotEqual,
                    lhs @ Self::Integer(_),
                    _,
                ) => {
                    self.discharge(lhs, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        program,
                        compile_context,
                    )
                }
                (_, Self::Name(name), _) => {
                    let name = Self::find_name(name, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(name), rhs.clone()),
                        program,
                        compile_context,
                    )
                }
                (_, _, Self::Name(name)) => {
                    let name = Self::find_name(name, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(name)),
                        program,
                        compile_context,
                    )
                }
                (_, upval @ Self::Upvalue(_), _) => {
                    self.discharge(upval, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        program,
                        compile_context,
                    )
                }
                // TODO expand to other `Binop`s
                (op, binop @ Self::Binop(Binop::Add, _, _), _) => {
                    self.discharge(binop, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        program,
                        compile_context,
                    )
                }
                (
                    op,
                    table_access @ Self::TableAccess {
                        table: _,
                        key: _,
                        record: _,
                    },
                    _,
                ) => {
                    self.discharge(table_access, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        program,
                        compile_context,
                    )
                }
                (
                    op,
                    Self::Local(_),
                    table_access @ Self::TableAccess {
                        table: _,
                        key: _,
                        record: _,
                    },
                ) => {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    stack_top.discharge(table_access, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(stack_top)),
                        program,
                        compile_context,
                    )?;
                    compile_context.stack_top -= 1;
                    Ok(())
                }
                (Binop::Add, Self::Integer(_), global @ Self::Global(_)) => {
                    self.discharge(global, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), lhs.clone()),
                        program,
                        compile_context,
                    )
                }
                (Binop::Add, Self::Local(lhs), Self::Integer(rhs)) => {
                    if let Ok(rhs) = i8::try_from(*rhs) {
                        program.byte_codes.push(Bytecode::add_integer(
                            dst,
                            u8::try_from(*lhs)?,
                            rhs,
                        ));
                        Ok(())
                    } else {
                        todo!()
                    }
                }
                (Binop::Add, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::add(
                        dst,
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                    ));
                    Ok(())
                }
                (Binop::Sub, Self::Local(lhs), Self::Integer(rhs)) => {
                    if let Ok(rhs) = i8::try_from(*rhs) {
                        program.byte_codes.push(Bytecode::add_integer(
                            dst,
                            u8::try_from(*lhs)?,
                            -rhs,
                        ));
                        Ok(())
                    } else {
                        todo!()
                    }
                }
                (Binop::Sub, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::sub(
                        dst,
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                    ));
                    Ok(())
                }
                (Binop::Mul, Self::Local(lhs), Self::Float(rhs)) => {
                    let rhs = program.push_constant(*rhs)?;
                    program.byte_codes.push(Bytecode::mul_constant(
                        dst,
                        u8::try_from(*lhs)?,
                        u8::try_from(rhs)?,
                    ));
                    Ok(())
                }
                (Binop::Div, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::div(
                        dst,
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                    ));
                    Ok(())
                }
                (Binop::ShiftLeft, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::shift_left(
                        dst,
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                    ));
                    Ok(())
                }
                (Binop::ShiftRight, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::shift_right(
                        dst,
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                    ));
                    Ok(())
                }
                (
                    Binop::Concat,
                    lhs @ (Self::Integer(_)
                    | Self::Float(_)
                    | Self::String(_)
                    | Self::Binop(_, _, _)),
                    _,
                ) => {
                    self.discharge(lhs, program, compile_context)?;
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    stack_top.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        program,
                        compile_context,
                    )?;
                    compile_context.stack_top -= 1;
                    Ok(())
                }
                (
                    Binop::Concat,
                    Self::Local(lhs),
                    rhs @ (Self::Integer(_) | Self::String(_) | Self::Local(_)),
                ) => {
                    self.discharge(rhs, program, compile_context)?;
                    program
                        .byte_codes
                        .push(Bytecode::concat(u8::try_from(*lhs)?, 2));

                    Ok(())
                }
                (Binop::Concat, Self::Local(lhs), rhs @ Self::Binop(Binop::Concat, _, _)) => {
                    self.discharge(rhs, program, compile_context)?;

                    let Some(last_bytecode) = program.byte_codes.last_mut() else {
                        unreachable!(
                            "Bytecodes should not be empty after discharging concatenation."
                        );
                    };
                    assert_eq!(last_bytecode.get_opcode(), OpCode::Concat);
                    let (_, b, _, _) = last_bytecode.decode_abck();
                    *last_bytecode = Bytecode::concat(u8::try_from(*lhs)?, b + 1);

                    Ok(())
                }
                (Binop::Or, lhs, rhs) => {
                    self.discharge(lhs, program, compile_context)?;
                    program.byte_codes.push(Bytecode::test(dst, 1));
                    let shortcircuit = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    compile_context.jumps_to_block.push(shortcircuit);

                    self.discharge(rhs, program, compile_context)
                }
                (
                    Binop::And,
                    lhs,
                    rhs @ Self::Binop(
                        Binop::LessThan
                        | Binop::GreaterThan
                        | Binop::LessEqual
                        | Binop::GreaterEqual
                        | Binop::Equal
                        | Binop::NotEqual,
                        _,
                        _,
                    ),
                ) => {
                    let jumps_to_block = compile_context.jumps_to_block.len();
                    let jumps_to_end = compile_context.jumps_to_end.len();

                    let lhs_cond = Self::Condition {
                        jump_to_end: false,
                        if_condition: false,
                    };
                    lhs_cond.discharge(lhs, program, compile_context)?;

                    let rhs_cond = Self::Condition {
                        jump_to_end: true,
                        if_condition: true,
                    };
                    rhs_cond.discharge(rhs, program, compile_context)?;

                    Self::resolve_jumps_to_block(jumps_to_block, program, compile_context)?;
                    program.byte_codes.push(Bytecode::load_false_skip(dst));

                    Self::resolve_jumps_to_end(jumps_to_end, program, compile_context)?;
                    program.byte_codes.push(Bytecode::load_true(dst));

                    Ok(())
                }
                (Binop::And, lhs, rhs) => {
                    let jumps_to_block = compile_context.jumps_to_block.len();

                    self.discharge(lhs, program, compile_context)?;
                    program.byte_codes.push(Bytecode::test(dst, 0));
                    let shortcircuit = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));

                    Self::resolve_jumps_to_block(jumps_to_block, program, compile_context)?;
                    compile_context.jumps_to_block.push(shortcircuit);

                    self.discharge(rhs, program, compile_context)
                }
                (Binop::LessThan, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::less_than(
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                        1,
                    ));
                    program.byte_codes.push(Bytecode::jump(1));
                    program.byte_codes.push(Bytecode::load_false_skip(dst));
                    program.byte_codes.push(Bytecode::load_true(dst));

                    Ok(())
                }
                (Binop::GreaterThan, Self::Local(lhs), Self::Integer(rhs)) => {
                    program.byte_codes.push(Bytecode::greater_than_integer(
                        u8::try_from(*lhs)?,
                        i8::try_from(*rhs)?,
                        1,
                    ));
                    program.byte_codes.push(Bytecode::jump(1));
                    program.byte_codes.push(Bytecode::load_false_skip(dst));
                    program.byte_codes.push(Bytecode::load_true(dst));

                    Ok(())
                }
                (Binop::GreaterThan, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::less_than(
                        u8::try_from(*rhs)?,
                        u8::try_from(*lhs)?,
                        1,
                    ));
                    program.byte_codes.push(Bytecode::jump(1));
                    program.byte_codes.push(Bytecode::load_false_skip(dst));
                    program.byte_codes.push(Bytecode::load_true(dst));

                    Ok(())
                }
                (Binop::LessEqual, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::less_equal(
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                        1,
                    ));
                    program.byte_codes.push(Bytecode::jump(1));
                    program.byte_codes.push(Bytecode::load_false_skip(dst));
                    program.byte_codes.push(Bytecode::load_true(dst));

                    Ok(())
                }
                (Binop::GreaterEqual, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::less_equal(
                        u8::try_from(*rhs)?,
                        u8::try_from(*lhs)?,
                        1,
                    ));
                    program.byte_codes.push(Bytecode::jump(1));
                    program.byte_codes.push(Bytecode::load_false_skip(dst));
                    program.byte_codes.push(Bytecode::load_true(dst));

                    Ok(())
                }
                (Binop::Equal, Self::Local(lhs), Self::String(rhs)) => {
                    let rhs = program.push_constant(*rhs)?;
                    program.byte_codes.push(Bytecode::equal_constant(
                        u8::try_from(*lhs)?,
                        u8::try_from(rhs)?,
                        1,
                    ));
                    program.byte_codes.push(Bytecode::jump(1));
                    program.byte_codes.push(Bytecode::load_false_skip(dst));
                    program.byte_codes.push(Bytecode::load_true(dst));

                    Ok(())
                }
                _ => unimplemented!("Can't discharge binary operation {:?}.", src),
            },
            Self::Local(local) => {
                let local = u8::try_from(*local)?;
                if local != dst {
                    program.byte_codes.push(Bytecode::move_bytecode(dst, local));
                }
                Ok(())
            }
            Self::Global(global) => {
                let env = program.push_upvalue("_ENV");
                program.byte_codes.push(Bytecode::get_uptable(
                    dst,
                    u8::try_from(env)?,
                    u8::try_from(*global)?,
                ));
                Ok(())
            }
            Self::Upvalue(upvalue) => {
                program
                    .byte_codes
                    .push(Bytecode::get_upvalue(dst, u8::try_from(*upvalue)?));
                Ok(())
            }
            Self::Table(fields) => {
                let array_count = fields
                    .iter()
                    .filter(|(field_key, _)| matches!(field_key, TableKey::Array))
                    .count();
                let last_array_field_is_variadic = fields
                    .iter()
                    .filter(|(field_key, _)| matches!(field_key, TableKey::Array))
                    .last()
                    .filter(|(_, field)| matches!(field, Self::VariadicArguments))
                    .is_some();

                program.byte_codes.push(Bytecode::new_table(
                    dst,
                    u8::try_from(fields.len() - array_count)?,
                    u8::try_from(array_count)? - (last_array_field_is_variadic as u8),
                ));

                let mut used_stack = 0;
                let mut last_variadic_bytecode = 0;

                for (key, field) in fields.iter() {
                    match key {
                        TableKey::Array => {
                            let (_, stack_top) = compile_context.reserve_stack_top();
                            used_stack += 1;
                            stack_top.discharge(field, program, compile_context)?;

                            let Some(last_bytecode) = program.byte_codes.last_mut() else {
                                unreachable!("Bytecodes should never be empty while discharging table fields.");
                            };
                            if last_bytecode.get_opcode() == OpCode::VariadicArguments {
                                let (a, _, _, _) = last_bytecode.decode_abck();
                                *last_bytecode = Bytecode::variadic_arguments(a, 2);
                                last_variadic_bytecode = program.byte_codes.len() - 1;
                            }
                        }
                        TableKey::General(key) => {
                            Self::TableAccess {
                                table: Box::new(Self::Local(usize::from(dst))),
                                key: key.clone(),
                                record: false,
                            }
                            .discharge(
                                field,
                                program,
                                compile_context,
                            )?;
                        }
                        TableKey::Record(key) => {
                            Self::TableAccess {
                                table: Box::new(Self::Local(usize::from(dst))),
                                key: key.clone(),
                                record: true,
                            }
                            .discharge(
                                field,
                                program,
                                compile_context,
                            )?;
                        }
                    }
                }

                let array_count = if last_array_field_is_variadic {
                    let (a, _, _, _) = program.byte_codes[last_variadic_bytecode].decode_abck();
                    program.byte_codes[last_variadic_bytecode] = Bytecode::variadic_arguments(a, 0);
                    Some(0)
                } else if array_count != 0 {
                    Some(u8::try_from(array_count)?)
                } else {
                    None
                };

                if let Some(array_count) = array_count {
                    program
                        .byte_codes
                        .push(Bytecode::set_list(dst, array_count, 0));
                }

                compile_context.stack_top -= used_stack;

                Ok(())
            }
            Self::TableAccess {
                table,
                key,
                record: false,
            } => match (table.as_ref(), key.as_ref()) {
                (Self::Name(table), _) => {
                    let table = Self::find_name(table, program, compile_context)?;
                    self.discharge(
                        &Self::TableAccess {
                            table: Box::new(table),
                            key: key.clone(),
                            record: false,
                        },
                        program,
                        compile_context,
                    )
                }
                (Self::Local(local_table), Self::Integer(index)) => {
                    if let Ok(index) = u8::try_from(*index) {
                        program.byte_codes.push(Bytecode::get_index(
                            dst,
                            u8::try_from(*local_table)?,
                            index,
                        ));
                        Ok(())
                    } else {
                        let (_, stack_top) = compile_context.reserve_stack_top();
                        stack_top.discharge(&Self::Integer(*index), program, compile_context)?;
                        self.discharge(
                            &Self::TableAccess {
                                table: table.clone(),
                                key: Box::new(stack_top),
                                record: false,
                            },
                            program,
                            compile_context,
                        )?;
                        compile_context.stack_top -= 1;
                        Ok(())
                    }
                }
                (Self::Local(table), Self::String(key)) => {
                    let key = program.push_constant(*key)?;
                    program.byte_codes.push(Bytecode::get_field(
                        dst,
                        u8::try_from(*table)?,
                        u8::try_from(key)?,
                    ));
                    Ok(())
                }
                (Self::Local(table), Self::Local(key)) => {
                    program.byte_codes.push(Bytecode::get_table(
                        dst,
                        u8::try_from(*table)?,
                        u8::try_from(*key)?,
                    ));
                    Ok(())
                }
                (table @ Self::Global(_), _) => {
                    self.discharge(table, program, compile_context)?;
                    let table_access = Self::TableAccess {
                        table: Box::new(self.clone()),
                        key: key.clone(),
                        record: false,
                    };
                    self.discharge(&table_access, program, compile_context)
                }
                (
                    table @ Self::TableAccess {
                        table: _,
                        key: _,
                        record: _,
                    },
                    _,
                ) => {
                    self.discharge(table, program, compile_context)?;
                    let table_access = Self::TableAccess {
                        table: Box::new(self.clone()),
                        key: key.clone(),
                        record: false,
                    };
                    self.discharge(&table_access, program, compile_context)
                }
                _ => unimplemented!("Can't access table with configuration {:?}.", src),
            },
            Self::TableAccess {
                table,
                key,
                record: true,
            } => {
                let Self::Name(key) = key.as_ref() else {
                    unreachable!(
                        "Record table access can only be keyd by name, but was {:?}.",
                        key
                    );
                };

                self.discharge(
                    &Self::TableAccess {
                        table: table.clone(),
                        key: Box::new(Self::String(key)),
                        record: false,
                    },
                    program,
                    compile_context,
                )
            }
            Self::Closure(closure) => {
                program
                    .byte_codes
                    .push(Bytecode::closure(dst, u32::try_from(*closure)?));
                Ok(())
            }
            Self::FunctionCall(function, args) => {
                self.discharge(function, program, compile_context)?;

                let jumps_to_block = compile_context.jumps_to_block.len();
                for arg in args.iter() {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    stack_top.discharge(arg, program, compile_context)?;

                    let Some(last_bytecode) = program.byte_codes.last_mut() else {
                        unreachable!("Bytecodes should not be empty after discharging argument,");
                    };
                    if last_bytecode.get_opcode() == OpCode::VariadicArguments {
                        let (a, _, _, _) = last_bytecode.decode_abck();
                        *last_bytecode = Bytecode::variadic_arguments(a, 2);
                    }
                }
                compile_context.stack_top -= u8::try_from(args.len())?;

                let Some(last_bytecode) = program.byte_codes.last_mut() else {
                    unreachable!("Bytecodes should not be empty after discharging argument,");
                };
                let in_params = match last_bytecode.get_opcode() {
                    OpCode::Call => {
                        let (func, in_params, _, _) = last_bytecode.decode_abck();
                        *last_bytecode = Bytecode::call(func, in_params, 0);
                        0
                    }
                    OpCode::VariadicArguments => {
                        let (a, _, _, _) = last_bytecode.decode_abck();
                        *last_bytecode = Bytecode::variadic_arguments(a, 0);
                        0
                    }
                    _ => u8::try_from(args.len())? + 1,
                };

                Self::resolve_jumps_to_block(jumps_to_block, program, compile_context)?;

                program.byte_codes.push(Bytecode::call(dst, in_params, 1));

                Ok(())
            }
            Self::MethodCall(table, method_name, exp_list) => {
                self.discharge(table, program, compile_context)?;

                let Self::Name(name) = method_name.as_ref() else {
                    unreachable!("Method name should be a Name, but was {:?}.", method_name);
                };
                let constant = program.push_constant(*name)?;
                program
                    .byte_codes
                    .push(Bytecode::table_self(dst, dst, u8::try_from(constant)?));

                // reserve `self`
                let (_, _) = compile_context.reserve_stack_top();
                let mut used_stack = 1;

                for exp in exp_list.iter() {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    stack_top.discharge(exp, program, compile_context)?;
                    used_stack += 1;
                }

                compile_context.stack_top -= used_stack;

                program
                    .byte_codes
                    .push(Bytecode::call(dst, used_stack + 1, 1));

                Ok(())
            }
            Self::VariadicArguments => {
                program
                    .byte_codes
                    .push(Bytecode::variadic_arguments(dst, 0));
                Ok(())
            }
            other => unreachable!("{:?} can't be discharged into Local.", other),
        }
    }

    fn discharge_into_global(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let Self::Global(global) = self else {
            unreachable!(
                "Destination of `discharge_into_global` must be `ExpDesc::Global`, but was {:?}.",
                self
            );
        };
        let global = u8::try_from(*global)?;

        match src {
            Self::Integer(integer) => {
                let env = program.push_upvalue("_ENV");
                let constant = program.push_constant(*integer)?;
                program.byte_codes.push(Bytecode::set_uptable(
                    u8::try_from(env)?,
                    global,
                    u8::try_from(constant)?,
                    1,
                ));
                Ok(())
            }
            Self::String(string) => {
                let env = program.push_upvalue("_ENV");
                let constant = program.push_constant(*string)?;
                program.byte_codes.push(Bytecode::set_uptable(
                    u8::try_from(env)?,
                    global,
                    u8::try_from(constant)?,
                    1,
                ));
                Ok(())
            }
            Self::Name(name) => {
                let name = Self::find_name(name, program, compile_context)?;
                self.discharge(&name, program, compile_context)
            }
            Self::Local(local) => {
                let env = program.push_upvalue("_ENV");
                program.byte_codes.push(Bytecode::set_uptable(
                    u8::try_from(env)?,
                    global,
                    u8::try_from(*local)?,
                    0,
                ));
                Ok(())
            }
            exp @ (Self::Global(_)
            | Self::Upvalue(_)
            | Self::Table(_)
            | Self::FunctionCall(_, _)) => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                stack_top.discharge(exp, program, compile_context)?;
                self.discharge(&stack_top, program, compile_context)?;
                compile_context.stack_top -= 1;

                Ok(())
            }
            other => unreachable!("Can't discharge {:?} into Global.", other),
        }
    }

    fn discharge_into_upvalue(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let Self::Upvalue(upvalue) = self else {
            unreachable!(
                "Destination of `discharge_into_upvalue` must be `ExpDesc::Upvalue`, but was {:?}.",
                self
            );
        };

        if let Self::Local(local) = src {
            program.byte_codes.push(Bytecode::set_upvalue(
                u8::try_from(*upvalue)?,
                u8::try_from(*local)?,
            ));
        } else {
            let (stack_loc, stack_top) = compile_context.reserve_stack_top();
            stack_top.discharge(src, program, compile_context)?;
            program
                .byte_codes
                .push(Bytecode::set_upvalue(u8::try_from(*upvalue)?, stack_loc));
            compile_context.stack_top -= 1;
        }
        Ok(())
    }

    fn discharge_into_explist(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let Self::ExpList(explist) = self else {
            unreachable!(
                "Destination of `discharge_into_explist` must be `ExpDesc::Explist`, but was {:?}.",
                self
            );
        };

        match src {
            Self::ExpList(src_explist) => {
                if explist.len() == 1 && src_explist.len() == 1 {
                    explist[0].discharge(&src_explist[0], program, compile_context)
                } else {
                    for lhs_exp in explist.iter() {
                        if let Self::Name(name) = lhs_exp {
                            match Self::find_name(name, program, compile_context)? {
                                Self::Local(_) => (),
                                Self::Global(_) => {
                                    program.push_constant(*name)?;
                                }
                                Self::Upvalue(_) => (),
                                _ => unreachable!(
                                    "ExpDesc::find_name can only return Local, Global, or Upvalue."
                                ),
                            }
                        }
                    }

                    let mut used_stack = 0;
                    let mut reverse_sets = Vec::new();

                    let mut first = true;
                    for (dst, src) in explist.iter().zip(src_explist.iter()) {
                        match dst {
                            ExpDesc::Name(name) => {
                                let dst_name = Self::find_name(name, program, compile_context)?;
                                let src = if let Self::Name(src) = src {
                                    Self::find_name(src, program, compile_context)?
                                } else {
                                    src.clone()
                                };

                                if  first || matches!(src, Self::Upvalue(_) | Self::FunctionCall(_, _)) || matches!(dst_name, Self::Upvalue(_))
                                {
                                    let (_, stack_top) = compile_context.reserve_stack_top();
                                    stack_top.discharge(&src, program, compile_context)?;
                                    reverse_sets.push((dst_name.clone(), stack_top));
                                    used_stack += 1;
                                } else {
                                    dst_name.discharge(&src, program, compile_context)?;
                                }
                            }
                            ExpDesc::TableAccess {
                                table: _,
                                key: _,
                                record: _,
                            } => dst.discharge(src, program, compile_context)?,
                            _ => unreachable!("Varlist expressions should always be Name or TableAccess, but was {:?}.", dst),
                        }
                        first = false;
                    }

                    if let Some(ExpDesc::FunctionCall(_, _)) = src_explist.last() {
                        for remaining in explist[src_explist.len()..].iter() {
                            if let Self::Name(_) = remaining {
                                let (_, stack_top) = compile_context.reserve_stack_top();
                                reverse_sets.push((remaining.clone(), stack_top));
                                used_stack += 1;
                            }
                        }

                        let Some(last_bytecode) = program.byte_codes.last_mut() else {
                            unreachable!("Bytecodes should not be empty while discharging.");
                        };
                        assert_eq!(last_bytecode.get_opcode(), OpCode::Call);

                        let (function, in_params, _, _) = last_bytecode.decode_abck();
                        *last_bytecode = Bytecode::call(
                            function,
                            in_params,
                            u8::try_from(explist.len() - src_explist.len() + 2)?,
                        );
                    } else {
                        for dst in explist[src_explist.len()..].iter() {
                            if matches!(dst, Self::Global(_)) {
                                let (_, stack_top) = compile_context.reserve_stack_top();
                                stack_top.discharge(&ExpDesc::Nil, program, compile_context)?;
                                reverse_sets.push((dst.clone(), stack_top));
                                used_stack += 1;
                            } else {
                                dst.discharge(&ExpDesc::Nil, program, compile_context)?;
                            }
                        }
                    }

                    for (dst, src) in reverse_sets.into_iter().rev() {
                        dst.discharge(&src, program, compile_context)?;
                    }

                    compile_context.stack_top -= used_stack;
                    Ok(())
                }
            }
            _ => unimplemented!(
                "Can only discharge into an explist another explist, but was {:?}.",
                src
            ),
        }
    }

    fn discharge_into_table_access(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let Self::TableAccess { table, key, record } = self else {
            unreachable!(
                "Destination of `discharge_into_table_access` must be `ExpDesc::TableAccess`, but was {:?}.",
                self
            );
        };

        match (table.as_ref(), key.as_ref(), record, src) {
            (_, Self::Name(key), true, _) => {
                // Rewrite all access in the form `t.x` as `t["x"]`
                let table_access = Self::TableAccess {
                    table: table.clone(),
                    key: Box::new(ExpDesc::String(key)),
                    record: false,
                };
                table_access.discharge(src, program, compile_context)
            }
            (_, key, true, _) => {
                unreachable!(
                    "Record table access must be keyd by Name, but was {:?}.",
                    key
                );
            }
            (table @ Self::Global(_), _, false, _) => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                stack_top.discharge(table, program, compile_context)?;
                let table_access = Self::TableAccess {
                    table: Box::new(stack_top),
                    key: key.clone(),
                    record: false,
                };
                table_access.discharge(src, program, compile_context)?;
                compile_context.stack_top -= 1;

                Ok(())
            }
            (Self::Name(table), _, false, _) => {
                let table = Self::find_name(table, program, compile_context)?;
                let table_access = Self::TableAccess {
                    table: Box::new(table),
                    key: key.clone(),
                    record: false,
                };
                table_access.discharge(src, program, compile_context)
            }
            (_, Self::Name(key), false, _) => {
                let name = Self::find_name(key, program, compile_context)?;

                let table_access = Self::TableAccess {
                    table: table.clone(),
                    key: Box::new(name),
                    record: false,
                };
                table_access.discharge(src, program, compile_context)?;

                Ok(())
            }
            (_, Self::String(key), false, Self::Name(name)) => {
                // Storing the key into constants early to match the ordering
                // of the official compiler
                let _ = program.push_constant(*key)?;
                let name = Self::find_name(name, program, compile_context)?;
                self.discharge(&name, program, compile_context)
            }
            (_, _, false, Self::Name(name)) => {
                let name = Self::find_name(name, program, compile_context)?;
                self.discharge(&name, program, compile_context)
            }
            // local t, k
            // t[k] = 1
            (Self::Local(table), Self::Local(key), false, Self::Integer(integer)) => {
                let constant = program.push_constant(*integer)?;
                program.byte_codes.push(Bytecode::set_table(
                    u8::try_from(*table)?,
                    u8::try_from(*key)?,
                    u8::try_from(constant)?,
                    1,
                ));
                Ok(())
            }
            // local t, k
            // t[k] = "a"
            (Self::Local(table), Self::Local(key), false, Self::String(string)) => {
                let constant = program.push_constant(*string)?;
                program.byte_codes.push(Bytecode::set_table(
                    u8::try_from(*table)?,
                    u8::try_from(*key)?,
                    u8::try_from(constant)?,
                    1,
                ));
                Ok(())
            }
            // local t
            // t["x"] = 1
            (Self::Local(table), Self::String(key), false, Self::Integer(integer)) => {
                let key_constant = program.push_constant(*key)?;
                let constant = program.push_constant(*integer)?;
                program.byte_codes.push(Bytecode::set_field(
                    u8::try_from(*table)?,
                    u8::try_from(key_constant)?,
                    u8::try_from(constant)?,
                    1,
                ));
                Ok(())
            }
            // local t
            // t["x"] = "y"
            (Self::Local(table), Self::String(key), false, Self::String(string)) => {
                let key_constant = program.push_constant(*key)?;
                let constant = program.push_constant(*string)?;
                program.byte_codes.push(Bytecode::set_field(
                    u8::try_from(*table)?,
                    u8::try_from(key_constant)?,
                    u8::try_from(constant)?,
                    1,
                ));
                Ok(())
            }
            // local t, a
            // t["x"] = a
            (Self::Local(table), Self::String(key), false, Self::Local(src)) => {
                let key_constant = program.push_constant(*key)?;
                program.byte_codes.push(Bytecode::set_field(
                    u8::try_from(*table)?,
                    u8::try_from(key_constant)?,
                    u8::try_from(*src)?,
                    0,
                ));
                Ok(())
            }
            // local t
            // t["x"] = a
            (_, _, false, global @ Self::Global(_)) => {
                let (_, stack_top) = compile_context.reserve_stack_top();

                stack_top.discharge(global, program, compile_context)?;
                self.discharge(&stack_top, program, compile_context)?;

                compile_context.stack_top -= 1;
                Ok(())
            }
            (_, _, false, table @ Self::Table(_)) => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                stack_top.discharge(table, program, compile_context)?;
                self.discharge(&stack_top, program, compile_context)?;
                compile_context.stack_top -= 1;

                Ok(())
            }
            (
                _,
                _,
                false,
                table_access @ Self::TableAccess {
                    table: _,
                    key: _,
                    record: _,
                },
            ) => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                stack_top.discharge(table_access, program, compile_context)?;
                self.discharge(&stack_top, program, compile_context)?;
                compile_context.stack_top -= 1;

                Ok(())
            }
            _ => unimplemented!("Can't discharge {:?} into {:?}", src, self),
        }
    }

    fn discharge_into_condition(
        &self,
        src: &ExpDesc<'a>,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let Self::Condition {
            jump_to_end,
            if_condition,
        } = self
        else {
            unreachable!(
                "Destination of `discharge_into_table_access` must be `ExpDesc::TableAccess`, but was {:?}.",
                self
            );
        };

        match src {
            Self::Name(name) => {
                let name = Self::find_name(name, program, compile_context)?;
                self.discharge(&name, program, compile_context)
            }
            Self::Binop(Binop::Or, lhs, rhs) => {
                Self::Condition {
                    jump_to_end: false,
                    if_condition: true,
                }
                .discharge(lhs, program, compile_context)?;
                Self::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(rhs, program, compile_context)?;
                Ok(())
            }
            Self::Binop(Binop::And, lhs, rhs) => {
                let jumps_to_block = compile_context.jumps_to_block.len();
                Self::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(lhs, program, compile_context)?;

                Self::resolve_jumps_to_block(jumps_to_block, program, compile_context)?;

                Self::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(rhs, program, compile_context)?;
                Ok(())
            }
            Self::Binop(
                op @ (Binop::LessThan
                | Binop::GreaterThan
                | Binop::LessEqual
                | Binop::GreaterEqual
                | Binop::Equal
                | Binop::NotEqual),
                lhs,
                rhs,
            ) => match (op, lhs.as_ref(), rhs.as_ref()) {
                (_, Self::Name(name), _) => {
                    let name = Self::find_name(name, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(name), rhs.clone()),
                        program,
                        compile_context,
                    )
                }
                (_, _, Self::Name(name)) => {
                    let name = Self::find_name(name, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(name)),
                        program,
                        compile_context,
                    )
                }
                (Binop::GreaterThan, Self::Local(local), Self::Integer(integer)) => {
                    if let Ok(integer) = i8::try_from(*integer) {
                        program.byte_codes.push(Bytecode::greater_equal_integer(
                            u8::try_from(*local)?,
                            integer,
                            *if_condition as u8,
                        ));

                        let jump = program.byte_codes.len();
                        program.byte_codes.push(Bytecode::jump(0));
                        if *jump_to_end {
                            compile_context.jumps_to_end.push(jump);
                        } else {
                            compile_context.jumps_to_block.push(jump);
                        }

                        Ok(())
                    } else {
                        let (_, stack_top) = compile_context.reserve_stack_top();
                        stack_top.discharge(rhs.as_ref(), program, compile_context)?;
                        self.discharge(
                            &Self::Binop(*op, lhs.clone(), Box::new(stack_top)),
                            program,
                            compile_context,
                        )?;
                        compile_context.stack_top -= 1;
                        Ok(())
                    }
                }
                (Binop::GreaterThan, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::less_than(
                        u8::try_from(*rhs)?,
                        u8::try_from(*lhs)?,
                        *if_condition as u8,
                    ));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    if *jump_to_end {
                        compile_context.jumps_to_end.push(jump);
                    } else {
                        compile_context.jumps_to_block.push(jump);
                    }

                    Ok(())
                }
                (Binop::LessEqual, Self::Local(_), string @ Self::String(_)) => {
                    let (_, stack_top) = compile_context.reserve_stack_top();
                    stack_top.discharge(string, program, compile_context)?;
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(stack_top)),
                        program,
                        compile_context,
                    )?;
                    compile_context.stack_top -= 1;
                    Ok(())
                }
                (Binop::LessEqual, Self::Local(lhs), Self::Local(rhs)) => {
                    program.byte_codes.push(Bytecode::less_equal(
                        u8::try_from(*lhs)?,
                        u8::try_from(*rhs)?,
                        *if_condition as u8,
                    ));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    if *jump_to_end {
                        compile_context.jumps_to_end.push(jump);
                    } else {
                        compile_context.jumps_to_block.push(jump);
                    }

                    Ok(())
                }
                (Binop::GreaterEqual, Self::Local(lhs), Self::Integer(integer)) => {
                    program.byte_codes.push(Bytecode::greater_equal_integer(
                        u8::try_from(*lhs)?,
                        i8::try_from(*integer)?,
                        *if_condition as u8,
                    ));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    if *jump_to_end {
                        compile_context.jumps_to_end.push(jump);
                    } else {
                        compile_context.jumps_to_block.push(jump);
                    }

                    Ok(())
                }
                (Binop::Equal, Self::Local(lhs), Self::String(name)) => {
                    let constant = program.push_constant(*name)?;
                    program.byte_codes.push(Bytecode::equal_constant(
                        u8::try_from(*lhs)?,
                        u8::try_from(constant)?,
                        *if_condition as u8,
                    ));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    if *jump_to_end {
                        compile_context.jumps_to_end.push(jump);
                    } else {
                        compile_context.jumps_to_block.push(jump);
                    }

                    Ok(())
                }
                (Binop::Equal, Self::Local(lhs), Self::Integer(integer)) => {
                    let constant = program.push_constant(*integer)?;
                    program.byte_codes.push(Bytecode::equal_constant(
                        u8::try_from(*lhs)?,
                        u8::try_from(constant)?,
                        *if_condition as u8,
                    ));
                    let jump = program.byte_codes.len();
                    program.byte_codes.push(Bytecode::jump(0));
                    if *jump_to_end {
                        compile_context.jumps_to_end.push(jump);
                    } else {
                        compile_context.jumps_to_block.push(jump);
                    }

                    Ok(())
                }
                _ => unimplemented!("Can't discharge binary operation {:?}.", src),
            },
            Self::Local(local) => {
                program
                    .byte_codes
                    .push(Bytecode::test(u8::try_from(*local)?, *if_condition as u8));
                let jump = program.byte_codes.len();
                program.byte_codes.push(Bytecode::jump(0));
                if *jump_to_end {
                    compile_context.jumps_to_end.push(jump);
                } else {
                    compile_context.jumps_to_block.push(jump);
                }
                Ok(())
            }
            global @ Self::Global(_) => {
                let (_, stack_top) = compile_context.reserve_stack_top();
                stack_top.discharge(global, program, compile_context)?;
                self.discharge(&stack_top, program, compile_context)?;
                compile_context.stack_top -= 1;
                Ok(())
            }
            _ => unimplemented!("Can't make condition out of {:?}.", src),
        }
    }

    fn find_name(
        name: &'a str,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<ExpDesc<'a>, Error> {
        match compile_context.find_name(name) {
            Some(local) => Ok(ExpDesc::Local(local)),
            None => {
                if compile_context.exists_in_upvalue(name) {
                    let upvalue = program.push_upvalue(name);
                    Ok(Self::Upvalue(upvalue))
                } else if name.len() > SHORT_STRING_LEN {
                    Ok(Self::LongName(name))
                } else {
                    Ok(Self::Global(usize::try_from(program.push_constant(name)?)?))
                }
            }
        }
    }

    pub fn get_local_or_discharge_at_location(
        &self,
        program: &mut Proto,
        location: u8,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<u8, Error> {
        match self {
            ExpDesc::Name(table_name) => match compile_context.find_name(table_name) {
                Some(pos) => u8::try_from(pos).map_err(Error::from),
                None => {
                    self.discharge(
                        &ExpDesc::Local(usize::from(location)),
                        program,
                        compile_context,
                    )?;
                    Ok(location)
                }
            },
            other => unreachable!(
                "Should always be called with a ExpDesc::Name, but was called with {:?}.",
                other
            ),
        }
    }

    fn resolve_jumps_to_block(
        start_of_jumps_to_resolve: usize,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let jumps_to_block = compile_context
            .jumps_to_block
            .drain(start_of_jumps_to_resolve..)
            .collect::<Vec<_>>();
        let jump_dst = program.byte_codes.len();
        for jump in jumps_to_block {
            program.byte_codes[jump] =
                Bytecode::jump(i32::try_from(jump_dst - jump - 1).map_err(|_| Error::LongJump)?);
        }
        Ok(())
    }

    fn resolve_jumps_to_end(
        start_of_jumps_to_resolve: usize,
        program: &mut Proto,
        compile_context: &mut CompileContext<'a>,
    ) -> Result<(), Error> {
        let jumps_to_end = compile_context
            .jumps_to_end
            .drain(start_of_jumps_to_resolve..)
            .collect::<Vec<_>>();
        let jump_dst = program.byte_codes.len();
        for jump in jumps_to_end {
            program.byte_codes[jump] =
                Bytecode::jump(i32::try_from(jump_dst - jump - 1).map_err(|_| Error::LongJump)?);
        }
        Ok(())
    }
}
