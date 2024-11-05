use alloc::{string::ToString, vec::Vec};

use super::{Error, ErrorKind, Token};

#[derive(Debug)]
pub struct State {
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) machine: StateMachine,
    pub(super) buffer: Vec<u8>,
}

impl State {
    pub fn process(&mut self, c: u8) -> Option<Result<Token, Error>> {
        let processed = match self.machine {
            StateMachine::Start => match c {
                b'\0' => {
                    self.machine = StateMachine::End;
                    None
                }
                b' ' | b'\r' | b'\n' | b'\t' => None,
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                    self.buffer.push(c);
                    self.machine = StateMachine::Name;
                    None
                }
                b'"' => {
                    self.machine = StateMachine::String;
                    None
                }
                b'-' => {
                    self.machine = StateMachine::SeenMinus;
                    None
                }
                _ => Some(Err(Error {
                    kind: ErrorKind::Uninmplemented,
                    line: self.line,
                    column: self.column,
                })),
            },
            StateMachine::Name => match c {
                b'\0' => {
                    self.machine = StateMachine::End;
                    let data = self
                        .buffer
                        .drain(..)
                        .collect::<Vec<u8>>()
                        .escape_ascii()
                        .to_string();
                    Some(Ok(Token::Name(data)))
                }
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.machine = StateMachine::Start;
                    let data = self
                        .buffer
                        .drain(..)
                        .collect::<Vec<u8>>()
                        .escape_ascii()
                        .to_string();
                    Some(Ok(Token::Name(data)))
                }
                b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'0'..=b'9' => {
                    self.buffer.push(c);
                    self.machine = StateMachine::Name;
                    None
                }
                _ => Some(Err(Error {
                    kind: ErrorKind::Uninmplemented,
                    line: self.line,
                    column: self.column,
                })),
            },
            StateMachine::String => match c {
                b'\0' => Some(Err(Error {
                    kind: ErrorKind::EofAtString,
                    line: self.line,
                    column: self.column,
                })),
                b'"' => {
                    self.machine = StateMachine::Start;
                    let data = self
                        .buffer
                        .drain(..)
                        .collect::<Vec<u8>>()
                        .escape_ascii()
                        .to_string();
                    Some(Ok(Token::String(data)))
                }
                c => {
                    self.buffer.push(c);
                    self.machine = StateMachine::String;
                    None
                }
            },
            StateMachine::SeenMinus => match c {
                b'-' => {
                    self.machine = StateMachine::Comment;
                    None
                }
                _ => Some(Err(Error {
                    kind: ErrorKind::Uninmplemented,
                    line: self.line,
                    column: self.column,
                })),
            },
            StateMachine::Comment => match c {
                b'\0' => {
                    self.machine = StateMachine::End;
                    None
                }
                b'\t' | b'\r' | b' '..b'\x7f' => {
                    self.machine = StateMachine::ShortComment;
                    None
                }
                b'\n' => {
                    self.machine = StateMachine::Start;
                    None
                }
                _ => Some(Err(Error {
                    kind: ErrorKind::ProhibtedControlCharacterOnComment,
                    line: self.line,
                    column: self.column,
                })),
            },
            StateMachine::ShortComment => match c {
                b'\0' => {
                    self.machine = StateMachine::End;
                    None
                }
                b'\t' | b'\r' | b' '..b'\x7f' => {
                    self.machine = StateMachine::ShortComment;
                    None
                }
                b'\n' => {
                    self.machine = StateMachine::Start;
                    None
                }
                _ => Some(Err(Error {
                    kind: ErrorKind::ProhibtedControlCharacterOnComment,
                    line: self.line,
                    column: self.column,
                })),
            },
            StateMachine::End => match c {
                b'\0' => None,
                _ => Some(Err(Error {
                    kind: ErrorKind::CharacterAfterEof,
                    line: self.line,
                    column: self.column,
                })),
            },
        };

        if c == b'\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        processed
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateMachine {
    Start,
    Name,
    String,
    SeenMinus,
    Comment,
    ShortComment,
    End,
}
