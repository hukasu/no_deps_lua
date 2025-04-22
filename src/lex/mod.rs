mod error;
mod lexeme;
mod states;
#[cfg(test)]
mod tests;

use alloc::{vec, vec::Vec};
use core::{iter::Peekable, str::Chars};
use error::ErrorKind;
use states::StateError;

use self::states::State;
pub use self::{
    error::Error,
    lexeme::{Lexeme, LexemeType},
};

pub struct Lex<'a> {
    program: &'a str,
    chars: Peekable<Chars<'a>>,
    state: State,
    /// Current `char` being read
    seek: usize,
    /// Start of lexeme being considered
    start: usize,
    lines: Vec<usize>,
}

impl<'a> Lex<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            program: data,
            chars: data.chars().peekable(),
            state: State::Start,
            seek: 0,
            start: 0,
            lines: vec![0],
        }
    }

    #[cfg(test)]
    pub fn remaining(&self) -> usize {
        self.program.len() - self.seek
    }

    fn build_lexeme(&self, state: State) -> Option<Result<Lexeme<'a>, Error>> {
        let mut line = self.lines.len() - 1;
        let column = {
            let mut rev = self.lines.iter().rev();
            if let Some(column) = rev.next().filter(|column| **column != 0) {
                *column
            } else {
                let Some(column) = rev.next() else {
                    unreachable!(
                        "Lexer must have read more than one line for it to have the current line at column 0."
                    );
                };
                line -= 1;
                *column
            }
        };

        let make_lexeme = |lexeme_type| Lexeme {
            line,
            column: column - 1,
            start: self.start - 1,
            lexeme_type,
        };

        match state {
            State::Start | State::ShortComment => None,
            State::Add => Some(Ok(make_lexeme(LexemeType::Add))),
            State::Sub => Some(Ok(make_lexeme(LexemeType::Sub))),
            State::Mul => Some(Ok(make_lexeme(LexemeType::Mul))),
            State::Div => Some(Ok(make_lexeme(LexemeType::Div))),
            State::Concat => Some(Ok(make_lexeme(LexemeType::Concat))),
            State::BitAnd => Some(Ok(make_lexeme(LexemeType::BitAnd))),
            State::BitOr => Some(Ok(make_lexeme(LexemeType::BitOr))),
            State::BitXor => Some(Ok(make_lexeme(LexemeType::BitXor))),
            State::ShiftRight => Some(Ok(make_lexeme(LexemeType::ShiftR))),
            State::ShiftLeft => Some(Ok(make_lexeme(LexemeType::ShiftL))),
            State::Len => Some(Ok(make_lexeme(LexemeType::Len))),
            State::Assign => Some(Ok(make_lexeme(LexemeType::Assign))),
            State::Less => Some(Ok(make_lexeme(LexemeType::Less))),
            State::LessEqual => Some(Ok(make_lexeme(LexemeType::Leq))),
            State::Greater => Some(Ok(make_lexeme(LexemeType::Greater))),
            State::GreaterEqual => Some(Ok(make_lexeme(LexemeType::Geq))),
            State::Equal => Some(Ok(make_lexeme(LexemeType::Eq))),
            State::NotEqual => Some(Ok(make_lexeme(LexemeType::Neq))),
            State::LParen => Some(Ok(make_lexeme(LexemeType::LParen))),
            State::RParen => Some(Ok(make_lexeme(LexemeType::RParen))),
            State::LSquare => Some(Ok(make_lexeme(LexemeType::LSquare))),
            State::RSquare => Some(Ok(make_lexeme(LexemeType::RSquare))),
            State::LCurly => Some(Ok(make_lexeme(LexemeType::LCurly))),
            State::RCurly => Some(Ok(make_lexeme(LexemeType::RCurly))),
            State::Comma => Some(Ok(make_lexeme(LexemeType::Comma))),
            State::Colon => Some(Ok(make_lexeme(LexemeType::Colon))),
            State::DoubleColon => Some(Ok(make_lexeme(LexemeType::DoubleColon))),
            State::SemiColon => Some(Ok(make_lexeme(LexemeType::SemiColon))),
            State::Dot => Some(Ok(make_lexeme(LexemeType::Dot))),
            State::Dots => Some(Ok(make_lexeme(LexemeType::Dots))),
            State::Number => {
                let start = self.start - 1;
                let end = self.seek - 1;
                let data = &self.program[start..end];
                let Ok(integer) = data.parse() else {
                    return Some(Err(Error {
                        kind: ErrorKind::ParseInt,
                        line,
                        column: column - 1,
                    }));
                };

                Some(Ok(make_lexeme(LexemeType::Integer(integer))))
            }
            State::Float => {
                let start = self.start - 1;
                let end = self.seek - 1;
                let data = &self.program[start..end];
                let Ok(float) = data.parse() else {
                    return Some(Err(Error {
                        kind: ErrorKind::ParseFloat,
                        line,
                        column: column - 1,
                    }));
                };

                Some(Ok(make_lexeme(LexemeType::Float(float))))
            }
            State::String(quotes)
            | State::StringAscii(quotes, _, _)
            | State::StringUtf8(quotes, 2) => {
                let start = self.start - 1;
                let end = self.seek - 1;
                let data = &self.program[start..end].trim_matches(quotes);

                Some(Ok(Lexeme {
                    line,
                    column,
                    start,
                    lexeme_type: LexemeType::String(data),
                }))
            }
            State::Name => {
                let start = self.start - 1;
                let end = if self.state == State::Eof {
                    self.seek
                } else {
                    self.seek - 1
                };
                let data = &self.program[start..end];

                let make_lexeme = |lexeme_type| Lexeme {
                    line,
                    column: column - 1,
                    start,
                    lexeme_type,
                };

                match data {
                    "and" => Some(Ok(make_lexeme(LexemeType::And))),
                    "break" => Some(Ok(make_lexeme(LexemeType::Break))),
                    "do" => Some(Ok(make_lexeme(LexemeType::Do))),
                    "else" => Some(Ok(make_lexeme(LexemeType::Else))),
                    "elseif" => Some(Ok(make_lexeme(LexemeType::Elseif))),
                    "end" => Some(Ok(make_lexeme(LexemeType::End))),
                    "false" => Some(Ok(make_lexeme(LexemeType::False))),
                    "for" => Some(Ok(make_lexeme(LexemeType::For))),
                    "function" => Some(Ok(make_lexeme(LexemeType::Function))),
                    "goto" => Some(Ok(make_lexeme(LexemeType::Goto))),
                    "if" => Some(Ok(make_lexeme(LexemeType::If))),
                    "in" => Some(Ok(make_lexeme(LexemeType::In))),
                    "local" => Some(Ok(make_lexeme(LexemeType::Local))),
                    "nil" => Some(Ok(make_lexeme(LexemeType::Nil))),
                    "not" => Some(Ok(make_lexeme(LexemeType::Not))),
                    "or" => Some(Ok(make_lexeme(LexemeType::Or))),
                    "repeat" => Some(Ok(make_lexeme(LexemeType::Repeat))),
                    "return" => Some(Ok(make_lexeme(LexemeType::Return))),
                    "then" => Some(Ok(make_lexeme(LexemeType::Then))),
                    "true" => Some(Ok(make_lexeme(LexemeType::True))),
                    "until" => Some(Ok(make_lexeme(LexemeType::Until))),
                    "while" => Some(Ok(make_lexeme(LexemeType::While))),
                    _ => Some(Ok(make_lexeme(LexemeType::Name(data)))),
                }
            }
            other => {
                unimplemented!("{:?}", other);
            }
        }
    }
}

