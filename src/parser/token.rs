use core::{borrow::Borrow, cmp::Ordering, fmt::Display};

use alloc::vec::Vec;

use crate::lex::{Lexeme, LexemeType};

use super::{Error, Parser};

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub(crate) tokens: Vec<Token<'a>>,
    pub(crate) token_type: TokenType<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    StatIf,
    StatForexp,
    StatAttexplist,
    Attnamelist,
    AttnamelistCont,
    Attrib,
    /// Retstat
    ///
    /// Reduced from Productions
    /// ```custom
    /// 34 retstat :== Return retstat_explist retstat_end
    /// ```
    Retstat,
    /// RetstatExplist
    ///
    /// Reduced from Productions
    /// ```custom
    /// 35 retstat_explist :==
    /// 36 retstat_explist :== explist
    /// ```
    RetstatExplist,
    /// RetstatEnd
    ///
    /// Reduced from Productions
    /// ```custom
    /// 37 retstat_end :==
    /// 38 retstat_end :== SemiColon
    /// ```
    RetstatEnd,
    Label,
    Funcname,
    FuncnameCont,
    FuncnameEnd,
    /// Varlist
    ///
    /// Reduced from Productions
    /// ```custom
    /// 45 varlist :== var varlist_cont
    /// ```
    Varlist,
    /// VarlistCont
    ///
    /// Reduced from Productions
    /// ```custom
    /// 46 varlist_cont :==
    /// 47 varlist_cont :== Comma var varlist_cont
    /// ```
    VarlistCont,
    /// Var
    ///
    /// Reduced from Productions
    /// ```custom
    /// 48 var :== Name
    /// 49 var :== prefixexp LSquare exp RSquare
    /// 50 var :== prefixexp Dot Name
    /// ```
    Var,
    Namelist,
    NamelistCont,
    /// Explist
    ///
    /// Reduced from Productions
    /// ```custom
    /// 54  explist :== exp explist_cont
    /// ```
    Explist,
    /// ExplistCont
    ///
    /// Reduced from Productions
    /// ```custom
    /// 55  explist_cont :==
    /// 56  explist_cont :== Comma exp explist_cont
    /// ```
    ExplistCont,
    /// Exp
    ///
    /// Reduced from Productions
    /// ```custom
    /// 57 exp :== Nil
    /// 58 exp :== False
    /// 59 exp :== True
    /// 60 exp :== String
    /// 61 exp :== Numeral
    /// 62 exp :== Dots
    /// 63 exp :== functiondef
    /// 64 exp :== prefixexp
    /// 65 exp :== tableconstructor
    /// 66 exp :== exp Or exp
    /// 67 exp :== exp And exp
    /// 68 exp :== exp Less exp
    /// 69 exp :== exp Greater exp
    /// 70 exp :== exp Leq exp
    /// 71 exp :== exp Geq exp
    /// 72 exp :== exp Eq exp
    /// 73 exp :== exp Neq exp
    /// 74 exp :== exp BitOr exp
    /// 75 exp :== exp BitXor exp
    /// 76 exp :== exp BitAnd exp
    /// 77 exp :== exp ShiftL exp
    /// 78 exp :== exp ShiftR exp
    /// 79 exp :== exp Concat exp
    /// 80 exp :== exp Add exp
    /// 81 exp :== exp Sub exp
    /// 82 exp :== exp Mul exp
    /// 83 exp :== exp Div exp
    /// 84 exp :== exp Idiv exp
    /// 85 exp :== exp Mod exp
    /// 86 exp :== Not exp
    /// 87 exp :== Len exp
    /// 88 exp :== Sub exp
    /// 89 exp :== BitXor exp
    /// 90 exp :== exp Pow exp
    /// ```
    Exp,
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
    ArgsExplist,
    Functiondef,
    Funcbody,
    FuncbodyParlist,
    Parlist,
    ParlistCont,
    Tableconstructor,
    TableconstructorFieldlist,
    Fieldlist,
    FieldlistCont,
    Field,
    Fieldsep,
    Binop,
    Unop,
}

