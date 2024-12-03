use core::{fmt::Display, num::TryFromIntError};

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse,
    StringDecode,
    OrphanExp,
    NilTableIndex,
    // Binary arithmetic operators
    NilArithmetic,
    BoolArithmetic,
    StringArithmetic,
    TableArithmetic,
    // Binary bitwise operators
    FloatBitwise,
    NilBitwise,
    BoolBitwise,
    StringBitwise,
    TableBitwise,
    // Concat
    NilConcat,
    BoolConcat,
    TableConcat,
    // Others
    StackOverflow,
    LongJump,
    Unimplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse => {
                write!(f, "Could not parse program.")
            }
            Self::StringDecode => {
                write!(f, "Failed to decode string.")
            }
            Self::OrphanExp => {
                write!(f, "Exp had nowhere to be stored.")
            }
            Self::NilTableIndex => {
                write!(f, "Can't use Nil as index to a table.")
            }
            Self::NilArithmetic => {
                write!(f, "Can't use Nil in arithmetic operations.")
            }
            Self::BoolArithmetic => {
                write!(f, "Can't use Boolean in arithmetic operations.")
            }
            Self::StringArithmetic => {
                write!(f, "Can't use String in arithmetic operations.")
            }
            Self::TableArithmetic => {
                write!(f, "Can't use Table in arithmetic operations.")
            }
            Self::FloatBitwise => {
                write!(f, "Can't use Float in bitwise operations.")
            }
            Self::NilBitwise => {
                write!(f, "Can't use Nil in bitwise operations.")
            }
            Self::BoolBitwise => {
                write!(f, "Can't use Boolean in bitwise operations.")
            }
            Self::StringBitwise => {
                write!(f, "Can't use String in bitwise operations.")
            }
            Self::TableBitwise => {
                write!(f, "Can't use Table in arithmetic operations.")
            }
            Self::NilConcat => {
                write!(f, "Can't use Nil in concat operations.")
            }
            Self::BoolConcat => {
                write!(f, "Can't use Boolean in concat operations.")
            }
            Self::TableConcat => {
                write!(f, "Can't use Table in concat operations.")
            }
            Self::StackOverflow => {
                write!(f, "Tried accessing index outside stack bounds.")
            }
            Self::LongJump => {
                write!(f, "Jump is longer than a i16.")
            }
            Self::Unimplemented => {
                write!(f, "Feature is not implemented.")
            }
        }
    }
}

impl core::error::Error for Error {}

impl From<crate::parser::Error> for Error {
    fn from(value: crate::parser::Error) -> Self {
        log::error!(target: "no_deps_lua::parser", "{:?}", value);
        Self::Parse
    }
}

impl From<crate::ext::UnescapeError> for Error {
    fn from(value: crate::ext::UnescapeError) -> Self {
        log::error!(target: "no_deps_lua::parser", "{:?}", value);
        Self::StringDecode
    }
}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        log::error!(target: "no_deps_lua::parser", "{:?}", value);
        Self::StackOverflow
    }
}
