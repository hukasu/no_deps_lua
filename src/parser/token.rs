use core::borrow::Borrow;

use alloc::vec::Vec;

use crate::lex::{Lexeme, LexemeType};

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub(crate) tokens: Vec<Token<'a>>,
    pub(crate) token_type: TokenType<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType<'a> {
    // Terminals
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Len,
    BitAnd,
    BitOr,
    BitXor,
    ShiftL,
    ShiftR,
    Idiv,
    Eq,
    Neq,
    Leq,
    Geq,
    Less,
    Greater,
    Assign,
    LParen,
    RParen,
    LCurly,
    RCurly,
    LSquare,
    RSquare,
    SemiColon,
    Colon,
    DoubleColon,
    Comma,
    Dot,
    Concat,
    Dots,
    Integer(i64),
    Float(f64),
    String(&'a str),
    Name(&'a str),
    Eof,

    // Non-terminals
    /// Chunk
    ///
    /// Reduced from Productions
    /// ```custom
    /// 0 chuck :== block Eof
    /// ```
    Chunk,
    /// Block
    ///
    /// Reduced from Productions
    /// ```custom
    /// 1 block :== block_stat block_retstat
    /// ```
    Block,
    /// BlockStat
    ///
    /// Reduced from Productions
    /// ```custom
    /// 2 block_stat :==
    /// 3 block_stat :== stat block_stat
    /// ```
    BlockStat,
    /// BlockRetstat
    ///
    /// Reduced from Productions
    /// ```custom
    /// 4 block_retstat :==
    /// 5 block_retstat :== retstat
    /// ```
    BlockRetstat,
    /// Stat
    ///
    /// Reduced from Productions
    /// ```custom
    /// 6   stat :== SemiColon
    /// 7   stat :== varlist Assign explist
    /// 8   stat :== functioncall
    /// 9   stat :== Label
    /// 10  stat :== Break
    /// 11  stat :== Goto Name
    /// 12  stat :== Do block End
    /// 13  stat :== While exp Do block End
    /// 14  stat :== Repeat block Until exp
    /// 15  stat :== If exp Then block stat_elseif stat_else End
    /// 20  stat :== For Name Assign exp Comma exp stat_forexp Do block End
    /// 23  stat :== For namelist In explist Do block End
    /// 24  stat :== Function funcname funcbody
    /// 25  stat :== Local Function Name funcbody
    /// 26  stat :== Local attnamelist stat_attexplist
    /// ```
    Stat,
    /// Var
    ///
    /// Reduced from Productions
    /// ```custom
    /// 48 var :== Name
    /// 49 var :== prefixexp LSquare exp RSquare
    /// 50 var :== prefixexp Dot Name
    /// ```
    Var,
    /// Prefixexp
    ///
    /// Reduced from Productions
    /// ```custom
    /// 91 prefixexp :== var
    /// 92 prefixexp :== functioncall
    /// 93 prefixexp :== Lparen exp Rparen
    /// ```
    Prefixexp,
    /// Functioncall
    ///
    /// Reduced from Productions
    /// ```custom
    /// 94  functioncall :== prefixexp args
    /// 95  functioncall :== prefixexp Colon Name args
    /// ```
    Functioncall,
    /// Args
    ///
    /// Reduced from Productions
    /// ```custom
    /// 96  args :== Lparen ArgsExplist Rparen
    /// 99  args :== tableconstructor
    /// 100 args :== String
    /// ```
    Args,
}

impl<'a, T: Borrow<Lexeme<'a>>> From<T> for Token<'a> {
    fn from(value: T) -> Self {
        match value.borrow().lexeme_type {
            LexemeType::And => Token {
                tokens: [].to_vec(),
                token_type: TokenType::And,
            },
            LexemeType::Break => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Break,
            },
            LexemeType::Do => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Do,
            },
            LexemeType::Else => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Else,
            },
            LexemeType::Elseif => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Elseif,
            },
            LexemeType::End => Token {
                tokens: [].to_vec(),
                token_type: TokenType::End,
            },
            LexemeType::False => Token {
                tokens: [].to_vec(),
                token_type: TokenType::False,
            },
            LexemeType::For => Token {
                tokens: [].to_vec(),
                token_type: TokenType::For,
            },
            LexemeType::Function => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Function,
            },
            LexemeType::Goto => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Goto,
            },
            LexemeType::If => Token {
                tokens: [].to_vec(),
                token_type: TokenType::If,
            },
            LexemeType::In => Token {
                tokens: [].to_vec(),
                token_type: TokenType::In,
            },
            LexemeType::Local => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Local,
            },
            LexemeType::Nil => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Nil,
            },
            LexemeType::Not => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Not,
            },
            LexemeType::Or => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Or,
            },
            LexemeType::Repeat => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Repeat,
            },
            LexemeType::Return => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Return,
            },
            LexemeType::Then => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Then,
            },
            LexemeType::True => Token {
                tokens: [].to_vec(),
                token_type: TokenType::True,
            },
            LexemeType::Until => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Until,
            },
            LexemeType::While => Token {
                tokens: [].to_vec(),
                token_type: TokenType::While,
            },
            LexemeType::Add => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Add,
            },
            LexemeType::Sub => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Sub,
            },
            LexemeType::Mul => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Mul,
            },
            LexemeType::Div => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Div,
            },
            LexemeType::Mod => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Mod,
            },
            LexemeType::Pow => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Pow,
            },
            LexemeType::Len => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Len,
            },
            LexemeType::BitAnd => Token {
                tokens: [].to_vec(),
                token_type: TokenType::BitAnd,
            },
            LexemeType::BitOr => Token {
                tokens: [].to_vec(),
                token_type: TokenType::BitOr,
            },
            LexemeType::BitXor => Token {
                tokens: [].to_vec(),
                token_type: TokenType::BitXor,
            },
            LexemeType::ShiftL => Token {
                tokens: [].to_vec(),
                token_type: TokenType::ShiftL,
            },
            LexemeType::ShiftR => Token {
                tokens: [].to_vec(),
                token_type: TokenType::ShiftR,
            },
            LexemeType::Idiv => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Idiv,
            },
            LexemeType::Eq => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Eq,
            },
            LexemeType::Neq => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Neq,
            },
            LexemeType::Leq => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Leq,
            },
            LexemeType::Geq => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Geq,
            },
            LexemeType::Less => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Less,
            },
            LexemeType::Greater => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Greater,
            },
            LexemeType::Assign => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Assign,
            },
            LexemeType::LParen => Token {
                tokens: [].to_vec(),
                token_type: TokenType::LParen,
            },
            LexemeType::RParen => Token {
                tokens: [].to_vec(),
                token_type: TokenType::RParen,
            },
            LexemeType::LCurly => Token {
                tokens: [].to_vec(),
                token_type: TokenType::LCurly,
            },
            LexemeType::RCurly => Token {
                tokens: [].to_vec(),
                token_type: TokenType::RCurly,
            },
            LexemeType::LSquare => Token {
                tokens: [].to_vec(),
                token_type: TokenType::LSquare,
            },
            LexemeType::RSquare => Token {
                tokens: [].to_vec(),
                token_type: TokenType::RSquare,
            },
            LexemeType::SemiColon => Token {
                tokens: [].to_vec(),
                token_type: TokenType::SemiColon,
            },
            LexemeType::Colon => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Colon,
            },
            LexemeType::DoubleColon => Token {
                tokens: [].to_vec(),
                token_type: TokenType::DoubleColon,
            },
            LexemeType::Comma => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Comma,
            },
            LexemeType::Dot => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Dot,
            },
            LexemeType::Concat => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Concat,
            },
            LexemeType::Dots => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Dots,
            },
            LexemeType::Integer(i) => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Integer(i),
            },
            LexemeType::Float(f) => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Float(f),
            },
            LexemeType::String(s) => Token {
                tokens: [].to_vec(),
                token_type: TokenType::String(s),
            },
            LexemeType::Name(n) => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Name(n),
            },
            LexemeType::Eof => Token {
                tokens: [].to_vec(),
                token_type: TokenType::Eof,
            },
        }
    }
}