impl TokenType<'_> {
    fn binop_strength(&self) -> u8 {
        match self {
            TokenType::Or => 0,
            TokenType::And => 1,
            TokenType::Less
            | TokenType::Greater
            | TokenType::Leq
            | TokenType::Geq
            | TokenType::Eq
            | TokenType::Neq => 2,
            TokenType::BitOr => 3,
            TokenType::BitXor => 4,
            TokenType::BitAnd => 5,
            TokenType::ShiftL | TokenType::ShiftR => 6,
            TokenType::Concat => 7,
            TokenType::Add | TokenType::Sub => 8,
            TokenType::Mul | TokenType::Div | TokenType::Idiv | TokenType::Mod => 9,
            TokenType::Pow => 10,
            _ => u8::MIN,
        }
    }

    /// Test precedence of tokens
    ///
    /// Lua 5.4 Precedence is as follows  
    ///  `or`  
    ///  `and`  
    ///  `<`     `>`     `<=`    `>=`    `~=`    `==`  
    ///  `|`  
    ///  `~`  
    ///  `&`  
    ///  `<<`    `>>`  
    ///  `..`  
    ///  `+`     `-`  
    ///  `*`     `/`     `//`    `%`  
    ///  unary operators (`not`   `#`     `-`     `~`)  
    ///  `^`  
    ///
    /// `..` and `^` are right associative
    pub fn precedence(&self, lookahead: TokenType) -> Precedence {
        let lhs = self.binop_strength();
        let rhs = lookahead.binop_strength();
        // Unary operators are treated on different states, so
        // no need to bother with them here
        match (self, lhs.cmp(&rhs)) {
            // Right associative
            (TokenType::Concat | TokenType::Pow, Ordering::Equal) => Precedence::Shift,
            // Left associative
            (_, Ordering::Equal) => Precedence::Reduce,
            // Lower precedence
            (_, Ordering::Less) => Precedence::Shift,
            // Higher precedence
            (_, Ordering::Greater) => Precedence::Reduce,
        }
    }
}

impl Display for TokenType<'_> {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // Terminals
            Self::And => write!(f, "and"),
            Self::Break => write!(f, "break"),
            Self::Do => write!(f, "do"),
            Self::Else => write!(f, "else"),
            Self::Elseif => write!(f, "elseif"),
            Self::End => write!(f, "end"),
            Self::False => write!(f, "false"),
            Self::For => write!(f, "for"),
            Self::Function => write!(f, "function"),
            Self::Goto => write!(f, "goto"),
            Self::If => write!(f, "if"),
            Self::In => write!(f, "in"),
            Self::Local => write!(f, "local"),
            Self::Nil => write!(f, "nil"),
            Self::Not => write!(f, "not"),
            Self::Or => write!(f, "or"),
            Self::Repeat => write!(f, "repeat"),
            Self::Return => write!(f, "return"),
            Self::Then => write!(f, "then"),
            Self::True => write!(f, "true"),
            Self::Until => write!(f, "until"),
            Self::While => write!(f, "while"),
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Pow => write!(f, "^"),
            Self::Len => write!(f, "#"),
            Self::BitAnd => write!(f, "&"),
            Self::BitOr => write!(f, "|"),
            Self::BitXor => write!(f, "~"),
            Self::ShiftL => write!(f, "<<"),
            Self::ShiftR => write!(f, ">>"),
            Self::Idiv => write!(f, "//"),
            Self::Eq => write!(f, "=="),
            Self::Neq => write!(f, "~="),
            Self::Leq => write!(f, "<="),
            Self::Geq => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::Greater => write!(f, ">"),
            Self::Assign => write!(f, "="),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LCurly => write!(f, "{{"),
            Self::RCurly => write!(f, "}}"),
            Self::LSquare => write!(f, "["),
            Self::RSquare => write!(f, "]"),
            Self::SemiColon => write!(f, ";"),
            Self::Colon => write!(f, ":"),
            Self::DoubleColon => write!(f, "::"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Concat => write!(f, ".."),
            Self::Dots => write!(f, "..."),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Float(float) => write!(f, "{}", float),
            Self::String(s) => write!(f, "{}", s),
            Self::Name(n) => write!(f, "{}", n),
            Self::Eof => write!(f, "eof"),
            // Non-terminals
            Self::Chunk => write!(f, "chunk"),
            Self::Block => write!(f, "block"),
            Self::BlockStat => write!(f, "block_stat"),
            Self::BlockRetstat => write!(f, "block_retstat"),
            Self::Stat => write!(f, "stat"),
            Self::StatIf => write!(f, "stat_if"),
            Self::StatForexp => write!(f, "stat_forexp"),
            Self::StatAttexplist => write!(f, "stat_attexplist"),
            Self::Attnamelist => write!(f, "attnamelist"),
            Self::AttnamelistCont => write!(f, "attnamelist_cont"),
            Self::Attrib => write!(f, "attrib"),
            Self::Retstat => write!(f, "retstat"),
            Self::RetstatExplist => write!(f, "retstat_explist"),
            Self::RetstatEnd => write!(f, "retstat_end"),
            Self::Label => write!(f, "label"),
            Self::Funcname => write!(f, "funcname"),
            Self::FuncnameCont => write!(f, "funcname_cont"),
            Self::FuncnameEnd => write!(f, "funcname_end"),
            Self::Varlist => write!(f, "varlist"),
            Self::VarlistCont => write!(f, "varlist_cont"),
            Self::Var => write!(f, "var"),
            Self::Namelist => write!(f, "namelist"),
            Self::NamelistCont => write!(f, "namelist_cont"),
            Self::Explist => write!(f, "explist"),
            Self::ExplistCont => write!(f, "explist_cont"),
            Self::Exp => write!(f, "exp"),
            Self::Prefixexp => write!(f, "prefixexp"),
            Self::Functioncall => write!(f, "functioncall"),
            Self::Args => write!(f, "args"),
            Self::ArgsExplist => write!(f, "args_explist"),
            Self::Functiondef => write!(f, "functiondef"),
            Self::Funcbody => write!(f, "funcbody"),
            Self::FuncbodyParlist => write!(f, "funcbody_parlist"),
            Self::Parlist => write!(f, "parlist"),
            Self::ParlistCont => write!(f, "parlist_cont"),
            Self::Tableconstructor => write!(f, "tableconstructor"),
            Self::TableconstructorFieldlist => write!(f, "tableconstructor_fieldlist"),
            Self::Fieldlist => write!(f, "fieldlist"),
            Self::FieldlistCont => write!(f, "fieldlist_cont"),
            Self::Field => write!(f, "field"),
            Self::Fieldsep => write!(f, "fieldsep"),
            Self::Binop => write!(f, "binop"),
            Self::Unop => write!(f, "unop"),
        }
    }
}

