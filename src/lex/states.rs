use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Start,
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    Mod,
    Pow,
    Len,
    BitAnd,
    BitOr,
    BitXor,
    ShiftRight,
    ShiftLeft,
    Concat,
    Assign,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Comma,
    Colon,
    DoubleColon,
    SemiColon,
    Dot,
    Dots,
    Number,
    Float,
    String(char),
    StringEscape(char),
    StringAscii(char, u8, u16),
    StringUtf8(char, u8),
    Name,
    ShortComment,
    Eof,
}

impl State {
    pub fn consume(&mut self, c: char) -> Result<Option<State>, StateError> {
        match self {
            Self::Start => Ok(Self::start_consume(c)),
            Self::Add => Ok(Self::add_consume(c)),
            Self::Sub => Ok(self.sub_consume(c)),
            Self::Mul => Ok(Self::mul_consume(c)),
            Self::Div => Ok(self.div_consume(c)),
            Self::Idiv => Ok(Self::idiv_consume(c)),
            Self::Mod => Ok(Self::mod_consume(c)),
            Self::Pow => Ok(Self::pow_consume(c)),
            Self::Len => Ok(Self::len_consume(c)),
            Self::BitAnd => Ok(Self::bit_and_consume(c)),
            Self::BitOr => Ok(Self::bit_or_consume(c)),
            Self::BitXor => Ok(self.bit_xor_consume(c)),
            Self::ShiftRight => Ok(Self::shift_right_consume(c)),
            Self::ShiftLeft => Ok(Self::shift_left_consume(c)),
            Self::Concat => Ok(self.concat_consume(c)),
            Self::Assign => Ok(self.assign_consume(c)),
            Self::Less => Ok(self.less_consume(c)),
            Self::LessEqual => Ok(Self::less_equal_consume(c)),
            Self::Greater => Ok(self.greater_consume(c)),
            Self::GreaterEqual => Ok(Self::greater_equal_consume(c)),
            Self::Equal => Ok(Self::equal_consume(c)),
            Self::NotEqual => Ok(Self::not_equal_consume(c)),
            Self::LParen => Ok(Self::lparen_consume(c)),
            Self::RParen => Ok(Self::rparen_consume(c)),
            Self::LSquare => Ok(Self::lsquare_consume(c)),
            Self::RSquare => Ok(Self::rsquare_consume(c)),
            Self::LCurly => Ok(Self::lcurly_consume(c)),
            Self::RCurly => Ok(Self::rcurly_consume(c)),
            Self::Comma => Ok(Self::comma_consume(c)),
            Self::Colon => Ok(self.colon_consume(c)),
            Self::DoubleColon => Ok(Self::double_colon_consume(c)),
            Self::SemiColon => Ok(Self::semi_colon_consume(c)),
            Self::Dot => Ok(self.dot_consume(c)),
            Self::Dots => Ok(Self::dots_consume(c)),
            Self::Number => Ok(self.number_consume(c)),
            Self::Float => Ok(Self::float_consume(c)),
            Self::String(start_quotes) => {
                let start_quotes = *start_quotes;
                Ok(self.string_consume(c, start_quotes))
            }
            Self::StringEscape(start_quotes) => {
                let start_quotes = *start_quotes;
                self.string_escape_consume(c, start_quotes)
            }
            Self::StringAscii(start_quotes, count, sum) => {
                let start_quotes = *start_quotes;
                let count = *count;
                let sum = *sum;
                self.string_ascii_consume(c, start_quotes, count, sum)
            }
            Self::StringUtf8(start_quotes, count) => {
                let start_quotes = *start_quotes;
                let count = *count;
                self.string_utf8_consume(c, start_quotes, count)
            }
            Self::Name => Ok(Self::name_consume(c)),
            Self::ShortComment => Ok(Self::short_comment_consume(c)),
            Self::Eof => Ok(None),
        }
        .map(|new_state_opt| new_state_opt.map(|new_state| self.replace_state(new_state)))
    }

    pub fn consume_eof(&mut self) -> Result<Option<Self>, StateError> {
        match self {
            Self::String(_) => Err(StateError::EofAtString),
            Self::Eof => Ok(None),
            _ => Ok(Some(self.replace_state(Self::Eof))),
        }
    }

    fn replace_state(&mut self, new_state: Self) -> Self {
        let old_state = *self;
        *self = new_state;
        old_state
    }

    fn start_consume(c: char) -> Option<Self> {
        match c {
            ' ' | '\t' | '\r' | '\n' => Some(Self::Start),
            '+' => Some(Self::Add),
            '-' => Some(Self::Sub),
            '*' => Some(Self::Mul),
            '/' => Some(Self::Div),
            '%' => Some(Self::Mod),
            '^' => Some(Self::Pow),
            '#' => Some(Self::Len),
            '&' => Some(Self::BitAnd),
            '|' => Some(Self::BitOr),
            '~' => Some(Self::BitXor),
            '<' => Some(Self::Less),
            '>' => Some(Self::Greater),
            '=' => Some(Self::Assign),
            '(' => Some(Self::LParen),
            ')' => Some(Self::RParen),
            '[' => Some(Self::LSquare),
            ']' => Some(Self::RSquare),
            '{' => Some(Self::LCurly),
            '}' => Some(Self::RCurly),
            ',' => Some(Self::Comma),
            ':' => Some(Self::Colon),
            ';' => Some(Self::SemiColon),
            '.' => Some(Self::Dot),
            '0'..='9' => Some(Self::Number),
            'a'..='z' | 'A'..='Z' | '_' => Some(Self::Name),
            string_start @ ('"' | '\'') => Some(Self::String(string_start)),
            other => unimplemented!("{:?}", other),
        }
    }

