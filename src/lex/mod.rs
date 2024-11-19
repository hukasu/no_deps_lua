mod error;
mod lexeme;
#[cfg(test)]
mod tests;

use core::{iter::Peekable, str::Chars};

pub use self::{
    error::{Error, ErrorKind},
    lexeme::{Lexeme, LexemeType},
};

pub struct Lex<'a> {
    program: &'a str,
    chars: Peekable<Chars<'a>>,
    /// Current `char` being read
    seek: usize,
    /// Start of lexeme being considered
    start: usize,
    line: usize,
    column: usize,
}

impl<'a> Lex<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            program: data,
            chars: data.chars().peekable(),
            seek: 0,
            start: 0,
            line: 0,
            column: 0,
        }
    }

    #[cfg(test)]
    pub fn remaining(&self) -> usize {
        self.program.len() - self.seek
    }
}

impl<'a> Iterator for Lex<'a> {
    type Item = Result<Lexeme<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item>
    where
        Self: 'a,
    {
        'lexer: loop {
            if self.start == usize::MAX {
                break None;
            }

            let Some(char) = self.chars.next() else {
                self.start = usize::MAX;

                break Some(Ok(Lexeme {
                    line: self.line,
                    column: self.column,
                    start: self.seek,
                    lexeme_type: LexemeType::Eof,
                }));
            };

            self.seek += char.len_utf8();
            self.start = self.seek - 1;
            self.column += 1;

            match char {
                '+' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::Add,
                    }));
                }
                '-' => match self.chars.peek() {
                    Some('-') => {
                        while let Some(c) = self.chars.peek().copied() {
                            if c == '\n' {
                                break;
                            }
                            self.chars.next();
                            self.seek += c.len_utf8();
                            self.start += c.len_utf8();
                            self.column += 1;
                        }
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::Sub,
                        }));
                    }
                },
                '*' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::Mul,
                    }));
                }
                '/' => match self.chars.peek() {
                    Some('/') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::Idiv,
                        }));
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::Div,
                        }));
                    }
                },
                '%' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::Mod,
                    }));
                }
                '^' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::Pow,
                    }));
                }
                '#' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::Len,
                    }));
                }
                '&' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::BitAnd,
                    }));
                }
                '|' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::BitOr,
                    }));
                }
                '~' => match self.chars.peek() {
                    Some('=') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::Neq,
                        }));
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::BitXor,
                        }));
                    }
                },
                '<' => match self.chars.peek() {
                    Some('<') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::ShiftL,
                        }));
                    }
                    Some('=') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::Leq,
                        }));
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::Less,
                        }));
                    }
                },
                '>' => match self.chars.peek() {
                    Some('>') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::ShiftR,
                        }));
                    }
                    Some('=') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::Geq,
                        }));
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::Greater,
                        }));
                    }
                },
                '=' => match self.chars.peek() {
                    Some('=') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::Eq,
                        }));
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::Assign,
                        }));
                    }
                },
                '(' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::LParen,
                    }));
                }
                ')' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::RParen,
                    }));
                }
                '[' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::LSquare,
                    }));
                }
                ']' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::RSquare,
                    }));
                }
                '{' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::LCurly,
                    }));
                }
                '}' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::RCurly,
                    }));
                }
                ';' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::SemiColon,
                    }));
                }
                ':' => match self.chars.peek() {
                    Some(':') => {
                        self.chars.next();
                        self.seek += 1;
                        self.start += 1;
                        self.column += 1;
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column - 1,
                            start: self.start,
                            lexeme_type: LexemeType::DoubleColon,
                        }));
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::Colon,
                        }));
                    }
                },
                ',' => {
                    break Some(Ok(Lexeme {
                        line: self.line,
                        column: self.column,
                        start: self.start,
                        lexeme_type: LexemeType::Comma,
                    }));
                }
                '.' => match self.chars.peek() {
                    Some('.') => {
                        self.chars.next();
                        match self.chars.peek() {
                            Some('.') => {
                                self.chars.next();
                                self.seek += 1;
                                self.start += 1;
                                self.column += 1;
                                break Some(Ok(Lexeme {
                                    line: self.line,
                                    column: self.column - 2,
                                    start: self.start - 1,
                                    lexeme_type: LexemeType::Dots,
                                }));
                            }
                            _ => {
                                break Some(Ok(Lexeme {
                                    line: self.line,
                                    column: self.column - 1,
                                    start: self.start,
                                    lexeme_type: LexemeType::Concat,
                                }));
                            }
                        }
                    }
                    _ => {
                        break Some(Ok(Lexeme {
                            line: self.line,
                            column: self.column,
                            start: self.start,
                            lexeme_type: LexemeType::Dot,
                        }));
                    }
                },
                short_string_start @ '"' | short_string_start @ '\'' => {
                    while let Some(c) = self.chars.peek().copied() {
                        match c {
                            c if c == short_string_start => {
                                let start = self.start;
                                let end = self.seek;

                                self.chars.next();
                                self.seek += 1;
                                self.start = self.seek - 1;
                                self.column += 1;
                                let output_str = &self.program[(start + 1)..end];
                                break 'lexer Some(Ok(Lexeme {
                                    line: self.line,
                                    column: self.column,
                                    start,
                                    // Skipping the start " or '
                                    lexeme_type: LexemeType::String(output_str),
                                }));
                            }
                            '\\' => match self.chars.peek().copied() {
                                Some('\n') => {
                                    self.chars.next();
                                    self.seek += 1;
                                    self.line += 1;
                                    self.column = 0;
                                }
                                Some('a') | Some('b') | Some('f') | Some('n') | Some('r')
                                | Some('t') | Some('v') | Some('\\') | Some('"') | Some('\'') => {
                                    self.chars.next();
                                    self.seek += 1;
                                    self.column += 1;
                                }
                                _ => {
                                    break 'lexer Some(Err(Error {
                                        kind: ErrorKind::ProhibtedControlCharacterOnString,
                                        line: self.line,
                                        column: self.column,
                                    }));
                                }
                            },
                            c => {
                                self.seek += c.len_utf8();
                                self.column += 1;
                                self.chars.next();
                            }
                        }
                    }
                    break Some(Err(Error {
                        kind: ErrorKind::EofAtString,
                        line: self.line,
                        column: self.column,
                    }));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    while let Some(c) = self.chars.peek().copied() {
                        match c {
                            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                                self.chars.next();
                                self.seek += 1;
                                self.column += 1;
                            }
                            _ => {
                                let start = self.start;
                                let name = &self.program[start..self.seek];
                                match name {
                                    "and" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::And,
                                        }));
                                    }
                                    "break" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Break,
                                        }));
                                    }
                                    "do" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Do,
                                        }));
                                    }
                                    "else" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Else,
                                        }));
                                    }
                                    "elseif" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Elseif,
                                        }));
                                    }

                                    "end" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::End,
                                        }));
                                    }
                                    "false" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::False,
                                        }));
                                    }
                                    "for" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::For,
                                        }));
                                    }
                                    "function" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Function,
                                        }));
                                    }
                                    "goto" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Goto,
                                        }));
                                    }
                                    "if" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::If,
                                        }));
                                    }
                                    "in" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::In,
                                        }));
                                    }
                                    "local" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Local,
                                        }));
                                    }
                                    "nil" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Nil,
                                        }));
                                    }
                                    "not" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Not,
                                        }));
                                    }
                                    "or" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Or,
                                        }));
                                    }
                                    "repeat" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Repeat,
                                        }));
                                    }
                                    "return" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Return,
                                        }));
                                    }
                                    "then" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Then,
                                        }));
                                    }
                                    "true" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::True,
                                        }));
                                    }
                                    "until" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Until,
                                        }));
                                    }
                                    "while" => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::While,
                                        }));
                                    }
                                    other => {
                                        break 'lexer Some(Ok(Lexeme {
                                            line: self.line,
                                            column: self.column,
                                            start,
                                            lexeme_type: LexemeType::Name(other),
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
                '0' => match self.chars.peek() {
                    Some('x' | 'X') => {
                        break Some(Err(Error {
                            kind: ErrorKind::Uninmplemented,
                            line: self.line,
                            column: self.column,
                        }));
                    }
                    Some('0'..='7') => {
                        break Some(Err(Error {
                            kind: ErrorKind::OctalNotSupported,
                            line: self.line,
                            column: self.column,
                        }));
                    }
                    Some('8' | '9') => {
                        break Some(Err(Error {
                            kind: ErrorKind::LeadingZero,
                            line: self.line,
                            column: self.column,
                        }));
                    }
                    _ => {
                        break Some(
                            self.program[self.start..self.seek]
                                .parse()
                                .map(|i| Lexeme {
                                    line: self.line,
                                    column: self.column,
                                    start: self.start,
                                    lexeme_type: LexemeType::Integer(i),
                                })
                                .map_err(|_err| Error {
                                    kind: ErrorKind::ParseInt,
                                    line: self.line,
                                    column: self.column,
                                }),
                        )
                    }
                },
                '1'..='9' => {
                    let mut is_float = false;
                    while let Some(c) = self.chars.peek().copied() {
                        match c {
                            '0'..='9' => {
                                self.chars.next();
                                self.seek += 1;
                                self.column += 1;
                            }
                            '.' => {
                                if is_float {
                                    break 'lexer Some(Err(Error {
                                        kind: ErrorKind::MalformedFloat,
                                        line: self.line,
                                        column: self.column,
                                    }));
                                } else {
                                    self.chars.next();
                                    self.seek += 1;
                                    self.column += 1;
                                    is_float = true;
                                }
                            }
                            'e' | 'E' => {
                                break 'lexer Some(Err(Error {
                                    kind: ErrorKind::Uninmplemented,
                                    line: self.line,
                                    column: self.column,
                                }));
                            }
                            _ => {
                                if is_float {
                                    break 'lexer Some(
                                        self.program[self.start..self.seek]
                                            .parse()
                                            .map(|i| Lexeme {
                                                line: self.line,
                                                column: self.column,
                                                start: self.start,
                                                lexeme_type: LexemeType::Float(i),
                                            })
                                            .map_err(|_err| Error {
                                                kind: ErrorKind::ParseFloat,
                                                line: self.line,
                                                column: self.column,
                                            }),
                                    );
                                } else {
                                    break 'lexer Some(
                                        self.program[self.start..self.seek]
                                            .parse()
                                            .map(|i| Lexeme {
                                                line: self.line,
                                                column: self.column,
                                                start: self.start,
                                                lexeme_type: LexemeType::Integer(i),
                                            })
                                            .map_err(|_err| Error {
                                                kind: ErrorKind::ParseInt,
                                                line: self.line,
                                                column: self.column,
                                            }),
                                    );
                                }
                            }
                        }
                    }
                }
                ' ' | '\t' | '\r' => {}
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                }
                _ => {
                    break Some(Err(Error {
                        kind: ErrorKind::Uninmplemented,
                        line: self.line,
                        column: self.column,
                    }));
                }
            }
        }
    }
}