impl<'a, T: Borrow<Lexeme<'a>>> From<T> for Token<'a> {
    #[allow(clippy::too_many_lines)]
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

#[derive(Debug, PartialEq)]
pub enum Precedence {
    /// The lookahead token has higher precedence
    Shift,
    /// The lookahead token has lower precedence
    Reduce,
}

impl Precedence {
    pub fn resolve<const REDUCE: usize>(
        self,
        parser: &mut Parser,
        shift: usize,
    ) -> Result<(), Error> {
        match self {
            Self::Shift => parser.shift(shift),
            Self::Reduce => parser.reduce::<REDUCE>(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_precedence() {
        let tests = [
            (TokenType::Or, TokenType::Or, Precedence::Reduce),
            (TokenType::Or, TokenType::And, Precedence::Shift),
            (TokenType::Or, TokenType::Less, Precedence::Shift),
            (TokenType::Or, TokenType::Greater, Precedence::Shift),
            (TokenType::Or, TokenType::Leq, Precedence::Shift),
            (TokenType::Or, TokenType::Geq, Precedence::Shift),
            (TokenType::Or, TokenType::Eq, Precedence::Shift),
            (TokenType::Or, TokenType::Neq, Precedence::Shift),
            (TokenType::Or, TokenType::BitOr, Precedence::Shift),
            (TokenType::Or, TokenType::BitXor, Precedence::Shift),
            (TokenType::Or, TokenType::BitAnd, Precedence::Shift),
            (TokenType::Or, TokenType::ShiftL, Precedence::Shift),
            (TokenType::Or, TokenType::ShiftR, Precedence::Shift),
            (TokenType::Or, TokenType::Concat, Precedence::Shift),
            (TokenType::Or, TokenType::Add, Precedence::Shift),
            (TokenType::Or, TokenType::Sub, Precedence::Shift),
            (TokenType::Or, TokenType::Mul, Precedence::Shift),
            (TokenType::Or, TokenType::Div, Precedence::Shift),
            (TokenType::Or, TokenType::Idiv, Precedence::Shift),
            (TokenType::Or, TokenType::Mod, Precedence::Shift),
            (TokenType::Or, TokenType::Pow, Precedence::Shift),
            (TokenType::Or, TokenType::Comma, Precedence::Reduce),
            (TokenType::And, TokenType::Or, Precedence::Reduce),
            (TokenType::And, TokenType::And, Precedence::Reduce),
            (TokenType::And, TokenType::Less, Precedence::Shift),
            (TokenType::And, TokenType::Greater, Precedence::Shift),
            (TokenType::And, TokenType::Leq, Precedence::Shift),
            (TokenType::And, TokenType::Geq, Precedence::Shift),
            (TokenType::And, TokenType::Eq, Precedence::Shift),
            (TokenType::And, TokenType::Neq, Precedence::Shift),
            (TokenType::And, TokenType::BitOr, Precedence::Shift),
            (TokenType::And, TokenType::BitXor, Precedence::Shift),
            (TokenType::And, TokenType::BitAnd, Precedence::Shift),
            (TokenType::And, TokenType::ShiftL, Precedence::Shift),
            (TokenType::And, TokenType::ShiftR, Precedence::Shift),
            (TokenType::And, TokenType::Concat, Precedence::Shift),
            (TokenType::And, TokenType::Add, Precedence::Shift),
            (TokenType::And, TokenType::Sub, Precedence::Shift),
            (TokenType::And, TokenType::Mul, Precedence::Shift),
            (TokenType::And, TokenType::Div, Precedence::Shift),
            (TokenType::And, TokenType::Idiv, Precedence::Shift),
            (TokenType::And, TokenType::Mod, Precedence::Shift),
            (TokenType::And, TokenType::Pow, Precedence::Shift),
            (TokenType::And, TokenType::Comma, Precedence::Reduce),
            (TokenType::Less, TokenType::Or, Precedence::Reduce),
            (TokenType::Less, TokenType::And, Precedence::Reduce),
            (TokenType::Less, TokenType::Less, Precedence::Reduce),
            (TokenType::Less, TokenType::Greater, Precedence::Reduce),
            (TokenType::Less, TokenType::Leq, Precedence::Reduce),
            (TokenType::Less, TokenType::Geq, Precedence::Reduce),
            (TokenType::Less, TokenType::Eq, Precedence::Reduce),
            (TokenType::Less, TokenType::Neq, Precedence::Reduce),
            (TokenType::Less, TokenType::BitOr, Precedence::Shift),
            (TokenType::Less, TokenType::BitXor, Precedence::Shift),
            (TokenType::Less, TokenType::BitAnd, Precedence::Shift),
            (TokenType::Less, TokenType::ShiftL, Precedence::Shift),
            (TokenType::Less, TokenType::ShiftR, Precedence::Shift),
            (TokenType::Less, TokenType::Concat, Precedence::Shift),
            (TokenType::Less, TokenType::Add, Precedence::Shift),
            (TokenType::Less, TokenType::Sub, Precedence::Shift),
            (TokenType::Less, TokenType::Mul, Precedence::Shift),
            (TokenType::Less, TokenType::Div, Precedence::Shift),
            (TokenType::Less, TokenType::Idiv, Precedence::Shift),
            (TokenType::Less, TokenType::Mod, Precedence::Shift),
            (TokenType::Less, TokenType::Pow, Precedence::Shift),
            (TokenType::Less, TokenType::Comma, Precedence::Reduce),
            (TokenType::Greater, TokenType::Or, Precedence::Reduce),
            (TokenType::Greater, TokenType::And, Precedence::Reduce),
            (TokenType::Greater, TokenType::Less, Precedence::Reduce),
            (TokenType::Greater, TokenType::Greater, Precedence::Reduce),
            (TokenType::Greater, TokenType::Leq, Precedence::Reduce),
            (TokenType::Greater, TokenType::Geq, Precedence::Reduce),
            (TokenType::Greater, TokenType::Eq, Precedence::Reduce),
            (TokenType::Greater, TokenType::Neq, Precedence::Reduce),
            (TokenType::Greater, TokenType::BitOr, Precedence::Shift),
            (TokenType::Greater, TokenType::BitXor, Precedence::Shift),
            (TokenType::Greater, TokenType::BitAnd, Precedence::Shift),
            (TokenType::Greater, TokenType::ShiftL, Precedence::Shift),
            (TokenType::Greater, TokenType::ShiftR, Precedence::Shift),
            (TokenType::Greater, TokenType::Concat, Precedence::Shift),
            (TokenType::Greater, TokenType::Add, Precedence::Shift),
            (TokenType::Greater, TokenType::Sub, Precedence::Shift),
            (TokenType::Greater, TokenType::Mul, Precedence::Shift),
            (TokenType::Greater, TokenType::Div, Precedence::Shift),
            (TokenType::Greater, TokenType::Idiv, Precedence::Shift),
            (TokenType::Greater, TokenType::Mod, Precedence::Shift),
            (TokenType::Greater, TokenType::Pow, Precedence::Shift),
            (TokenType::Greater, TokenType::Comma, Precedence::Reduce),
            (TokenType::Leq, TokenType::Or, Precedence::Reduce),
            (TokenType::Leq, TokenType::And, Precedence::Reduce),
            (TokenType::Leq, TokenType::Less, Precedence::Reduce),
            (TokenType::Leq, TokenType::Greater, Precedence::Reduce),
            (TokenType::Leq, TokenType::Leq, Precedence::Reduce),
            (TokenType::Leq, TokenType::Geq, Precedence::Reduce),
            (TokenType::Leq, TokenType::Eq, Precedence::Reduce),
            (TokenType::Leq, TokenType::Neq, Precedence::Reduce),
            (TokenType::Leq, TokenType::BitOr, Precedence::Shift),
            (TokenType::Leq, TokenType::BitXor, Precedence::Shift),
            (TokenType::Leq, TokenType::BitAnd, Precedence::Shift),
            (TokenType::Leq, TokenType::ShiftL, Precedence::Shift),
            (TokenType::Leq, TokenType::ShiftR, Precedence::Shift),
            (TokenType::Leq, TokenType::Concat, Precedence::Shift),
            (TokenType::Leq, TokenType::Add, Precedence::Shift),
            (TokenType::Leq, TokenType::Sub, Precedence::Shift),
            (TokenType::Leq, TokenType::Mul, Precedence::Shift),
            (TokenType::Leq, TokenType::Div, Precedence::Shift),
            (TokenType::Leq, TokenType::Idiv, Precedence::Shift),
            (TokenType::Leq, TokenType::Mod, Precedence::Shift),
            (TokenType::Leq, TokenType::Pow, Precedence::Shift),
            (TokenType::Leq, TokenType::Comma, Precedence::Reduce),
            (TokenType::Geq, TokenType::Or, Precedence::Reduce),
            (TokenType::Geq, TokenType::And, Precedence::Reduce),
            (TokenType::Geq, TokenType::Less, Precedence::Reduce),
            (TokenType::Geq, TokenType::Greater, Precedence::Reduce),
            (TokenType::Geq, TokenType::Leq, Precedence::Reduce),
            (TokenType::Geq, TokenType::Geq, Precedence::Reduce),
            (TokenType::Geq, TokenType::Eq, Precedence::Reduce),
            (TokenType::Geq, TokenType::Neq, Precedence::Reduce),
            (TokenType::Geq, TokenType::BitOr, Precedence::Shift),
            (TokenType::Geq, TokenType::BitXor, Precedence::Shift),
            (TokenType::Geq, TokenType::BitAnd, Precedence::Shift),
            (TokenType::Geq, TokenType::ShiftL, Precedence::Shift),
            (TokenType::Geq, TokenType::ShiftR, Precedence::Shift),
            (TokenType::Geq, TokenType::Concat, Precedence::Shift),
            (TokenType::Geq, TokenType::Add, Precedence::Shift),
            (TokenType::Geq, TokenType::Sub, Precedence::Shift),
            (TokenType::Geq, TokenType::Mul, Precedence::Shift),
            (TokenType::Geq, TokenType::Div, Precedence::Shift),
            (TokenType::Geq, TokenType::Idiv, Precedence::Shift),
            (TokenType::Geq, TokenType::Mod, Precedence::Shift),
            (TokenType::Geq, TokenType::Pow, Precedence::Shift),
            (TokenType::Geq, TokenType::Comma, Precedence::Reduce),
            (TokenType::Eq, TokenType::Or, Precedence::Reduce),
            (TokenType::Eq, TokenType::And, Precedence::Reduce),
            (TokenType::Eq, TokenType::Less, Precedence::Reduce),
            (TokenType::Eq, TokenType::Greater, Precedence::Reduce),
            (TokenType::Eq, TokenType::Leq, Precedence::Reduce),
            (TokenType::Eq, TokenType::Geq, Precedence::Reduce),
            (TokenType::Eq, TokenType::Eq, Precedence::Reduce),
            (TokenType::Eq, TokenType::Neq, Precedence::Reduce),
            (TokenType::Eq, TokenType::BitOr, Precedence::Shift),
            (TokenType::Eq, TokenType::BitXor, Precedence::Shift),
            (TokenType::Eq, TokenType::BitAnd, Precedence::Shift),
            (TokenType::Eq, TokenType::ShiftL, Precedence::Shift),
            (TokenType::Eq, TokenType::ShiftR, Precedence::Shift),
            (TokenType::Eq, TokenType::Concat, Precedence::Shift),
            (TokenType::Eq, TokenType::Add, Precedence::Shift),
            (TokenType::Eq, TokenType::Sub, Precedence::Shift),
            (TokenType::Eq, TokenType::Mul, Precedence::Shift),
            (TokenType::Eq, TokenType::Div, Precedence::Shift),
            (TokenType::Eq, TokenType::Idiv, Precedence::Shift),
            (TokenType::Eq, TokenType::Mod, Precedence::Shift),
            (TokenType::Eq, TokenType::Pow, Precedence::Shift),
            (TokenType::Eq, TokenType::Comma, Precedence::Reduce),
            (TokenType::Neq, TokenType::Or, Precedence::Reduce),
            (TokenType::Neq, TokenType::And, Precedence::Reduce),
            (TokenType::Neq, TokenType::Less, Precedence::Reduce),
            (TokenType::Neq, TokenType::Greater, Precedence::Reduce),
            (TokenType::Neq, TokenType::Leq, Precedence::Reduce),
            (TokenType::Neq, TokenType::Geq, Precedence::Reduce),
            (TokenType::Neq, TokenType::Eq, Precedence::Reduce),
            (TokenType::Neq, TokenType::Neq, Precedence::Reduce),
            (TokenType::Neq, TokenType::BitOr, Precedence::Shift),
            (TokenType::Neq, TokenType::BitXor, Precedence::Shift),
            (TokenType::Neq, TokenType::BitAnd, Precedence::Shift),
            (TokenType::Neq, TokenType::ShiftL, Precedence::Shift),
            (TokenType::Neq, TokenType::ShiftR, Precedence::Shift),
            (TokenType::Neq, TokenType::Concat, Precedence::Shift),
            (TokenType::Neq, TokenType::Add, Precedence::Shift),
            (TokenType::Neq, TokenType::Sub, Precedence::Shift),
            (TokenType::Neq, TokenType::Mul, Precedence::Shift),
            (TokenType::Neq, TokenType::Div, Precedence::Shift),
            (TokenType::Neq, TokenType::Idiv, Precedence::Shift),
            (TokenType::Neq, TokenType::Mod, Precedence::Shift),
            (TokenType::Neq, TokenType::Pow, Precedence::Shift),
            (TokenType::Neq, TokenType::Comma, Precedence::Reduce),
            (TokenType::BitOr, TokenType::Or, Precedence::Reduce),
            (TokenType::BitOr, TokenType::And, Precedence::Reduce),
            (TokenType::BitOr, TokenType::Less, Precedence::Reduce),
            (TokenType::BitOr, TokenType::Greater, Precedence::Reduce),
            (TokenType::BitOr, TokenType::Leq, Precedence::Reduce),
            (TokenType::BitOr, TokenType::Geq, Precedence::Reduce),
            (TokenType::BitOr, TokenType::Eq, Precedence::Reduce),
            (TokenType::BitOr, TokenType::Neq, Precedence::Reduce),
            (TokenType::BitOr, TokenType::BitOr, Precedence::Reduce),
            (TokenType::BitOr, TokenType::BitXor, Precedence::Shift),
            (TokenType::BitOr, TokenType::BitAnd, Precedence::Shift),
            (TokenType::BitOr, TokenType::ShiftL, Precedence::Shift),
            (TokenType::BitOr, TokenType::ShiftR, Precedence::Shift),
            (TokenType::BitOr, TokenType::Concat, Precedence::Shift),
            (TokenType::BitOr, TokenType::Add, Precedence::Shift),
            (TokenType::BitOr, TokenType::Sub, Precedence::Shift),
            (TokenType::BitOr, TokenType::Mul, Precedence::Shift),
            (TokenType::BitOr, TokenType::Div, Precedence::Shift),
            (TokenType::BitOr, TokenType::Idiv, Precedence::Shift),
            (TokenType::BitOr, TokenType::Mod, Precedence::Shift),
            (TokenType::BitOr, TokenType::Pow, Precedence::Shift),
            (TokenType::BitOr, TokenType::Comma, Precedence::Reduce),
            (TokenType::BitXor, TokenType::Or, Precedence::Reduce),
            (TokenType::BitXor, TokenType::And, Precedence::Reduce),
            (TokenType::BitXor, TokenType::Less, Precedence::Reduce),
            (TokenType::BitXor, TokenType::Greater, Precedence::Reduce),
            (TokenType::BitXor, TokenType::Leq, Precedence::Reduce),
            (TokenType::BitXor, TokenType::Geq, Precedence::Reduce),
            (TokenType::BitXor, TokenType::Eq, Precedence::Reduce),
            (TokenType::BitXor, TokenType::Neq, Precedence::Reduce),
            (TokenType::BitXor, TokenType::BitOr, Precedence::Reduce),
            (TokenType::BitXor, TokenType::BitXor, Precedence::Reduce),
            (TokenType::BitXor, TokenType::BitAnd, Precedence::Shift),
            (TokenType::BitXor, TokenType::ShiftL, Precedence::Shift),
            (TokenType::BitXor, TokenType::ShiftR, Precedence::Shift),
            (TokenType::BitXor, TokenType::Concat, Precedence::Shift),
            (TokenType::BitXor, TokenType::Add, Precedence::Shift),
            (TokenType::BitXor, TokenType::Sub, Precedence::Shift),
            (TokenType::BitXor, TokenType::Mul, Precedence::Shift),
            (TokenType::BitXor, TokenType::Div, Precedence::Shift),
            (TokenType::BitXor, TokenType::Idiv, Precedence::Shift),
            (TokenType::BitXor, TokenType::Mod, Precedence::Shift),
            (TokenType::BitXor, TokenType::Pow, Precedence::Shift),
            (TokenType::BitXor, TokenType::Comma, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::Or, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::And, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::Less, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::Greater, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::Leq, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::Geq, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::Eq, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::Neq, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::BitOr, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::BitXor, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::BitAnd, TokenType::ShiftL, Precedence::Shift),
            (TokenType::BitAnd, TokenType::ShiftR, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Concat, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Add, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Sub, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Mul, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Div, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Idiv, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Mod, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Pow, Precedence::Shift),
            (TokenType::BitAnd, TokenType::Comma, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Or, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::And, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Less, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Greater, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Leq, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Geq, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Eq, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Neq, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::BitOr, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::BitXor, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::ShiftL, TokenType::Concat, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Add, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Sub, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Mul, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Div, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Idiv, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Mod, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Pow, Precedence::Shift),
            (TokenType::ShiftL, TokenType::Comma, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Or, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::And, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Less, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Greater, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Leq, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Geq, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Eq, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Neq, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::BitOr, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::BitXor, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::ShiftR, TokenType::Concat, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Add, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Sub, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Mul, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Div, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Idiv, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Mod, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Pow, Precedence::Shift),
            (TokenType::ShiftR, TokenType::Comma, Precedence::Reduce),
            (TokenType::Concat, TokenType::Or, Precedence::Reduce),
            (TokenType::Concat, TokenType::And, Precedence::Reduce),
            (TokenType::Concat, TokenType::Less, Precedence::Reduce),
            (TokenType::Concat, TokenType::Greater, Precedence::Reduce),
            (TokenType::Concat, TokenType::Leq, Precedence::Reduce),
            (TokenType::Concat, TokenType::Geq, Precedence::Reduce),
            (TokenType::Concat, TokenType::Eq, Precedence::Reduce),
            (TokenType::Concat, TokenType::Neq, Precedence::Reduce),
            (TokenType::Concat, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Concat, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Concat, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Concat, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Concat, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Concat, TokenType::Concat, Precedence::Shift),
            (TokenType::Concat, TokenType::Add, Precedence::Shift),
            (TokenType::Concat, TokenType::Sub, Precedence::Shift),
            (TokenType::Concat, TokenType::Mul, Precedence::Shift),
            (TokenType::Concat, TokenType::Div, Precedence::Shift),
            (TokenType::Concat, TokenType::Idiv, Precedence::Shift),
            (TokenType::Concat, TokenType::Mod, Precedence::Shift),
            (TokenType::Concat, TokenType::Pow, Precedence::Shift),
            (TokenType::Concat, TokenType::Comma, Precedence::Reduce),
            (TokenType::Add, TokenType::Or, Precedence::Reduce),
            (TokenType::Add, TokenType::And, Precedence::Reduce),
            (TokenType::Add, TokenType::Less, Precedence::Reduce),
            (TokenType::Add, TokenType::Greater, Precedence::Reduce),
            (TokenType::Add, TokenType::Leq, Precedence::Reduce),
            (TokenType::Add, TokenType::Geq, Precedence::Reduce),
            (TokenType::Add, TokenType::Eq, Precedence::Reduce),
            (TokenType::Add, TokenType::Neq, Precedence::Reduce),
            (TokenType::Add, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Add, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Add, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Add, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Add, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Add, TokenType::Concat, Precedence::Reduce),
            (TokenType::Add, TokenType::Add, Precedence::Reduce),
            (TokenType::Add, TokenType::Sub, Precedence::Reduce),
            (TokenType::Add, TokenType::Mul, Precedence::Shift),
            (TokenType::Add, TokenType::Div, Precedence::Shift),
            (TokenType::Add, TokenType::Idiv, Precedence::Shift),
            (TokenType::Add, TokenType::Mod, Precedence::Shift),
            (TokenType::Add, TokenType::Pow, Precedence::Shift),
            (TokenType::Add, TokenType::Comma, Precedence::Reduce),
            (TokenType::Sub, TokenType::Or, Precedence::Reduce),
            (TokenType::Sub, TokenType::And, Precedence::Reduce),
            (TokenType::Sub, TokenType::Less, Precedence::Reduce),
            (TokenType::Sub, TokenType::Greater, Precedence::Reduce),
            (TokenType::Sub, TokenType::Leq, Precedence::Reduce),
            (TokenType::Sub, TokenType::Geq, Precedence::Reduce),
            (TokenType::Sub, TokenType::Eq, Precedence::Reduce),
            (TokenType::Sub, TokenType::Neq, Precedence::Reduce),
            (TokenType::Sub, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Sub, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Sub, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Sub, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Sub, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Sub, TokenType::Concat, Precedence::Reduce),
            (TokenType::Sub, TokenType::Add, Precedence::Reduce),
            (TokenType::Sub, TokenType::Sub, Precedence::Reduce),
            (TokenType::Sub, TokenType::Mul, Precedence::Shift),
            (TokenType::Sub, TokenType::Div, Precedence::Shift),
            (TokenType::Sub, TokenType::Idiv, Precedence::Shift),
            (TokenType::Sub, TokenType::Mod, Precedence::Shift),
            (TokenType::Sub, TokenType::Pow, Precedence::Shift),
            (TokenType::Sub, TokenType::Comma, Precedence::Reduce),
            (TokenType::Mul, TokenType::Or, Precedence::Reduce),
            (TokenType::Mul, TokenType::And, Precedence::Reduce),
            (TokenType::Mul, TokenType::Less, Precedence::Reduce),
            (TokenType::Mul, TokenType::Greater, Precedence::Reduce),
            (TokenType::Mul, TokenType::Leq, Precedence::Reduce),
            (TokenType::Mul, TokenType::Geq, Precedence::Reduce),
            (TokenType::Mul, TokenType::Eq, Precedence::Reduce),
            (TokenType::Mul, TokenType::Neq, Precedence::Reduce),
            (TokenType::Mul, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Mul, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Mul, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Mul, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Mul, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Mul, TokenType::Concat, Precedence::Reduce),
            (TokenType::Mul, TokenType::Add, Precedence::Reduce),
            (TokenType::Mul, TokenType::Sub, Precedence::Reduce),
            (TokenType::Mul, TokenType::Mul, Precedence::Reduce),
            (TokenType::Mul, TokenType::Div, Precedence::Reduce),
            (TokenType::Mul, TokenType::Idiv, Precedence::Reduce),
            (TokenType::Mul, TokenType::Mod, Precedence::Reduce),
            (TokenType::Mul, TokenType::Pow, Precedence::Shift),
            (TokenType::Mul, TokenType::Comma, Precedence::Reduce),
            (TokenType::Div, TokenType::Or, Precedence::Reduce),
            (TokenType::Div, TokenType::And, Precedence::Reduce),
            (TokenType::Div, TokenType::Less, Precedence::Reduce),
            (TokenType::Div, TokenType::Greater, Precedence::Reduce),
            (TokenType::Div, TokenType::Leq, Precedence::Reduce),
            (TokenType::Div, TokenType::Geq, Precedence::Reduce),
            (TokenType::Div, TokenType::Eq, Precedence::Reduce),
            (TokenType::Div, TokenType::Neq, Precedence::Reduce),
            (TokenType::Div, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Div, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Div, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Div, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Div, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Div, TokenType::Concat, Precedence::Reduce),
            (TokenType::Div, TokenType::Add, Precedence::Reduce),
            (TokenType::Div, TokenType::Sub, Precedence::Reduce),
            (TokenType::Div, TokenType::Mul, Precedence::Reduce),
            (TokenType::Div, TokenType::Div, Precedence::Reduce),
            (TokenType::Div, TokenType::Idiv, Precedence::Reduce),
            (TokenType::Div, TokenType::Mod, Precedence::Reduce),
            (TokenType::Div, TokenType::Pow, Precedence::Shift),
            (TokenType::Div, TokenType::Comma, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Or, Precedence::Reduce),
            (TokenType::Idiv, TokenType::And, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Less, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Greater, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Leq, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Geq, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Eq, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Neq, Precedence::Reduce),
            (TokenType::Idiv, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Idiv, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Idiv, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Idiv, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Idiv, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Concat, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Add, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Sub, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Mul, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Div, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Idiv, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Mod, Precedence::Reduce),
            (TokenType::Idiv, TokenType::Pow, Precedence::Shift),
            (TokenType::Idiv, TokenType::Comma, Precedence::Reduce),
            (TokenType::Mod, TokenType::Or, Precedence::Reduce),
            (TokenType::Mod, TokenType::And, Precedence::Reduce),
            (TokenType::Mod, TokenType::Less, Precedence::Reduce),
            (TokenType::Mod, TokenType::Greater, Precedence::Reduce),
            (TokenType::Mod, TokenType::Leq, Precedence::Reduce),
            (TokenType::Mod, TokenType::Geq, Precedence::Reduce),
            (TokenType::Mod, TokenType::Eq, Precedence::Reduce),
            (TokenType::Mod, TokenType::Neq, Precedence::Reduce),
            (TokenType::Mod, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Mod, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Mod, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Mod, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Mod, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Mod, TokenType::Concat, Precedence::Reduce),
            (TokenType::Mod, TokenType::Add, Precedence::Reduce),
            (TokenType::Mod, TokenType::Sub, Precedence::Reduce),
            (TokenType::Mod, TokenType::Mul, Precedence::Reduce),
            (TokenType::Mod, TokenType::Div, Precedence::Reduce),
            (TokenType::Mod, TokenType::Idiv, Precedence::Reduce),
            (TokenType::Mod, TokenType::Mod, Precedence::Reduce),
            (TokenType::Mod, TokenType::Pow, Precedence::Shift),
            (TokenType::Mod, TokenType::Comma, Precedence::Reduce),
            (TokenType::Pow, TokenType::Or, Precedence::Reduce),
            (TokenType::Pow, TokenType::And, Precedence::Reduce),
            (TokenType::Pow, TokenType::Less, Precedence::Reduce),
            (TokenType::Pow, TokenType::Greater, Precedence::Reduce),
            (TokenType::Pow, TokenType::Leq, Precedence::Reduce),
            (TokenType::Pow, TokenType::Geq, Precedence::Reduce),
            (TokenType::Pow, TokenType::Eq, Precedence::Reduce),
            (TokenType::Pow, TokenType::Neq, Precedence::Reduce),
            (TokenType::Pow, TokenType::BitOr, Precedence::Reduce),
            (TokenType::Pow, TokenType::BitXor, Precedence::Reduce),
            (TokenType::Pow, TokenType::BitAnd, Precedence::Reduce),
            (TokenType::Pow, TokenType::ShiftL, Precedence::Reduce),
            (TokenType::Pow, TokenType::ShiftR, Precedence::Reduce),
            (TokenType::Pow, TokenType::Concat, Precedence::Reduce),
            (TokenType::Pow, TokenType::Add, Precedence::Reduce),
            (TokenType::Pow, TokenType::Sub, Precedence::Reduce),
            (TokenType::Pow, TokenType::Mul, Precedence::Reduce),
            (TokenType::Pow, TokenType::Div, Precedence::Reduce),
            (TokenType::Pow, TokenType::Idiv, Precedence::Reduce),
            (TokenType::Pow, TokenType::Mod, Precedence::Reduce),
            (TokenType::Pow, TokenType::Pow, Precedence::Shift),
            (TokenType::Pow, TokenType::Comma, Precedence::Reduce),
        ];

        for (lhs, rhs, res) in tests {
            assert_eq!(lhs.precedence(rhs), res, "{:?} <=> {:?}", lhs, rhs);
        }
    }
}
