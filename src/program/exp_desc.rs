use core::u8;

use alloc::boxed::Box;

use crate::ext::Unescape;

use super::{ByteCode, Error, Program};

#[derive(Debug, Clone)]
pub enum ExpDesc<'a> {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    Local(usize),
    Global(usize),
    TableLocal(usize, Box<ExpDesc<'a>>),
}

impl<'a> ExpDesc<'a> {
    pub fn discharge(self, dst: ExpDesc<'a>, program: &mut Program) -> Result<(), Error> {
        match (&self, &dst) {
            (Self::Nil, Self::Local(dst)) => u8::try_from(*dst).map_err(Error::from).map(|dst| {
                program.byte_codes.push(ByteCode::LoadNil(dst));
            }),
            (Self::Boolean(boolean), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(ByteCode::LoadBool(dst, *boolean));

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
            (Self::Float(float), Self::Local(dst)) => {
                let dst = u8::try_from(*dst)?;
                let position = program.push_constant(*float)?;

                program
                    .byte_codes
                    .push(ByteCode::LoadConstant(dst, position));

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
            (Self::Global(key), Self::Local(dst)) => {
                let key = u8::try_from(*key)?;
                let dst = u8::try_from(*dst)?;

                program.byte_codes.push(ByteCode::GetGlobal(dst, key));

                Ok(())
            }
            (Self::Nil, Self::Global(key)) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(())?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

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
            (Self::Float(float), Self::Global(key)) => {
                let key = u8::try_from(*key)?;
                let constant = program.push_constant(*float)?;

                program
                    .byte_codes
                    .push(ByteCode::SetGlobalConstant(key, constant));

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
            (Self::Local(src), Self::Global(key)) => {
                let src = u8::try_from(*src)?;
                let key = u8::try_from(*key)?;

                program.byte_codes.push(ByteCode::SetGlobal(key, src));

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
            (Self::TableLocal(table, exp), Self::Local(dst)) => {
                let table = u8::try_from(*table)?;
                let dst = u8::try_from(*dst)?;

                match exp.as_ref() {
                    ExpDesc::Integer(integer) => {
                        if let Ok(i) = u8::try_from(*integer) {
                            program.byte_codes.push(ByteCode::GetInt(dst, table, i));
                            Ok(())
                        } else {
                            log::error!("Only index on small integers is supported.");
                            Err(Error::Unimplemented)
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
                        log::error!("Only index on small integers is supported.");
                        Err(Error::Unimplemented)
                    }
                }
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
