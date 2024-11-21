use core::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Accept,
    Reduction,
    Lex,
    Unimplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Accept => {
                write!(f, "Could not accept program due to malformed stack.")
            }
            Self::Reduction => {
                write!(f, "Could not reduce production due to malformed stack.")
            }
            Self::Lex => {
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
        log::error!(
            "{}",
            value
        );
        Self::Lex
    }
}
