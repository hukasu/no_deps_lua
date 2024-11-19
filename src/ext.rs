use core::fmt::Display;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

pub trait Unescape {
    fn unescape(&self) -> Result<String, UnescapeError>;
}

impl Unescape for &str {
    fn unescape(&self) -> Result<String, UnescapeError> {
        let mut vec = Vec::with_capacity(self.len());
        let mut iter = self.chars().peekable();
        let mut buffer = [0; 4];

        while let Some(c) = iter.next() {
            if c == '\\' {
                match iter.next() {
                    Some('a') => vec.push(7),
                    Some('b') => vec.push(8),
                    Some('f') => vec.push(12),
                    Some('n') => vec.push(b'\n'),
                    Some('r') => vec.push(b'\r'),
                    Some('t') => vec.push(b'\t'),
                    Some('v') => vec.push(11),
                    Some('\\') => vec.push(b'\\'),
                    Some('"') => vec.push(b'"'),
                    Some('\'') => vec.push(b'\''),
                    Some(d @ '0'..='9') => {
                        let mut three_digit_escaped = [d].to_vec();

                        if matches!(iter.peek(), Some('0'..='9')) {
                            let Some(d2) = iter.next() else {
                                unreachable!("Already checked that it is Some.")
                            };
                            three_digit_escaped.push(d2);
                        }
                        if matches!(iter.peek(), Some('0'..='9')) {
                            let Some(d3) = iter.next() else {
                                unreachable!("Already checked that it is Some.")
                            };
                            three_digit_escaped.push(d3);
                        }

                        let Ok(ordinal) =
                            three_digit_escaped.into_iter().collect::<String>().parse()
                        else {
                            unreachable!("Vec only has digits.");
                        };

                        let c = unsafe { char::from_u32_unchecked(ordinal) };
                        c.encode_utf8(&mut buffer);
                        vec.extend(&buffer[..c.len_utf8()]);
                    }
                    Some('x') => {
                        let Some(&d1 @ ('A'..='F' | 'a'..='f' | '0'..='9')) = iter.peek() else {
                            return Err(UnescapeError::MalformedHexChar);
                        };
                        iter.next();
                        let Some(&d2 @ ('A'..='F' | 'a'..='f' | '0'..='9')) = iter.peek() else {
                            return Err(UnescapeError::MalformedHexChar);
                        };
                        iter.next();

                        let Some(d1) = d1.to_digit(16) else {
                            unreachable!("Already checked that is hex digit.");
                        };
                        let Some(d2) = d2.to_digit(16) else {
                            unreachable!("Already checked that is hex digit.");
                        };

                        let Ok(ordinal) = u8::try_from(d1 * 16 + d2) else {
                            unreachable!("Sum of 2 hex digits shouldn't overflow a u8.");
                        };

                        vec.push(ordinal);
                    }
                    Some(_) => return Err(UnescapeError::UnknownEscapedCharacter),
                    None => return Err(UnescapeError::UnfinishedEscapedCharacter),
                }
            } else {
                c.encode_utf8(&mut buffer);
                vec.extend(&buffer[..c.len_utf8()]);
            }
        }

        let result = String::from_utf8_lossy(&vec).to_string();
        Ok(result)
    }
}

#[derive(Debug)]
pub enum UnescapeError {
    UnfinishedEscapedCharacter,
    UnknownEscapedCharacter,
    MalformedHexChar,
}

impl Display for UnescapeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnfinishedEscapedCharacter => {
                write!(
                    f,
                    "String reach its end while processing a escaped character."
                )
            }
            Self::UnknownEscapedCharacter => {
                write!(f, "Tried processing a unknown escaped character.")
            }
            Self::MalformedHexChar => {
                write!(f, "A hex escaped character did not have 2 digits.")
            }
        }
    }
}

impl core::error::Error for UnescapeError {}
