use core::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Error {
    pub(super) kind: ErrorKind,
    pub(super) line: usize,
    pub(super) column: usize,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    EofAtString,
    ProhibtedControlCharacterOnComment,
    CharacterAfterEof,
    Uninmplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            ErrorKind::EofAtString => {
                write!(f, "Reached End of File while reading a String.",)
            }
            ErrorKind::ProhibtedControlCharacterOnComment => {
                write!(f, "A control character was found in a comment.",)
            }
            ErrorKind::CharacterAfterEof => {
                write!(f, "Lexer received a character after Eof.")
            }
            ErrorKind::Uninmplemented => {
                write!(f, "Unimplemented")
            }
        }
    }
}

impl core::error::Error for Error {}
