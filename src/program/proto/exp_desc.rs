use alloc::{boxed::Box, format, vec::Vec};

use crate::{
    bytecode::{
        OpCode,
        arguments::{A, B, BytecodeArgument, C, K, Sbx, Sj},
    },
    ext::Unescape,
};

use super::{
    Bytecode, Error,
    binops::Binop,
    compile_stack::{CompileFrame, CompileStack, ExpList},
    helper_types::{TableFields, TableKey},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExpDesc<'a> {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    Name(&'a str),
    LongName(&'a str),
    Unop(fn(A, B) -> Bytecode, Box<ExpDesc<'a>>),
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
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        match self {
            Self::Name(_) => self.discharge_into_name(src, compile_stack),
            Self::LongName(_) => self.discharge_into_long_name(src, compile_stack),
            Self::Local(_) => self.discharge_into_local(src, compile_stack),
            Self::Global(_) => self.discharge_into_global(src, compile_stack),
            Self::Upvalue(_) => self.discharge_into_upvalue(src, compile_stack),
            Self::ExpList(_) => self.discharge_into_explist(src, compile_stack),
            Self::TableAccess {
                table: _,
                key: _,
                record: _,
            } => self.discharge_into_table_access(src, compile_stack),
            Self::Condition {
                jump_to_end: _,
                if_condition: _,
            } => self.discharge_into_condition(src, compile_stack),
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
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        let Self::Name(name) = self else {
            unreachable!(
                "Destination of `discharge_into_name` must be `ExpDesc::Name`, but was {:?}.",
                self
            );
        };

        let Some(name) = compile_stack
            .view()
            .find_name(name)
            .or_else(|| compile_stack.view().capture_name(name))
            .or_else(|| compile_stack.view().capture_environment(name))
        else {
            unreachable!("Should always fallback to Global.");
        };
        name.discharge(src, compile_stack)
    }

    fn discharge_into_long_name(
        &self,
        src: &ExpDesc<'a>,
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        let Self::LongName(long_name) = self else {
            unreachable!(
                "Destination of `discharge_into_long_name` must be `ExpDesc::LongName`, but was {:?}.",
                self
            );
        };

        let env = compile_stack.proto_mut().push_upvalue("_ENV");

        let (_, env_top) = compile_stack.compile_context_mut().reserve_stack_top();
        env_top.discharge(&Self::Upvalue(env), compile_stack)?;

        let (_, key_top) = compile_stack.compile_context_mut().reserve_stack_top();
        key_top.discharge(&Self::String(long_name), compile_stack)?;

        let env_table = Self::TableAccess {
            table: Box::new(env_top),
            key: Box::new(key_top),
            record: false,
        };
        env_table.discharge(src, compile_stack)?;

        compile_stack.compile_context_mut().stack_top -= 2;

        Ok(())
    }

    fn discharge_into_local(
        &self,
        src: &ExpDesc<'a>,
        compile_stack: &mut CompileStack<'a>,
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
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::load_nil(dst.into(), 0.into()));
                Ok(())
            }
            Self::Boolean(boolean) => {
                let bytecode = if *boolean {
                    Bytecode::load_true(dst.into())
                } else {
                    Bytecode::load_false(dst.into())
                };
                compile_stack.proto_mut().byte_codes.push(bytecode);
                Ok(())
            }
            Self::Integer(integer) => {
                if let Ok(integer) = Sbx::try_from(*integer) {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_integer(dst.into(), integer));
                } else {
                    let constant = compile_stack.proto_mut().push_constant(*integer)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_constant(dst.into(), constant.try_into()?));
                }
                Ok(())
            }
            Self::Float(float) => {
                if let Ok(float) = Sbx::try_from(*float) {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_float(dst.into(), float));
                    Ok(())
                } else {
                    let constant = compile_stack.proto_mut().push_constant(*float)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_constant(dst.into(), constant.try_into()?));
                    Ok(())
                }
            }
            Self::String(string) => {
                let unescaped_string = string.unescape()?;
                let constant = compile_stack
                    .proto_mut()
                    .push_constant(unescaped_string.as_str())?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::load_constant(dst.into(), constant.try_into()?));
                Ok(())
            }
            Self::Name(name) => {
                let Some(name) = compile_stack
                    .view()
                    .find_name(name)
                    .or_else(|| compile_stack.view().capture_name(name))
                    .or_else(|| compile_stack.view().capture_environment(name))
                else {
                    unreachable!("Should always fallback to Global.");
                };
                self.discharge(&name, compile_stack)
            }
            Self::LongName(long_name) => {
                // Reaching here already means that it is a global
                let env = compile_stack.proto_mut().push_upvalue("_ENV");

                self.discharge(&Self::Upvalue(env), compile_stack)?;
                let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_top.discharge(&Self::String(long_name), compile_stack)?;
                self.discharge(
                    &Self::TableAccess {
                        table: Box::new(self.clone()),
                        key: Box::new(stack_top),
                        record: false,
                    },
                    compile_stack,
                )?;
                compile_stack.compile_context_mut().stack_top -= 1;

                Ok(())
            }
            Self::Unop(op, var) => match var.as_ref() {
                Self::Name(name) => {
                    let Some(name) = compile_stack
                        .view()
                        .find_name(name)
                        .or_else(|| compile_stack.view().capture_name(name))
                        .or_else(|| compile_stack.view().capture_environment(name))
                    else {
                        unreachable!("Should always fallback to Global.");
                    };
                    self.discharge(&Self::Unop(*op, Box::new(name)), compile_stack)
                }
                Self::Local(local) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(op(dst.into(), u8::try_from(*local)?.into()));
                    Ok(())
                }
                global @ Self::Global(_) => {
                    self.discharge(global, compile_stack)?;
                    self.discharge(&Self::Unop(*op, Box::new(self.clone())), compile_stack)
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
                    self.discharge(lhs, compile_stack)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        compile_stack,
                    )
                }
                (_, Self::Name(name), _) => {
                    let Some(name) = compile_stack
                        .view()
                        .find_name(name)
                        .or_else(|| compile_stack.view().capture_name(name))
                        .or_else(|| compile_stack.view().capture_environment(name))
                    else {
                        unreachable!("Should always fallback to Global.");
                    };
                    self.discharge(
                        &Self::Binop(*op, Box::new(name), rhs.clone()),
                        compile_stack,
                    )
                }
                (_, _, Self::Name(name)) => {
                    let Some(name) = compile_stack
                        .view()
                        .find_name(name)
                        .or_else(|| compile_stack.view().capture_name(name))
                        .or_else(|| compile_stack.view().capture_environment(name))
                    else {
                        unreachable!("Should always fallback to Global.");
                    };
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(name)),
                        compile_stack,
                    )
                }
                (_, upval @ Self::Upvalue(_), _) => {
                    self.discharge(upval, compile_stack)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        compile_stack,
                    )
                }
                // TODO expand to other `Binop`s
                (op, binop @ Self::Binop(Binop::Add, _, _), _) => {
                    self.discharge(binop, compile_stack)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        compile_stack,
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
                    self.discharge(table_access, compile_stack)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        compile_stack,
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
                    let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                    stack_top.discharge(table_access, compile_stack)?;
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(stack_top)),
                        compile_stack,
                    )?;
                    compile_stack.compile_context_mut().stack_top -= 1;
                    Ok(())
                }
                (Binop::Add, Self::Integer(_), global @ Self::Global(_)) => {
                    self.discharge(global, compile_stack)?;
                    self.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), lhs.clone()),
                        compile_stack,
                    )
                }
                (Binop::Add, Self::Local(lhs), Self::Integer(rhs)) => {
                    if let Ok(rhs) = i8::try_from(*rhs) {
                        compile_stack
                            .proto_mut()
                            .byte_codes
                            .push(Bytecode::add_integer(
                                dst.into(),
                                u8::try_from(*lhs)?.into(),
                                rhs.into(),
                            ));
                        Ok(())
                    } else {
                        todo!()
                    }
                }
                (Binop::Add, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack.proto_mut().byte_codes.push(Bytecode::add(
                        dst.into(),
                        u8::try_from(*lhs)?.into(),
                        u8::try_from(*rhs)?.into(),
                    ));
                    Ok(())
                }
                (Binop::Sub, Self::Local(lhs), Self::Integer(rhs)) => {
                    if let Ok(rhs) = i8::try_from(*rhs) {
                        compile_stack
                            .proto_mut()
                            .byte_codes
                            .push(Bytecode::add_integer(
                                dst.into(),
                                u8::try_from(*lhs)?.into(),
                                (-rhs).into(),
                            ));
                        Ok(())
                    } else {
                        todo!()
                    }
                }
                (Binop::Sub, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack.proto_mut().byte_codes.push(Bytecode::sub(
                        dst.into(),
                        u8::try_from(*lhs)?.into(),
                        u8::try_from(*rhs)?.into(),
                    ));
                    Ok(())
                }
                (Binop::Mul, Self::Local(lhs), Self::Float(rhs)) => {
                    let rhs = compile_stack.proto_mut().push_constant(*rhs)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::mul_constant(
                            dst.into(),
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(rhs)?.into(),
                        ));
                    Ok(())
                }
                (Binop::Div, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack.proto_mut().byte_codes.push(Bytecode::div(
                        dst.into(),
                        u8::try_from(*lhs)?.into(),
                        u8::try_from(*rhs)?.into(),
                    ));
                    Ok(())
                }
                (Binop::ShiftLeft, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::shift_left(
                            dst.into(),
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(*rhs)?.into(),
                        ));
                    Ok(())
                }
                (Binop::ShiftRight, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::shift_right(
                            dst.into(),
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(*rhs)?.into(),
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
                    self.discharge(lhs, compile_stack)?;
                    let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                    stack_top.discharge(
                        &Self::Binop(*op, Box::new(self.clone()), rhs.clone()),
                        compile_stack,
                    )?;
                    compile_stack.compile_context_mut().stack_top -= 1;
                    Ok(())
                }
                (
                    Binop::Concat,
                    Self::Local(lhs),
                    rhs @ (Self::Integer(_) | Self::String(_) | Self::Local(_)),
                ) => {
                    self.discharge(rhs, compile_stack)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::concat(u8::try_from(*lhs)?.into(), 2.into()));

                    Ok(())
                }
                (Binop::Concat, Self::Local(lhs), rhs @ Self::Binop(Binop::Concat, _, _)) => {
                    self.discharge(rhs, compile_stack)?;

                    let Some(last_bytecode) = compile_stack.proto_mut().byte_codes.last_mut()
                    else {
                        unreachable!(
                            "Bytecodes should not be empty after discharging concatenation."
                        );
                    };
                    assert_eq!(OpCode::read(**last_bytecode), OpCode::Concat);
                    let (_, b, _, _) = last_bytecode.decode_abck();
                    *last_bytecode = Bytecode::concat(u8::try_from(*lhs)?.into(), (*b + 1).into());

                    Ok(())
                }
                (Binop::Or, lhs, rhs) => {
                    self.discharge(lhs, compile_stack)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::test(dst.into(), K::ONE));
                    let shortcircuit = compile_stack.proto_mut().byte_codes.len();
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(Sj::ZERO));
                    compile_stack
                        .compile_context_mut()
                        .jumps_to_block
                        .push(shortcircuit);

                    self.discharge(rhs, compile_stack)
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
                    let jumps_to_block = compile_stack.compile_context_mut().jumps_to_block.len();
                    let jumps_to_end = compile_stack.compile_context_mut().jumps_to_end.len();

                    let lhs_cond = Self::Condition {
                        jump_to_end: false,
                        if_condition: false,
                    };
                    lhs_cond.discharge(lhs, compile_stack)?;

                    let rhs_cond = Self::Condition {
                        jump_to_end: true,
                        if_condition: true,
                    };
                    rhs_cond.discharge(rhs, compile_stack)?;

                    Self::resolve_jumps_to_block(jumps_to_block, compile_stack)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_false_skip(dst.into()));

                    Self::resolve_jumps_to_end(jumps_to_end, compile_stack)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_true(dst.into()));

                    Ok(())
                }
                (Binop::And, lhs, rhs) => {
                    let jumps_to_block = compile_stack.compile_context_mut().jumps_to_block.len();

                    self.discharge(lhs, compile_stack)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::test(dst.into(), K::ZERO));
                    let shortcircuit = compile_stack.proto_mut().byte_codes.len();
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(Sj::ZERO));

                    Self::resolve_jumps_to_block(jumps_to_block, compile_stack)?;
                    compile_stack
                        .compile_context_mut()
                        .jumps_to_block
                        .push(shortcircuit);

                    self.discharge(rhs, compile_stack)
                }
                (Binop::LessThan, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::less_than(
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(*rhs)?.into(),
                            K::ONE,
                        ));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(1i8.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_false_skip(dst.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_true(dst.into()));

                    Ok(())
                }
                (Binop::GreaterThan, Self::Local(lhs), Self::Integer(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::greater_than_integer(
                            u8::try_from(*lhs)?.into(),
                            i8::try_from(*rhs)?.into(),
                            K::ONE,
                        ));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(1i8.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_false_skip(dst.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_true(dst.into()));

                    Ok(())
                }
                (Binop::GreaterThan, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::less_than(
                            u8::try_from(*rhs)?.into(),
                            u8::try_from(*lhs)?.into(),
                            K::ONE,
                        ));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(1i8.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_false_skip(dst.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_true(dst.into()));

                    Ok(())
                }
                (Binop::LessEqual, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::less_equal(
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(*rhs)?.into(),
                            K::ONE,
                        ));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(1i8.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_false_skip(dst.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_true(dst.into()));

                    Ok(())
                }
                (Binop::GreaterEqual, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::less_equal(
                            u8::try_from(*rhs)?.into(),
                            u8::try_from(*lhs)?.into(),
                            K::ONE,
                        ));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(1i8.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_false_skip(dst.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_true(dst.into()));

                    Ok(())
                }
                (Binop::Equal, Self::Local(lhs), Self::String(rhs)) => {
                    let rhs = compile_stack.proto_mut().push_constant(*rhs)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::equal_constant(
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(rhs)?.into(),
                            K::ONE,
                        ));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(1i8.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_false_skip(dst.into()));
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_true(dst.into()));

                    Ok(())
                }
                _ => unimplemented!("Can't discharge binary operation {:?}.", src),
            },
            Self::Local(local) => {
                let local = u8::try_from(*local)?;
                if local != dst {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::move_bytecode(dst.into(), local.into()));
                }
                Ok(())
            }
            Self::Global(global) => {
                let env = compile_stack.proto_mut().push_upvalue("_ENV");
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::get_uptable(
                        dst.into(),
                        u8::try_from(env)?.into(),
                        u8::try_from(*global)?.into(),
                    ));
                Ok(())
            }
            Self::Upvalue(upvalue) => {
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::get_upvalue(
                        dst.into(),
                        u8::try_from(*upvalue)?.into(),
                    ));
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
                    .next_back()
                    .filter(|(_, field)| matches!(field, Self::VariadicArguments))
                    .is_some();

                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::new_table(
                        dst.into(),
                        u8::try_from(fields.len() - array_count)?.into(),
                        (u8::try_from(array_count)? - (last_array_field_is_variadic as u8)).into(),
                    ));

                let mut used_stack = 0;
                let mut last_variadic_bytecode = 0;

                for (key, field) in fields.iter() {
                    match key {
                        TableKey::Array => {
                            let (_, stack_top) =
                                compile_stack.compile_context_mut().reserve_stack_top();
                            used_stack += 1;
                            stack_top.discharge(field, compile_stack)?;

                            let Some(last_bytecode) =
                                compile_stack.proto_mut().byte_codes.last_mut()
                            else {
                                unreachable!(
                                    "Bytecodes should never be empty while discharging table fields."
                                );
                            };
                            if OpCode::read(**last_bytecode) == OpCode::VariadicArguments {
                                let (a, _, _, _) = last_bytecode.decode_abck();
                                *last_bytecode = Bytecode::variadic_arguments(a, 2.into());
                                last_variadic_bytecode =
                                    compile_stack.proto_mut().byte_codes.len() - 1;
                            }
                        }
                        TableKey::General(key) => {
                            Self::TableAccess {
                                table: Box::new(Self::Local(usize::from(dst))),
                                key: key.clone(),
                                record: false,
                            }
                            .discharge(field, compile_stack)?;
                        }
                        TableKey::Record(key) => {
                            Self::TableAccess {
                                table: Box::new(Self::Local(usize::from(dst))),
                                key: key.clone(),
                                record: true,
                            }
                            .discharge(field, compile_stack)?;
                        }
                    }
                }

                let array_count = if last_array_field_is_variadic {
                    let (a, _, _, _) =
                        compile_stack.proto_mut().byte_codes[last_variadic_bytecode].decode_abck();
                    compile_stack.proto_mut().byte_codes[last_variadic_bytecode] =
                        Bytecode::variadic_arguments(a, C::ZERO);
                    Some(0)
                } else if array_count != 0 {
                    Some(u8::try_from(array_count)?)
                } else {
                    None
                };

                if let Some(array_count) = array_count {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::set_list(dst.into(), array_count.into(), C::ZERO));
                }

                compile_stack.compile_context_mut().stack_top -= used_stack;

                Ok(())
            }
            Self::TableAccess {
                table,
                key,
                record: false,
            } => match (table.as_ref(), key.as_ref()) {
                (Self::Name(table), _) => {
                    let Some(table) = compile_stack
                        .view()
                        .find_name(table)
                        .or_else(|| compile_stack.view().capture_name(table))
                        .or_else(|| compile_stack.view().capture_environment(table))
                    else {
                        unreachable!("Should always fallback to Global.");
                    };
                    self.discharge(
                        &Self::TableAccess {
                            table: Box::new(table),
                            key: key.clone(),
                            record: false,
                        },
                        compile_stack,
                    )
                }
                (table @ Self::Upvalue(_), Self::String(key)) => {
                    let global = compile_stack.proto_mut().push_constant(*key)?;

                    self.discharge(
                        &Self::TableAccess {
                            table: Box::new(table.clone()),
                            key: Box::new(ExpDesc::Global(usize::try_from(global)?)),
                            record: false,
                        },
                        compile_stack,
                    )
                }
                (Self::Upvalue(table), Self::Global(global)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::get_uptable(
                            dst.into(),
                            u8::try_from(*table)?.into(),
                            u8::try_from(*global)?.into(),
                        ));
                    Ok(())
                }
                (Self::Local(local_table), Self::Integer(index)) => {
                    if let Ok(index) = u8::try_from(*index) {
                        compile_stack
                            .proto_mut()
                            .byte_codes
                            .push(Bytecode::get_index(
                                dst.into(),
                                u8::try_from(*local_table)?.into(),
                                index.into(),
                            ));
                        Ok(())
                    } else {
                        let (_, stack_top) =
                            compile_stack.compile_context_mut().reserve_stack_top();
                        stack_top.discharge(&Self::Integer(*index), compile_stack)?;
                        self.discharge(
                            &Self::TableAccess {
                                table: table.clone(),
                                key: Box::new(stack_top),
                                record: false,
                            },
                            compile_stack,
                        )?;
                        compile_stack.compile_context_mut().stack_top -= 1;
                        Ok(())
                    }
                }
                (Self::Local(table), Self::String(key)) => {
                    let key = compile_stack.proto_mut().push_constant(*key)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::get_field(
                            dst.into(),
                            u8::try_from(*table)?.into(),
                            u8::try_from(key)?.into(),
                        ));
                    Ok(())
                }
                (Self::Local(table), Self::Local(key)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::get_table(
                            dst.into(),
                            u8::try_from(*table)?.into(),
                            u8::try_from(*key)?.into(),
                        ));
                    Ok(())
                }
                (table @ Self::Global(_), _) => {
                    self.discharge(table, compile_stack)?;
                    let table_access = Self::TableAccess {
                        table: Box::new(self.clone()),
                        key: key.clone(),
                        record: false,
                    };
                    self.discharge(&table_access, compile_stack)
                }
                (
                    table @ Self::TableAccess {
                        table: _,
                        key: _,
                        record: _,
                    },
                    _,
                ) => {
                    self.discharge(table, compile_stack)?;
                    let table_access = Self::TableAccess {
                        table: Box::new(self.clone()),
                        key: key.clone(),
                        record: false,
                    };
                    self.discharge(&table_access, compile_stack)
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
                    compile_stack,
                )
            }
            Self::Closure(closure) => {
                compile_stack.proto_mut().byte_codes.push(Bytecode::closure(
                    dst.into(),
                    u32::try_from(*closure)?.try_into()?,
                ));
                Ok(())
            }
            Self::FunctionCall(function, args) => {
                self.discharge(function, compile_stack)?;

                let jumps_to_block = compile_stack.compile_context_mut().jumps_to_block.len();
                for arg in args.iter() {
                    let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                    stack_top.discharge(arg, compile_stack)?;

                    match compile_stack.proto_mut().byte_codes.last_mut() {
                        Some(bytecode) => match OpCode::read(**bytecode) {
                            OpCode::VariadicArguments => {
                                let (register, _, _, _) = bytecode.decode_abck();
                                *bytecode = Bytecode::variadic_arguments(register, 2.into());
                            }
                            OpCode::Call => {
                                let (func, in_params, _, _) = bytecode.decode_abck();
                                *bytecode = Bytecode::call(func, in_params, 2.into());
                            }
                            _ => (),
                        },
                        None => unreachable!(
                            "Bytecodes should not be empty after discharging argument."
                        ),
                    }
                }
                compile_stack.compile_context_mut().stack_top -= u8::try_from(args.len())?;

                let Some(last_bytecode) = compile_stack.proto_mut().byte_codes.last_mut() else {
                    unreachable!("Bytecodes should not be empty after discharging argument,");
                };
                let in_params = match OpCode::read(**last_bytecode) {
                    OpCode::Call => {
                        let (func, in_params, _, _) = last_bytecode.decode_abck();
                        *last_bytecode = Bytecode::call(func, in_params, C::ZERO);
                        0
                    }
                    OpCode::VariadicArguments => {
                        let (a, _, _, _) = last_bytecode.decode_abck();
                        *last_bytecode = Bytecode::variadic_arguments(a, C::ZERO);
                        0
                    }
                    _ => u8::try_from(args.len())? + 1,
                };

                Self::resolve_jumps_to_block(jumps_to_block, compile_stack)?;

                compile_stack.proto_mut().byte_codes.push(Bytecode::call(
                    dst.into(),
                    in_params.into(),
                    1.into(),
                ));

                Ok(())
            }
            Self::MethodCall(table, method_name, exp_list) => {
                self.discharge(table, compile_stack)?;

                let Self::Name(name) = method_name.as_ref() else {
                    unreachable!("Method name should be a Name, but was {:?}.", method_name);
                };
                let constant = compile_stack.proto_mut().push_constant(*name)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::table_self(
                        dst.into(),
                        dst.into(),
                        u8::try_from(constant)?.into(),
                    ));

                // reserve `self`
                let (_, _) = compile_stack.compile_context_mut().reserve_stack_top();
                let mut used_stack = 1;

                for exp in exp_list.iter() {
                    let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                    stack_top.discharge(exp, compile_stack)?;
                    used_stack += 1;
                }

                compile_stack.compile_context_mut().stack_top -= used_stack;

                compile_stack.proto_mut().byte_codes.push(Bytecode::call(
                    dst.into(),
                    (used_stack + 1).into(),
                    1.into(),
                ));

                Ok(())
            }
            Self::VariadicArguments => {
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::variadic_arguments(dst.into(), C::ZERO));
                Ok(())
            }
            other => unreachable!("{:?} can't be discharged into Local.", other),
        }
    }

    fn discharge_into_global(
        &self,
        src: &ExpDesc<'a>,
        compile_stack: &mut CompileStack<'a>,
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
                let env = compile_stack.proto_mut().push_upvalue("_ENV");
                let constant = compile_stack.proto_mut().push_constant(*integer)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_uptable(
                        u8::try_from(env)?.into(),
                        global.into(),
                        u8::try_from(constant)?.into(),
                        K::ONE,
                    ));
                Ok(())
            }
            Self::String(string) => {
                let env = compile_stack.proto_mut().push_upvalue("_ENV");
                let constant = compile_stack.proto_mut().push_constant(*string)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_uptable(
                        u8::try_from(env)?.into(),
                        global.into(),
                        u8::try_from(constant)?.into(),
                        K::ONE,
                    ));
                Ok(())
            }
            Self::Name(name) => {
                let Some(name) = compile_stack
                    .view()
                    .find_name(name)
                    .or_else(|| compile_stack.view().capture_name(name))
                    .or_else(|| compile_stack.view().capture_environment(name))
                else {
                    unreachable!("Should always fallback to Global.");
                };
                self.discharge(&name, compile_stack)
            }
            Self::Local(local) => {
                let env = compile_stack.proto_mut().push_upvalue("_ENV");
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_uptable(
                        u8::try_from(env)?.into(),
                        global.into(),
                        u8::try_from(*local)?.into(),
                        K::ZERO,
                    ));
                Ok(())
            }
            exp @ (Self::Global(_)
            | Self::Upvalue(_)
            | Self::Table(_)
            | Self::FunctionCall(_, _)) => {
                let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_top.discharge(exp, compile_stack)?;
                self.discharge(&stack_top, compile_stack)?;
                compile_stack.compile_context_mut().stack_top -= 1;

                Ok(())
            }
            other => unreachable!("Can't discharge {:?} into Global.", other),
        }
    }

    fn discharge_into_upvalue(
        &self,
        src: &ExpDesc<'a>,
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        let Self::Upvalue(upvalue) = self else {
            unreachable!(
                "Destination of `discharge_into_upvalue` must be `ExpDesc::Upvalue`, but was {:?}.",
                self
            );
        };

        if let Self::Local(local) = src {
            compile_stack
                .proto_mut()
                .byte_codes
                .push(Bytecode::set_upvalue(
                    u8::try_from(*upvalue)?.into(),
                    u8::try_from(*local)?.into(),
                ));
        } else {
            let (stack_loc, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
            stack_top.discharge(src, compile_stack)?;
            compile_stack
                .proto_mut()
                .byte_codes
                .push(Bytecode::set_upvalue(
                    u8::try_from(*upvalue)?.into(),
                    stack_loc.into(),
                ));
            compile_stack.compile_context_mut().stack_top -= 1;
        }
        Ok(())
    }

    fn discharge_into_explist(
        &self,
        src: &ExpDesc<'a>,
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        let Self::ExpList(explist) = self else {
            unreachable!(
                "Destination of `discharge_into_explist` must be `ExpDesc::Explist`, but was {:?}.",
                self
            );
        };

        match src {
            Self::ExpList(src_explist) => {
                if src_explist.is_empty() {
                    let Some(dst) = explist.iter().rev().try_fold(usize::MAX, |prev, cur| {
                        let ExpDesc::Local(i) = cur else {
                            unreachable!("Detination of empty local initialization need to be Local, but was {:?}.", cur);  
                        };
                        Some(*i).filter(|i| if prev == usize::MAX {
                            true
                        } else {
                            i + 1 == prev
                        })
                    }) else {
                        return Err(Error::NonSequentialLocalInitialization(format!{"{:?}",explist}.into_boxed_str()));
                    };
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::load_nil(
                            u8::try_from(dst)?.into(),
                            u8::try_from(explist.len() - 1)?.into(),
                        ));
                    Ok(())
                } else {
                    let mut used_stack = 0;
                    let mut reverse_sets = Vec::new();

                    if explist.len() == 1 && src_explist.len() == 1 {
                        explist[0].discharge(&src_explist[0], compile_stack)?;
                    } else {
                        for lhs_exp in explist.iter() {
                            if let Self::Name(name) = lhs_exp {
                                if compile_stack
                                    .view()
                                    .find_name(name)
                                    .or_else(|| compile_stack.view().capture_name(name))
                                    .or_else(|| compile_stack.view().capture_environment(name))
                                    .is_none()
                                {
                                    unreachable!("Should always fallback to Global.");
                                };
                            }
                        }

                        let mut first = true;
                        for (dst, src) in explist.iter().zip(src_explist.iter()) {
                            match dst {
                                ExpDesc::Name(name) => {
                                    let Some(dst_name) = compile_stack
                                        .view()
                                        .find_name(name)
                                        .or_else(|| compile_stack.view().capture_name(name))
                                        .or_else(|| compile_stack.view().capture_environment(name))
                                    else {
                                        unreachable!("Should always fallback to Global.");
                                    };
                                    let src = if let Self::Name(src) = src {
                                        let Some(src) = compile_stack
                                            .view()
                                            .find_name(src)
                                            .or_else(|| compile_stack.view().capture_name(src))
                                            .or_else(|| {
                                                compile_stack.view().capture_environment(src)
                                            })
                                        else {
                                            unreachable!("Should always fallback to Global.");
                                        };
                                        src
                                    } else {
                                        src.clone()
                                    };

                                    if first
                                        || matches!(
                                            src,
                                            Self::Upvalue(_) | Self::FunctionCall(_, _)
                                        )
                                        || matches!(dst_name, Self::Upvalue(_))
                                    {
                                        let (_, stack_top) =
                                            compile_stack.compile_context_mut().reserve_stack_top();
                                        stack_top.discharge(&src, compile_stack)?;
                                        reverse_sets.push((dst_name.clone(), stack_top));
                                        used_stack += 1;
                                    } else {
                                        dst_name.discharge(&src, compile_stack)?;
                                    }
                                }
                                local @ ExpDesc::Local(_) => {
                                    local.discharge(src, compile_stack)?;
                                }
                                ExpDesc::TableAccess {
                                    table: _,
                                    key: _,
                                    record: _,
                                } => dst.discharge(src, compile_stack)?,
                                _ => unreachable!(
                                    "Varlist expressions should always be Name or TableAccess, but was {:?}.",
                                    dst
                                ),
                            }
                            first = false;
                        }
                    }

                    match src_explist.last() {
                        Some(ExpDesc::FunctionCall(_, _)) => {
                            for remaining in explist[src_explist.len()..].iter() {
                                if let Self::Name(_) = remaining {
                                    let (_, stack_top) =
                                        compile_stack.compile_context_mut().reserve_stack_top();
                                    reverse_sets.push((remaining.clone(), stack_top));
                                    used_stack += 1;
                                }
                            }

                            let Some(last_bytecode) =
                                compile_stack.proto_mut().byte_codes.last_mut()
                            else {
                                unreachable!("Bytecodes should not be empty while discharging.");
                            };
                            assert_eq!(OpCode::read(**last_bytecode), OpCode::Call);

                            let (function, in_params, _, _) = last_bytecode.decode_abck();
                            *last_bytecode = Bytecode::call(
                                function,
                                in_params,
                                u8::try_from(explist.len() - src_explist.len() + 2)?.into(),
                            );
                        }
                        Some(ExpDesc::VariadicArguments) => {
                            for remaining in explist[src_explist.len()..].iter() {
                                if let Self::Name(_) = remaining {
                                    let (_, stack_top) =
                                        compile_stack.compile_context_mut().reserve_stack_top();
                                    reverse_sets.push((remaining.clone(), stack_top));
                                    used_stack += 1;
                                }
                            }

                            let Some(last_bytecode) =
                                compile_stack.proto_mut().byte_codes.last_mut()
                            else {
                                unreachable!("Bytecodes should not be empty while discharging.");
                            };
                            assert_eq!(OpCode::read(**last_bytecode), OpCode::VariadicArguments);

                            let (register, _, _, _) = last_bytecode.decode_abck();
                            *last_bytecode = Bytecode::variadic_arguments(
                                register,
                                u8::try_from(explist.len() - src_explist.len() + 2)?.into(),
                            );
                        }
                        Some(_) => {
                            for dst in explist[src_explist.len()..].iter() {
                                if matches!(dst, Self::Global(_)) {
                                    let (_, stack_top) =
                                        compile_stack.compile_context_mut().reserve_stack_top();
                                    stack_top.discharge(&ExpDesc::Nil, compile_stack)?;
                                    reverse_sets.push((dst.clone(), stack_top));
                                    used_stack += 1;
                                } else {
                                    dst.discharge(&ExpDesc::Nil, compile_stack)?;
                                }
                            }
                        }
                        None => {
                            unreachable!("src_explist should never be empty")
                        }
                    }

                    for (dst, src) in reverse_sets.into_iter().rev() {
                        dst.discharge(&src, compile_stack)?;
                    }

                    compile_stack.compile_context_mut().stack_top -= used_stack;
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
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        let Self::TableAccess { table, key, record } = self else {
            unreachable!(
                "Destination of `discharge_into_table_access` must be `ExpDesc::TableAccess`, but was {:?}.",
                self
            );
        };

        match (table.as_ref(), key.as_ref(), record, src) {
            (_, _, _, src @ ExpDesc::Upvalue(_)) => {
                let (_, stack_exp) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_exp.discharge(src, compile_stack)?;
                compile_stack.compile_context_mut().stack_top -= 1;

                self.discharge(&stack_exp, compile_stack)
            }
            (_, _, _, src @ ExpDesc::Closure(_)) => {
                let (_, stack_exp) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_exp.discharge(src, compile_stack)?;
                compile_stack.compile_context_mut().stack_top -= 1;

                self.discharge(&stack_exp, compile_stack)
            }
            (_, Self::Name(key), true, _) => {
                // Rewrite all access in the form `t.x` as `t["x"]`
                let table_access = Self::TableAccess {
                    table: table.clone(),
                    key: Box::new(ExpDesc::String(key)),
                    record: false,
                };
                table_access.discharge(src, compile_stack)
            }
            (_, key, true, _) => {
                unreachable!(
                    "Record table access must be keyd by Name, but was {:?}.",
                    key
                );
            }
            (table @ Self::Global(_), _, false, _) => {
                let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_top.discharge(table, compile_stack)?;
                let table_access = Self::TableAccess {
                    table: Box::new(stack_top),
                    key: key.clone(),
                    record: false,
                };
                table_access.discharge(src, compile_stack)?;
                compile_stack.compile_context_mut().stack_top -= 1;

                Ok(())
            }
            (Self::Name(table), _, false, _) => {
                let Some(table) = compile_stack
                    .view()
                    .find_name(table)
                    .or_else(|| compile_stack.view().capture_name(table))
                    .or_else(|| compile_stack.view().capture_environment(table))
                else {
                    unreachable!("Should always fallback to Global.");
                };
                let table_access = Self::TableAccess {
                    table: Box::new(table),
                    key: key.clone(),
                    record: false,
                };
                table_access.discharge(src, compile_stack)
            }
            (_, Self::Name(key), false, _) => {
                let Some(name) = compile_stack
                    .view()
                    .find_name(key)
                    .or_else(|| compile_stack.view().capture_name(key))
                    .or_else(|| compile_stack.view().capture_environment(key))
                else {
                    unreachable!("Should always fallback to Global.");
                };

                let table_access = Self::TableAccess {
                    table: table.clone(),
                    key: Box::new(name),
                    record: false,
                };
                table_access.discharge(src, compile_stack)?;

                Ok(())
            }
            (_, Self::String(key), false, Self::Name(name)) => {
                // Storing the key into constants early to match the ordering
                // of the official compiler
                let _ = compile_stack.proto_mut().push_constant(*key)?;
                let Some(name) = compile_stack
                    .view()
                    .find_name(name)
                    .or_else(|| compile_stack.view().capture_name(name))
                    .or_else(|| compile_stack.view().capture_environment(name))
                else {
                    unreachable!("Should always fallback to Global.");
                };
                self.discharge(&name, compile_stack)
            }
            (_, _, false, Self::Name(name)) => {
                let Some(name) = compile_stack
                    .view()
                    .find_name(name)
                    .or_else(|| compile_stack.view().capture_name(name))
                    .or_else(|| compile_stack.view().capture_environment(name))
                else {
                    unreachable!("Should always fallback to Global.");
                };
                self.discharge(&name, compile_stack)
            }
            // local t, k
            // t[k] = 1
            (Self::Local(table), Self::Local(key), false, Self::Integer(integer)) => {
                let constant = compile_stack.proto_mut().push_constant(*integer)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_table(
                        u8::try_from(*table)?.into(),
                        u8::try_from(*key)?.into(),
                        u8::try_from(constant)?.into(),
                        K::ONE,
                    ));
                Ok(())
            }
            // local t, k
            // t[k] = "a"
            (Self::Local(table), Self::Local(key), false, Self::String(string)) => {
                let constant = compile_stack.proto_mut().push_constant(*string)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_table(
                        u8::try_from(*table)?.into(),
                        u8::try_from(*key)?.into(),
                        u8::try_from(constant)?.into(),
                        K::ONE,
                    ));
                Ok(())
            }
            // local t, k, a
            // t[k] = a
            (Self::Local(table), Self::Local(key), false, Self::Local(src)) => {
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_table(
                        u8::try_from(*table)?.into(),
                        u8::try_from(*key)?.into(),
                        u8::try_from(*src)?.into(),
                        K::ZERO,
                    ));
                Ok(())
            }
            // local t
            // t["x"] = 1
            (Self::Local(table), Self::String(key), false, Self::Integer(integer)) => {
                let key_constant = compile_stack.proto_mut().push_constant(*key)?;
                let constant = compile_stack.proto_mut().push_constant(*integer)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_field(
                        u8::try_from(*table)?.into(),
                        u8::try_from(key_constant)?.into(),
                        u8::try_from(constant)?.into(),
                        K::ONE,
                    ));
                Ok(())
            }
            // local t
            // t["x"] = "y"
            (Self::Local(table), Self::String(key), false, Self::String(string)) => {
                let key_constant = compile_stack.proto_mut().push_constant(*key)?;
                let constant = compile_stack.proto_mut().push_constant(*string)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_field(
                        u8::try_from(*table)?.into(),
                        u8::try_from(key_constant)?.into(),
                        u8::try_from(constant)?.into(),
                        K::ONE,
                    ));
                Ok(())
            }
            // local t, a
            // t["x"] = a
            (Self::Local(table), Self::String(key), false, Self::Local(src)) => {
                let key_constant = compile_stack.proto_mut().push_constant(*key)?;
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::set_field(
                        u8::try_from(*table)?.into(),
                        u8::try_from(key_constant)?.into(),
                        u8::try_from(*src)?.into(),
                        K::ZERO,
                    ));
                Ok(())
            }
            // local t
            // t["x"] = a
            (_, _, false, global @ Self::Global(_)) => {
                let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();

                stack_top.discharge(global, compile_stack)?;
                self.discharge(&stack_top, compile_stack)?;

                compile_stack.compile_context_mut().stack_top -= 1;
                Ok(())
            }
            (_, _, false, table @ Self::Table(_)) => {
                let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_top.discharge(table, compile_stack)?;
                self.discharge(&stack_top, compile_stack)?;
                compile_stack.compile_context_mut().stack_top -= 1;

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
                let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_top.discharge(table_access, compile_stack)?;
                self.discharge(&stack_top, compile_stack)?;
                compile_stack.compile_context_mut().stack_top -= 1;

                Ok(())
            }
            _ => unimplemented!("Can't discharge {:?} into {:?}", src, self),
        }
    }

    fn discharge_into_condition(
        &self,
        src: &ExpDesc<'a>,
        compile_stack: &mut CompileStack<'a>,
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
                let Some(name) = compile_stack
                    .view()
                    .find_name(name)
                    .or_else(|| compile_stack.view().capture_name(name))
                    .or_else(|| compile_stack.view().capture_environment(name))
                else {
                    unreachable!("Should always fallback to Global.");
                };
                self.discharge(&name, compile_stack)
            }
            Self::Binop(Binop::Or, lhs, rhs) => {
                Self::Condition {
                    jump_to_end: false,
                    if_condition: true,
                }
                .discharge(lhs, compile_stack)?;
                Self::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(rhs, compile_stack)?;
                Ok(())
            }
            Self::Binop(Binop::And, lhs, rhs) => {
                let jumps_to_block = compile_stack.compile_context_mut().jumps_to_block.len();
                Self::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(lhs, compile_stack)?;

                Self::resolve_jumps_to_block(jumps_to_block, compile_stack)?;

                Self::Condition {
                    jump_to_end: true,
                    if_condition: false,
                }
                .discharge(rhs, compile_stack)?;
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
                    let Some(name) = compile_stack
                        .view()
                        .find_name(name)
                        .or_else(|| compile_stack.view().capture_name(name))
                        .or_else(|| compile_stack.view().capture_environment(name))
                    else {
                        unreachable!("Should always fallback to Global.");
                    };
                    self.discharge(
                        &Self::Binop(*op, Box::new(name), rhs.clone()),
                        compile_stack,
                    )
                }
                (_, _, Self::Name(name)) => {
                    let Some(name) = compile_stack
                        .view()
                        .find_name(name)
                        .or_else(|| compile_stack.view().capture_name(name))
                        .or_else(|| compile_stack.view().capture_environment(name))
                    else {
                        unreachable!("Should always fallback to Global.");
                    };
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(name)),
                        compile_stack,
                    )
                }
                (Binop::LessThan, Self::Local(local), Self::Integer(integer)) => {
                    if let Ok(integer) = i8::try_from(*integer) {
                        compile_stack
                            .proto_mut()
                            .byte_codes
                            .push(Bytecode::less_than_integer(
                                u8::try_from(*local)?.into(),
                                integer.into(),
                                (*if_condition).into(),
                            ));

                        let jump = compile_stack.proto_mut().byte_codes.len();
                        compile_stack
                            .proto_mut()
                            .byte_codes
                            .push(Bytecode::jump(Sj::ZERO));
                        if *jump_to_end {
                            compile_stack.compile_context_mut().jumps_to_end.push(jump);
                        } else {
                            compile_stack
                                .compile_context_mut()
                                .jumps_to_block
                                .push(jump);
                        }

                        Ok(())
                    } else {
                        let (_, stack_top) =
                            compile_stack.compile_context_mut().reserve_stack_top();
                        stack_top.discharge(rhs.as_ref(), compile_stack)?;
                        self.discharge(
                            &Self::Binop(*op, lhs.clone(), Box::new(stack_top)),
                            compile_stack,
                        )?;
                        compile_stack.compile_context_mut().stack_top -= 1;
                        Ok(())
                    }
                }
                (Binop::GreaterThan, Self::Local(local), Self::Integer(integer)) => {
                    if let Ok(integer) = i8::try_from(*integer) {
                        compile_stack
                            .proto_mut()
                            .byte_codes
                            .push(Bytecode::greater_than_integer(
                                u8::try_from(*local)?.into(),
                                integer.into(),
                                (*if_condition).into(),
                            ));

                        let jump = compile_stack.proto_mut().byte_codes.len();
                        compile_stack
                            .proto_mut()
                            .byte_codes
                            .push(Bytecode::jump(Sj::ZERO));
                        if *jump_to_end {
                            compile_stack.compile_context_mut().jumps_to_end.push(jump);
                        } else {
                            compile_stack
                                .compile_context_mut()
                                .jumps_to_block
                                .push(jump);
                        }

                        Ok(())
                    } else {
                        let (_, stack_top) =
                            compile_stack.compile_context_mut().reserve_stack_top();
                        stack_top.discharge(rhs.as_ref(), compile_stack)?;
                        self.discharge(
                            &Self::Binop(*op, lhs.clone(), Box::new(stack_top)),
                            compile_stack,
                        )?;
                        compile_stack.compile_context_mut().stack_top -= 1;
                        Ok(())
                    }
                }
                (Binop::GreaterThan, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::less_than(
                            u8::try_from(*rhs)?.into(),
                            u8::try_from(*lhs)?.into(),
                            (*if_condition).into(),
                        ));
                    let jump = compile_stack.proto_mut().byte_codes.len();
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(Sj::ZERO));
                    if *jump_to_end {
                        compile_stack.compile_context_mut().jumps_to_end.push(jump);
                    } else {
                        compile_stack
                            .compile_context_mut()
                            .jumps_to_block
                            .push(jump);
                    }

                    Ok(())
                }
                (Binop::LessEqual, Self::Local(_), string @ Self::String(_)) => {
                    let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                    stack_top.discharge(string, compile_stack)?;
                    self.discharge(
                        &Self::Binop(*op, lhs.clone(), Box::new(stack_top)),
                        compile_stack,
                    )?;
                    compile_stack.compile_context_mut().stack_top -= 1;
                    Ok(())
                }
                (Binop::LessEqual, Self::Local(lhs), Self::Local(rhs)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::less_equal(
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(*rhs)?.into(),
                            (*if_condition).into(),
                        ));
                    let jump = compile_stack.proto_mut().byte_codes.len();
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(Sj::ZERO));
                    if *jump_to_end {
                        compile_stack.compile_context_mut().jumps_to_end.push(jump);
                    } else {
                        compile_stack
                            .compile_context_mut()
                            .jumps_to_block
                            .push(jump);
                    }

                    Ok(())
                }
                (Binop::GreaterEqual, Self::Local(lhs), Self::Integer(integer)) => {
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::greater_equal_integer(
                            u8::try_from(*lhs)?.into(),
                            i8::try_from(*integer)?.into(),
                            (*if_condition).into(),
                        ));
                    let jump = compile_stack.proto_mut().byte_codes.len();
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(Sj::ZERO));
                    if *jump_to_end {
                        compile_stack.compile_context_mut().jumps_to_end.push(jump);
                    } else {
                        compile_stack
                            .compile_context_mut()
                            .jumps_to_block
                            .push(jump);
                    }

                    Ok(())
                }
                (Binop::Equal, Self::Local(lhs), Self::String(name)) => {
                    let constant = compile_stack.proto_mut().push_constant(*name)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::equal_constant(
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(constant)?.into(),
                            (*if_condition).into(),
                        ));
                    let jump = compile_stack.proto_mut().byte_codes.len();
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(Sj::ZERO));
                    if *jump_to_end {
                        compile_stack.compile_context_mut().jumps_to_end.push(jump);
                    } else {
                        compile_stack
                            .compile_context_mut()
                            .jumps_to_block
                            .push(jump);
                    }

                    Ok(())
                }
                (Binop::Equal, Self::Local(lhs), Self::Integer(integer)) => {
                    let constant = compile_stack.proto_mut().push_constant(*integer)?;
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::equal_constant(
                            u8::try_from(*lhs)?.into(),
                            u8::try_from(constant)?.into(),
                            (*if_condition).into(),
                        ));
                    let jump = compile_stack.proto_mut().byte_codes.len();
                    compile_stack
                        .proto_mut()
                        .byte_codes
                        .push(Bytecode::jump(Sj::ZERO));
                    if *jump_to_end {
                        compile_stack.compile_context_mut().jumps_to_end.push(jump);
                    } else {
                        compile_stack
                            .compile_context_mut()
                            .jumps_to_block
                            .push(jump);
                    }

                    Ok(())
                }
                _ => unimplemented!("Can't discharge binary operation {:?}.", src),
            },
            Self::Local(local) => {
                compile_stack.proto_mut().byte_codes.push(Bytecode::test(
                    u8::try_from(*local)?.into(),
                    (*if_condition).into(),
                ));
                let jump = compile_stack.proto_mut().byte_codes.len();
                compile_stack
                    .proto_mut()
                    .byte_codes
                    .push(Bytecode::jump(Sj::ZERO));
                if *jump_to_end {
                    compile_stack.compile_context_mut().jumps_to_end.push(jump);
                } else {
                    compile_stack
                        .compile_context_mut()
                        .jumps_to_block
                        .push(jump);
                }
                Ok(())
            }
            global @ Self::Global(_) => {
                let (_, stack_top) = compile_stack.compile_context_mut().reserve_stack_top();
                stack_top.discharge(global, compile_stack)?;
                self.discharge(&stack_top, compile_stack)?;
                compile_stack.compile_context_mut().stack_top -= 1;
                Ok(())
            }
            _ => unimplemented!("Can't make condition out of {:?}.", src),
        }
    }

    pub fn get_local_or_discharge_at_location(
        &self,
        compile_stack: &mut CompileStack<'a>,
        location: u8,
    ) -> Result<u8, Error> {
        match self {
            ExpDesc::Name(table_name) => {
                match compile_stack.compile_context_mut().find_name(table_name) {
                    Some(pos) => u8::try_from(pos).map_err(Error::from),
                    None => {
                        self.discharge(&ExpDesc::Local(usize::from(location)), compile_stack)?;
                        Ok(location)
                    }
                }
            }
            other => unreachable!(
                "Should always be called with a ExpDesc::Name, but was called with {:?}.",
                other
            ),
        }
    }

    fn resolve_jumps_to_block(
        start_of_jumps_to_resolve: usize,
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        let CompileFrame {
            proto,
            compile_context,
        } = compile_stack.frame_mut();

        let jumps_to_block = compile_context
            .jumps_to_block
            .drain(start_of_jumps_to_resolve..)
            .collect::<Vec<_>>();
        let jump_dst = proto.byte_codes.len();
        for jump in jumps_to_block {
            proto.byte_codes[jump] = Bytecode::jump(
                i32::try_from(jump_dst - jump - 1)
                    .map_err(|_| Error::LongJump)?
                    .try_into()?,
            );
        }
        Ok(())
    }

    fn resolve_jumps_to_end(
        start_of_jumps_to_resolve: usize,
        compile_stack: &mut CompileStack<'a>,
    ) -> Result<(), Error> {
        let CompileFrame {
            proto,
            compile_context,
        } = compile_stack.frame_mut();

        let jumps_to_end = compile_context
            .jumps_to_end
            .drain(start_of_jumps_to_resolve..)
            .collect::<Vec<_>>();
        let jump_dst = proto.byte_codes.len();
        for jump in jumps_to_end {
            proto.byte_codes[jump] = Bytecode::jump(
                i32::try_from(jump_dst - jump - 1)
                    .map_err(|_| Error::LongJump)?
                    .try_into()?,
            );
        }
        Ok(())
    }
}
