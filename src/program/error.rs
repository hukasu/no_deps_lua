use core::{fmt::Display, num::TryFromIntError};

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse,
    StringDecode,
    OrphanExp,
    NilTableIndex,
    NilArithmetic,
    BoolArithmetic,
    StringArithmetic,
    TableArithmetic,
    StackOverflow,
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
            Self::StackOverflow => {
                write!(f, "Tried accessing index outside stack bounds.")
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