    fn add_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn sub_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '-' => {
                self.replace_state(Self::ShortComment);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn mul_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn div_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '/' => {
                self.replace_state(Self::Idiv);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn idiv_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn mod_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn pow_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn len_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn bit_and_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn bit_or_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn bit_xor_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '=' => {
                self.replace_state(Self::NotEqual);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn shift_right_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn shift_left_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn concat_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '.' => {
                self.replace_state(Self::Dots);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn assign_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '=' => {
                self.replace_state(Self::Equal);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn less_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '<' => {
                self.replace_state(Self::ShiftLeft);
                None
            }
            '=' => {
                self.replace_state(Self::LessEqual);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn less_equal_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn greater_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '>' => {
                self.replace_state(Self::ShiftRight);
                None
            }
            '=' => {
                self.replace_state(Self::GreaterEqual);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn greater_equal_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn equal_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn not_equal_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn lparen_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn rparen_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn lsquare_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn rsquare_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn lcurly_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn rcurly_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn comma_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn colon_consume(&mut self, c: char) -> Option<Self> {
        match c {
            ':' => {
                self.replace_state(Self::DoubleColon);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn double_colon_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn semi_colon_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn dot_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '.' => {
                self.replace_state(Self::Concat);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn dots_consume(c: char) -> Option<Self> {
        Self::start_consume(c)
    }

    fn number_consume(&mut self, c: char) -> Option<Self> {
        match c {
            '0'..='9' => None,
            '.' => {
                self.replace_state(Self::Float);
                None
            }
            _ => Self::start_consume(c),
        }
    }

    fn float_consume(c: char) -> Option<Self> {
        match c {
            '0'..='9' => None,
            _ => Self::start_consume(c),
        }
    }

    fn string_consume(&mut self, c: char, start_quotes: char) -> Option<Self> {
        log::trace!("{:?} {:?} {:?}", self, c, start_quotes);
        match c {
            quotes @ ('"' | '\'') => {
                if quotes == start_quotes {
                    Some(Self::Start)
                } else {
                    None
                }
            }
            '\\' => {
                self.replace_state(Self::StringEscape(start_quotes));
                None
            }
            _ => None,
        }
    }

    fn string_escape_consume(
        &mut self,
        c: char,
        start_quotes: char,
    ) -> Result<Option<Self>, StateError> {
        match c {
            'x' => {
                self.replace_state(Self::StringUtf8(start_quotes, 0));
                Ok(None)
            }
            '0'..='9' => {
                self.replace_state(Self::StringAscii(start_quotes, 0, 0));
                Ok(None)
            }
            'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '\\' | '"' | '\'' => {
                self.replace_state(Self::String(start_quotes));
                Ok(None)
            }
            _ => Err(StateError::EscapedChar(c)),
        }
    }

    fn string_ascii_consume(
        &mut self,
        c: char,
        start_quotes: char,
        count: u8,
        sum: u16,
    ) -> Result<Option<Self>, StateError> {
        if count == 3 {
            Ok(self.string_consume(c, start_quotes))
        } else {
            match c {
                quotes @ ('"' | '\'') => {
                    if quotes == start_quotes {
                        Ok(Some(Self::Start))
                    } else {
                        Ok(None)
                    }
                }
                d @ '0'..='9' => {
                    let digit = d as u8 - b'0';
                    let sum = sum * 10 + u16::from(digit);
                    if sum <= 255 {
                        self.replace_state(Self::StringAscii(start_quotes, count + 1, sum));
                        Ok(None)
                    } else {
                        Err(StateError::AsciiOutOfBounds(sum))
                    }
                }
                _ => Ok(self.string_consume(c, start_quotes)),
            }
        }
    }

    fn string_utf8_consume(
        &mut self,
        c: char,
        start_quotes: char,
        count: u8,
    ) -> Result<Option<Self>, StateError> {
        if count == 2 {
            Ok(self.string_consume(c, start_quotes))
        } else {
            match c {
                'a'..='f' | 'A'..='F' | '0'..='9' => {
                    self.replace_state(Self::StringUtf8(start_quotes, count + 1));
                    Ok(None)
                }
                _ => Err(StateError::HexCharacter(c)),
            }
        }
    }

    fn name_consume(c: char) -> Option<Self> {
        match c {
            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => None,
            _ => Self::start_consume(c),
        }
    }

    fn short_comment_consume(c: char) -> Option<Self> {
        match c {
            '\n' => Some(Self::Start),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum StateError {
    EofAtString,
    EscapedChar(char),
    HexCharacter(char),
    AsciiOutOfBounds(u16),
}

impl Display for StateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EofAtString => write!(f, "Reached end of file while parsing a string."),
            Self::EscapedChar(c) => write!(f, "Invalid escaped character `{}`.", c),
            Self::HexCharacter(c) => write!(f, "Invalid hex character `{}`.", c),
            Self::AsciiOutOfBounds(sum) => {
                write!(f, "Escaped number `{}` is not an Ascii character.", sum)
            }
        }
    }
}

impl core::error::Error for StateError {}