impl<'a> Iterator for Lex<'a> {
    type Item = Result<Lexeme<'a>, Error>;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item>
    where
        Self: 'a,
    {
        loop {
            if self.start == usize::MAX {
                break None;
            }

            let consumed = if let Some(c) = self.chars.next() {
                self.seek += c.len_utf8();
                if let Some(column) = self.lines.last_mut() {
                    *column += c.len_utf8();
                } else {
                    unreachable!("List of line columns should never be empty.");
                }
                if c == '\n' {
                    self.lines.push(0);
                }
                self.state.consume(c)
            } else if self.start != self.seek {
                self.state.consume_eof()
            } else {
                let start = self.start;
                self.start = usize::MAX;

                break Some(Ok(Lexeme {
                    line: self.lines.len() - 1,
                    column: self.lines.last().copied().unwrap_or_default(),
                    start,
                    lexeme_type: LexemeType::Eof,
                }));
            };

            match consumed {
                Ok(consumed) => {
                    if let Some(state) = consumed {
                        let lexeme = self.build_lexeme(state);
                        self.start = self.seek;

                        if lexeme.is_some() {
                            break lexeme;
                        }
                    }
                }
                Err(StateError::EofAtString) => {
                    return Some(Err(Error {
                        kind: ErrorKind::EofAtString,
                        line: self.lines.len() - 1,
                        column: self.lines.last().copied().unwrap_or_default(),
                    }));
                }
                Err(
                    err @ (StateError::EscapedChar(_)
                    | StateError::HexCharacter(_)
                    | StateError::AsciiOutOfBounds(_)),
                ) => {
                    log::error!("{}", err);
                    return Some(Err(Error {
                        kind: ErrorKind::ProhibtedControlCharacterOnString,
                        line: self.lines.len() - 1,
                        column: self.lines.last().copied().unwrap_or_default(),
                    }));
                }
            }
        }
    }
}
