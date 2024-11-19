use core::fmt::Display;

use crate::value::Value;

#[derive(Debug)]
pub enum Error<'a> {
    InvalidGlobalKey(Value<'a>),
    InvalidFunction(Value<'a>),
    ExpectedName,
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidGlobalKey(value) => write!(f, "Global {:?} is not a String.", value),
            Self::InvalidFunction(value) => write!(f, "Value {:?} is not a function.", value),
            Self::ExpectedName => write!(f, "Expected global or local name."),
        }
    }
}

impl<'a> core::error::Error for Error<'a> {}
