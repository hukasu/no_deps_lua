use core::fmt::Display;

use log::error;

#[derive(Debug, PartialEq)]
pub enum Error {
    LexError,
    Unimplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LexError => {
                write!(f, "Could not parse program due to lexical error.")
            }
            Self::Unimplemented => {
                write!(f, "Feature is not implemented.")
            }
        }
    }
}

impl core::error::Error for Error {}

impl From<crate::lex::Error> for Error {
    fn from(value: crate::lex::Error) -> Self {
        error!(target: "lua_program", "{:?}", value);
        Self::LexError
    }
}
