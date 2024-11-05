use super::{Error, ErrorKind, Token, TokenType};

#[derive(Debug)]
pub struct State {
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) seek: usize,
    pub(super) machine: StateMachine,
    pub(super) buffer_len: usize,
}

impl State {
    pub fn process<'a>(
        &mut self,
        program: &'a str,
        c: char,
    ) -> Option<Result<(Token<'a>, Option<Token<'a>>), Error>> {
        let processed = match (&self.machine, c) {
            // Processing end of file
            (StateMachine::Start | StateMachine::Comment | StateMachine::ShortComment, '\0') => {
                self.machine = StateMachine::End;
                None
            }
            (StateMachine::Name, '\0') => {
                self.machine = StateMachine::End;
                self.finish_name(program)
                    .map(|token_res| token_res.map(|token| (token, None)))
            }
            (StateMachine::String, '\0') => Some(Err(Error {
                kind: ErrorKind::EofAtString,
                line: self.line,
                column: self.column,
            })),
            (StateMachine::End, '\0') => None,

            // Processing empty characters
            (StateMachine::Start, ' ' | '\r' | '\n' | '\t') => None,
            (StateMachine::Name, ' ' | '\r' | '\n' | '\t') => {
                self.machine = StateMachine::Start;
                self.finish_name(program)
                    .map(|token_res| token_res.map(|token| (token, None)))
            }
            (StateMachine::SeenMinus, ' ' | '\r' | '\n' | '\t') => {
                self.machine = StateMachine::Start;
                Some(Ok((self.finish_token(TokenType::Sub, 1), None)))
            }

            // Processing `+`
            (StateMachine::Start, '+') => Some(Ok((self.finish_token(TokenType::Add, 0), None))),
            (StateMachine::Name, '+') => {
                self.machine = StateMachine::Start;
                self.finish_name(program).map(|token_res| {
                    token_res.map(|token| (token, Some(self.finish_token(TokenType::Sub, 0))))
                })
            }
            (StateMachine::SeenMinus, '+') => Some(Ok((
                self.finish_token(TokenType::Sub, 1),
                Some(self.finish_token(TokenType::Add, 0)),
            ))),
            (StateMachine::SeenSlash, '+') => Some(Ok((
                self.finish_token(TokenType::Div, 1),
                Some(self.finish_token(TokenType::Add, 0)),
            ))),
            (StateMachine::SeenTilde, '+') => Some(Ok((
                self.finish_token(TokenType::BitXor, 0),
                Some(self.finish_token(TokenType::Add, 0)),
            ))),

            // Processing `-`
            (StateMachine::Start, '-') => {
                self.machine = StateMachine::SeenMinus;
                None
            }
            (StateMachine::Name, '-') => {
                self.machine = StateMachine::SeenMinus;
                self.finish_name(program)
                    .map(|token_res| token_res.map(|token| (token, None)))
            }
            (StateMachine::SeenMinus, '-') => {
                self.machine = StateMachine::Comment;
                None
            }
            (StateMachine::SeenSlash, '-') => {
                self.machine = StateMachine::SeenMinus;
                Some(Ok((self.finish_token(TokenType::Div, 1), None)))
            }
            (StateMachine::SeenTilde, '-') => {
                self.machine = StateMachine::SeenMinus;
                Some(Ok((self.finish_token(TokenType::BitXor, 1), None)))
            }

            // Processing `*`
            (StateMachine::Start, '*') => Some(Ok((self.finish_token(TokenType::Mul, 0), None))),
            (StateMachine::Name, '*') => {
                self.machine = StateMachine::Start;
                self.finish_name(program).map(|token_res| {
                    token_res.map(|token| (token, Some(self.finish_token(TokenType::Mul, 0))))
                })
            }
            (StateMachine::SeenMinus, '*') => {
                self.machine = StateMachine::Start;
                Some(Ok((
                    self.finish_token(TokenType::Sub, 1),
                    Some(self.finish_token(TokenType::Mul, 0)),
                )))
            }
            (StateMachine::SeenSlash, '*') => {
                self.machine = StateMachine::Start;
                Some(Ok((
                    self.finish_token(TokenType::Div, 1),
                    Some(self.finish_token(TokenType::Mul, 0)),
                )))
            }
            (StateMachine::SeenTilde, '*') => {
                self.machine = StateMachine::Start;
                Some(Ok((
                    self.finish_token(TokenType::BitXor, 1),
                    Some(self.finish_token(TokenType::Mul, 0)),
                )))
            }

            // Processing `/`
            (StateMachine::Start, '/') => {
                self.machine = StateMachine::SeenSlash;
                None
            }
            (StateMachine::Name, '/') => {
                self.machine = StateMachine::SeenSlash;
                self.finish_name(program)
                    .map(|token_res| token_res.map(|token| (token, None)))
            }
            (StateMachine::SeenMinus, '/') => {
                self.machine = StateMachine::SeenSlash;
                Some(Ok((self.finish_token(TokenType::Sub, 0), None)))
            }
            (StateMachine::SeenSlash, '/') => {
                self.machine = StateMachine::Start;
                Some(Ok((self.finish_token(TokenType::Idiv, 1), None)))
            }
            (StateMachine::SeenTilde, '/') => {
                self.machine = StateMachine::SeenSlash;
                Some(Ok((self.finish_token(TokenType::BitXor, 1), None)))
            }

            (StateMachine::Start, '%') => Some(Ok((self.finish_token(TokenType::Mod, 0), None))),
            (StateMachine::Start, '^') => Some(Ok((self.finish_token(TokenType::Pow, 0), None))),
            (StateMachine::Start, '#') => Some(Ok((self.finish_token(TokenType::Len, 0), None))),
            (StateMachine::Start, '&') => Some(Ok((self.finish_token(TokenType::BitAnd, 0), None))),
            (StateMachine::Start, '~') => {
                self.machine = StateMachine::SeenTilde;
                None
            }
            (StateMachine::Start, '|') => Some(Ok((self.finish_token(TokenType::BitOr, 0), None))),
            (StateMachine::Start, '<') => {
                self.machine = StateMachine::SeenLess;
                None
            }
            (StateMachine::Start, '>') => {
                self.machine = StateMachine::SeenGreater;
                None
            }
            (StateMachine::Start, '=') => {
                self.machine = StateMachine::SeenEquals;
                None
            }
            (StateMachine::Start, '(') => Some(Ok((self.finish_token(TokenType::LParen, 0), None))),
            (StateMachine::Start, ')') => Some(Ok((self.finish_token(TokenType::RParen, 0), None))),
            (StateMachine::Start, '{') => Some(Ok((self.finish_token(TokenType::LCurly, 0), None))),
            (StateMachine::Start, '}') => Some(Ok((self.finish_token(TokenType::RCurly, 0), None))),
            (StateMachine::Start, '[') => {
                Some(Ok((self.finish_token(TokenType::LSquare, 0), None)))
            }
            (StateMachine::Start, ']') => {
                Some(Ok((self.finish_token(TokenType::RSquare, 0), None)))
            }
            (StateMachine::Start, ':') => {
                self.machine = StateMachine::SeenColon;
                None
            }
            (StateMachine::Start, ';') => {
                Some(Ok((self.finish_token(TokenType::SemiColon, 0), None)))
            }
            (StateMachine::Start, ',') => Some(Ok((self.finish_token(TokenType::Comma, 0), None))),
            (StateMachine::Start, '.') => {
                self.machine = StateMachine::SeenDot;
                None
            }
            (StateMachine::Start | StateMachine::SeenMinus, 'A'..='Z' | 'a'..='z' | '_') => {
                self.buffer_len += 1;
                self.machine = StateMachine::Name;
                None
            }
            (StateMachine::Start, '"') => {
                self.machine = StateMachine::String;
                None
            }

            (StateMachine::Name, '(') => {
                self.machine = StateMachine::Start;
                self.finish_name(program).map(|token_res| {
                    token_res.map(|token| (token, Some(self.finish_token(TokenType::LParen, 0))))
                })
            }
            (StateMachine::Name, 'A'..='Z' | 'a'..='z' | '_' | '0'..='9') => {
                self.buffer_len += 1;
                self.machine = StateMachine::Name;
                None
            }

            // Processing String
            (StateMachine::String, '"') => {
                self.machine = StateMachine::Start;
                let buffer_len = self.buffer_len;
                let buffer = self.buffer(program);
                self.buffer_len = 0;
                Some(Ok((
                    self.finish_token(TokenType::String(buffer), buffer_len + 2),
                    None,
                )))
            }
            (StateMachine::String, _) => {
                self.buffer_len += 1;
                self.machine = StateMachine::String;
                None
            }

            // Comments
            (StateMachine::Comment | StateMachine::ShortComment, '\t' | '\r' | ' '..'\x7f') => {
                self.machine = StateMachine::ShortComment;
                None
            }
            (StateMachine::Comment | StateMachine::ShortComment, '\n') => {
                self.machine = StateMachine::Start;
                None
            }

            // Processing Unimplementeded tokens
            (
                StateMachine::Start
                | StateMachine::Name
                | StateMachine::SeenMinus
                | StateMachine::SeenSlash
                | StateMachine::SeenTilde
                | StateMachine::SeenLess
                | StateMachine::SeenGreater
                | StateMachine::SeenEquals
                | StateMachine::SeenColon
                | StateMachine::SeenDot,
                _,
            ) => Some(Err(Error {
                kind: ErrorKind::Uninmplemented,
                line: self.line,
                column: self.column,
            })),
            (StateMachine::Comment | StateMachine::ShortComment, _) => Some(Err(Error {
                kind: ErrorKind::ProhibtedControlCharacterOnComment,
                line: self.line,
                column: self.column,
            })),

            // State Machine is on end state
            (StateMachine::End, _) => Some(Err(Error {
                kind: ErrorKind::CharacterAfterEof,
                line: self.line,
                column: self.column,
            })),
        };

        if c == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
        if c != '\0' {
            self.seek += 1;
        }

        processed
    }

    fn finish_name<'a>(&mut self, program: &'a str) -> Option<Result<Token<'a>, Error>> {
        let buffer = self.buffer(program);
        let buffer_len = self.buffer_len;
        self.buffer_len = 0;
        match buffer {
            "and" => Some(Ok(self.finish_token(TokenType::And, 3))),
            "break" => Some(Ok(self.finish_token(TokenType::Break, 5))),
            "do" => Some(Ok(self.finish_token(TokenType::Do, 2))),
            "else" => Some(Ok(self.finish_token(TokenType::Else, 4))),
            "elseif" => Some(Ok(self.finish_token(TokenType::Elseif, 6))),
            "end" => Some(Ok(self.finish_token(TokenType::End, 3))),
            "false" => Some(Ok(self.finish_token(TokenType::False, 5))),
            "for" => Some(Ok(self.finish_token(TokenType::For, 3))),
            "function" => Some(Ok(self.finish_token(TokenType::Function, 8))),
            "goto" => Some(Ok(self.finish_token(TokenType::Goto, 4))),
            "if" => Some(Ok(self.finish_token(TokenType::If, 2))),
            "in" => Some(Ok(self.finish_token(TokenType::In, 2))),
            "local" => Some(Ok(self.finish_token(TokenType::Local, 5))),
            "nil" => Some(Ok(self.finish_token(TokenType::Nil, 3))),
            "not" => Some(Ok(self.finish_token(TokenType::Not, 3))),
            "or" => Some(Ok(self.finish_token(TokenType::Or, 2))),
            "repeat" => Some(Ok(self.finish_token(TokenType::Repeat, 6))),
            "return" => Some(Ok(self.finish_token(TokenType::Return, 6))),
            "then" => Some(Ok(self.finish_token(TokenType::Then, 4))),
            "true" => Some(Ok(self.finish_token(TokenType::True, 4))),
            "until" => Some(Ok(self.finish_token(TokenType::Until, 5))),
            "while" => Some(Ok(self.finish_token(TokenType::While, 5))),
            data => Some(Ok(self.finish_token(TokenType::Name(data), buffer_len))),
        }
    }

    fn finish_token<'a>(&self, token_type: TokenType<'a>, start_offset: usize) -> Token<'a> {
        Token {
            line: self.line,
            column: self.column,
            start_offset,
            token: token_type,
        }
    }

    fn buffer<'a>(&self, program: &'a str) -> &'a str {
        &program[(self.seek - self.buffer_len)..self.seek]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateMachine {
    Start,
    Name,
    String,
    SeenMinus,
    SeenSlash,
    SeenTilde,
    Comment,
    ShortComment,
    End,
    SeenLess,
    SeenGreater,
    SeenEquals,
    SeenColon,
    SeenDot,
}
