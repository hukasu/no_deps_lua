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
}

impl<'a> ExpDesc<'a> {
    pub fn discharge(self, dst: ExpDesc<'a>, program: &mut Program) -> Result<(), Error> {
        match (self, dst) {
            (Self::Nil, Self::Local(dst)) => u8::try_from(dst).map_err(Error::from).map(|dst| {
                program.byte_codes.push(ByteCode::LoadNil(dst));
            }),
            (Self::Boolean(boolean), Self::Local(dst)) => {
                u8::try_from(dst).map_err(Error::from).map(|dst| {
                    program.byte_codes.push(ByteCode::LoadBool(dst, boolean));
                })
            }
            (Self::Integer(integer), Self::Local(dst)) => u8::try_from(dst)
                .map_err(Error::from)
                .and_then(|dst| {
                    if let Ok(i) = i16::try_from(integer) {
                        Ok(ByteCode::LoadInt(dst, i))
                    } else {
                        program
                            .push_constant(integer)
                            .map(|position| ByteCode::LoadConstant(dst, position))
                    }
                })
                .map(|code| {
                    program.byte_codes.push(code);
                }),
            (Self::Float(float), Self::Local(dst)) => {
                u8::try_from(dst).map_err(Error::from).and_then(|dst| {
                    program.push_constant(float).map(|position| {
                        program
                            .byte_codes
                            .push(ByteCode::LoadConstant(dst, position));
                    })
                })
            }
            (Self::String(string), Self::Local(dst)) => {
                u8::try_from(dst).map_err(Error::from).and_then(|dst| {
                    string
                        .unescape()
                        .map_err(Error::from)
                        .and_then(|string| program.push_constant(string.as_str()))
                        .map(|position| {
                            program
                                .byte_codes
                                .push(ByteCode::LoadConstant(dst, position));
                        })
                })
            }
            (Self::Local(src), Self::Local(dst)) => {
                if src == dst {
                    Ok(())
                } else {
                    u8::try_from(src)
                        .and_then(|src| u8::try_from(dst).map(|dst| (src, dst)))
                        .map_err(Error::from)
                        .map(|(src, dst)| {
                            program.byte_codes.push(ByteCode::Move(dst, src));
                        })
                }
            }
            (Self::Global(key), Self::Local(dst)) => u8::try_from(key)
                .and_then(|key| u8::try_from(dst).map(|dst| (key, dst)))
                .map_err(Error::from)
                .map(|(key, dst)| {
                    program.byte_codes.push(ByteCode::GetGlobal(dst, key));
                }),
            (Self::Nil, Self::Global(key)) => {
                u8::try_from(key).map_err(Error::from).and_then(|key| {
                    program.push_constant(()).map(|constant| {
                        program
                            .byte_codes
                            .push(ByteCode::SetGlobalConstant(key, constant));
                    })
                })
            }
            (Self::Boolean(boolean), Self::Global(key)) => {
                u8::try_from(key).map_err(Error::from).and_then(|key| {
                    program.push_constant(boolean).map(|constant| {
                        program
                            .byte_codes
                            .push(ByteCode::SetGlobalConstant(key, constant));
                    })
                })
            }
            (Self::Integer(integer), Self::Global(key)) => u8::try_from(key)
                .map_err(Error::from)
                .and_then(|key| {
                    if let Ok(i) = i16::try_from(integer) {
                        Ok(ByteCode::SetGlobalInteger(key, i))
                    } else {
                        program
                            .push_constant(integer)
                            .map(|constant| ByteCode::SetGlobalConstant(key, constant))
                    }
                })
                .map(|code| {
                    program.byte_codes.push(code);
                }),
            (Self::Float(float), Self::Global(key)) => {
                u8::try_from(key).map_err(Error::from).and_then(|key| {
                    program.push_constant(float).map(|constant| {
                        program
                            .byte_codes
                            .push(ByteCode::SetGlobalConstant(key, constant));
                    })
                })
            }
            (Self::String(string), Self::Global(key)) => {
                u8::try_from(key).map_err(Error::from).and_then(|key| {
                    string
                        .unescape()
                        .map_err(Error::from)
                        .and_then(|string| program.push_constant(string.as_str()))
                        .map(|constant| {
                            program
                                .byte_codes
                                .push(ByteCode::SetGlobalConstant(key, constant));
                        })
                })
            }
            (Self::Local(src), Self::Global(key)) => u8::try_from(src)
                .and_then(|src| u8::try_from(key).map(|key| (src, key)))
                .map_err(Error::from)
                .map(|(src, key)| {
                    program.byte_codes.push(ByteCode::SetGlobal(key, src));
                }),
            (Self::Global(src_key), Self::Global(dst_key)) => u8::try_from(src_key)
                .and_then(|key| u8::try_from(dst_key).map(|dst_key| (key, dst_key)))
                .map_err(Error::from)
                .map(|(src_key, dst_key)| {
                    program
                        .byte_codes
                        .push(ByteCode::SetGlobalGlobal(dst_key, src_key));
                }),
            _ => {
                log::error!("Unimplemented discharge");
                Err(Error::Unimplemented)?
            }
        }
    }
}
