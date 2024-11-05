use core::fmt::Display;

use log::error;

use crate::lex::Token;

#[derive(Debug, PartialEq)]
pub enum Error<'a> {
    LexFailure,
    InvalidTokenAfterName(Token<'a>),
    Unimplemented,
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LexFailure => {
                write!(f, "Could not parse program due to lexical error.")
            }
            Self::InvalidTokenAfterName(token) => {
                write!(
                    f,
                    "Found invalid token {:?} at line {}, column {}.",
                    token.token, token.line, token.column
                )
            }
            Self::Unimplemented => {
                write!(f, "Feature is not implemented.")
            }
        }
    }
}

impl<'a> core::error::Error for Error<'a> {}

impl<'a> From<crate::lex::Error> for Error<'a> {
    fn from(value: crate::lex::Error) -> Self {
        error!(target: "lua_program", "{:?}", value);
        Self::LexFailure
    }
}
