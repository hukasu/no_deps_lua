use core::{borrow::Borrow, fmt::Display};

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
    StatElseif,
    StatElse,
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
}

impl<'a> Display for TokenType<'a> {
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
            Self::StatElseif => write!(f, "stat_elseif"),
            Self::StatElse => write!(f, "stat_else"),
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
        }
    }
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
