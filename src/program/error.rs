use core::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse,
    Unimplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse => {
                write!(f, "Could not parse program.")
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
