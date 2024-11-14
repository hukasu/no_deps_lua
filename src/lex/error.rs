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
    ParseInt,
    ParseFloat,
    ProhibtedControlCharacterOnString,
    Uninmplemented,
    OctalNotSupported,
    LeadingZero,
    MalformedFloat,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            ErrorKind::EofAtString => {
                write!(f, "Reached End of File while reading a String.",)
            }
            ErrorKind::ParseInt => {
                write!(f, "Could not parse an number into an integer.",)
            }
            ErrorKind::ParseFloat => {
                write!(f, "Could not parse an number into an float.",)
            }
            ErrorKind::LeadingZero => {
                write!(f, "Non hexadecimal numbers can't start with leading zeros.",)
            }
            ErrorKind::OctalNotSupported => {
                write!(f, "Octal numbers are not supported.",)
            }
            ErrorKind::MalformedFloat => {
                write!(f, "Floating-point number was malformed.",)
            }
            ErrorKind::ProhibtedControlCharacterOnString => {
                write!(f, "A control character was found in a comment.",)
            }
            ErrorKind::Uninmplemented => {
                write!(f, "Unimplemented")
            }
        }
    }
}

impl core::error::Error for Error {}
