use core::{fmt::Display, num::TryFromIntError};

use alloc::boxed::Box;

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse,
    StringDecode,
    OrphanExp,
    IncompatibleConditional,
    // TableAccess
    TableRecordAccess(&'static str),
    // Binary arithmetic operators
    NotBinaryOperator,
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
    LongJump,
    BreakOutsideLoop,
    LabelRedefinition,
    StackOverflow,
    UnmatchedGoto,
    IntCoversion,
    GotoIntoScope,
    NonSequentialLocalInitialization(Box<str>),
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
            Self::TableRecordAccess(key_type) => {
                write!(f, "Can't access table using {}.", key_type)
            }
            Self::IncompatibleConditional => {
                write!(
                    f,
                    "A conditional was not a variable, global, constant, or logical operation."
                )
            }
            Self::NotBinaryOperator => {
                write!(f, "Token was not a binary operator.")
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
            Self::LongJump => {
                write!(f, "Jump is longer than a i16.")
            }
            Self::BreakOutsideLoop => {
                write!(f, "Break outside of loop.")
            }
            Self::LabelRedefinition => {
                write!(f, "Label is already defined.")
            }
            Self::UnmatchedGoto => {
                write!(f, "Label was not visible for goto.")
            }
            Self::GotoIntoScope => {
                write!(f, "Jumping into scope of local.")
            }
            Self::IntCoversion => {
                write!(f, "Failed to convert an integer.")
            }
            Self::StackOverflow => {
                write!(f, "Tried accessing index outside stack bounds.")
            }
            Self::NonSequentialLocalInitialization(explist) => {
                write!(
                    f,
                    "Local initialization need to have the locations in sequential order, but was {:?}.",
                    explist
                )
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
