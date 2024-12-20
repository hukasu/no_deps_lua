use core::{fmt::Display, num::TryFromIntError};

use crate::value::Value;

#[derive(Debug)]
pub enum Error {
    InvalidGlobalKey(Value),
    InvalidFunction(Value),
    ExpectedName,
    ExpectedTable,
    // Unary operators
    InvalidLenOperand,
    InvalidNegOperand,
    InvalidBitNotOperand,
    // Binary arithmetic operators
    NilArithmetic,
    BoolArithmetic,
    StringArithmetic,
    TableArithmetic,
    FunctionArithmetic,
    // Binary bitwise operators
    FloatBitwise,
    NilBitwise,
    BoolBitwise,
    StringBitwise,
    TableBitwise,
    FunctionBitwise,
    // Concat
    NilConcat,
    BoolConcat,
    TableConcat,
    FunctionConcat,
    // Other
    TryFloatConversion,
    IntegerConversion,
    ForZeroStep,
    StackOverflow,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidGlobalKey(value) => write!(f, "Global {:?} is not a String.", value),
            Self::InvalidFunction(value) => write!(f, "Value {:?} is not a function.", value),
            Self::ExpectedName => write!(f, "Expected global or local name."),
            Self::ExpectedTable => write!(f, "Tried accessing a value as a Table."),
            Self::InvalidLenOperand => write!(f, "Len can only operate over String."),
            Self::InvalidNegOperand => write!(f, "Neg can only operate over Integers and Floats."),
            Self::InvalidBitNotOperand => write!(f, "BitNot can only operate over Integers."),
            Self::NilArithmetic => write!(f, "{}", crate::program::Error::NilArithmetic),
            Self::BoolArithmetic => write!(f, "{}", crate::program::Error::BoolArithmetic),
            Self::StringArithmetic => write!(f, "{}", crate::program::Error::StringArithmetic),
            Self::TableArithmetic => write!(f, "{}", crate::program::Error::TableArithmetic),
            Self::FunctionArithmetic => write!(f, "Can't use Function in arithmetic operations."),
            Self::FloatBitwise => write!(f, "{}", crate::program::Error::FloatBitwise),
            Self::NilBitwise => write!(f, "{}", crate::program::Error::NilBitwise),
            Self::BoolBitwise => write!(f, "{}", crate::program::Error::BoolBitwise),
            Self::StringBitwise => write!(f, "{}", crate::program::Error::StringBitwise),
            Self::TableBitwise => write!(f, "{}", crate::program::Error::TableBitwise),
            Self::FunctionBitwise => write!(f, "Can't use Function in bitwise operations."),
            Self::NilConcat => write!(f, "{}", crate::program::Error::NilConcat),
            Self::BoolConcat => write!(f, "{}", crate::program::Error::BoolConcat),
            Self::TableConcat => write!(f, "{}", crate::program::Error::TableConcat),
            Self::FunctionConcat => write!(f, "Can't use Function in concat operations."),
            Self::TryFloatConversion => write!(f, "Failed to convert Value to Value::Float."),
            Self::IntegerConversion => write!(
                f,
                "Tried converting an integer that does not fit into a i64."
            ),
            Self::ForZeroStep => write!(f, "For loop had a step of zero."),
            Self::StackOverflow => write!(f, "Vm's stack has overflown."),
        }
    }
}

impl core::error::Error for Error {}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        log::error!(target: "no_deps_lua::vm", "{value}");
        Self::IntegerConversion
    }
}
