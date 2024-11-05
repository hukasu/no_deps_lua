#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub(crate) line: usize,
    pub(crate) column: usize,
    /// How many bytes where read while processing this token.
    ///
    /// This might be across multiple lines
    pub(crate) start_offset: usize,
    pub(crate) token: TokenType<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType<'a> {
    // Keywords
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

    /// Addition (`+`)
    Add,
    /// Subtraction (`-`)
    Sub,
    /// Multiplication (`*`)
    Mul,
    /// Float division (`/`)
    Div,
    /// Modulus (`%`)
    Mod,
    /// Power (`^`)
    Pow,
    /// Length (`#`)
    Len,
    /// Bitwise and (`&`)
    BitAnd,
    /// Bitwise xor (`~`)
    BitXor,
    /// Bitwise or (`|`)
    BitOr,
    /// Left shift (`>>`)
    ShiftL,
    /// Right shift (`>>`)
    ShiftR,
    /// Integer division (`//`)
    Idiv,
    /// Equality (`==`)
    Eq,
    /// Inequality (`~=`)
    Neq,
    /// Less than or equal (`<=`)
    Leq,
    /// Greater than or equal (`>=`)
    Geq,
    /// Less (`<`)
    Less,
    /// Greater (`>`)
    Greater,
    /// Assign (`=`)
    Assign,
    /// Left parenthesis (`(`)
    LParen,
    /// Right parenthesis (`)`)
    RParen,
    /// Left curly braces (`{`)
    LCurly,
    /// Right parenthesis (`}`)
    RCurly,
    /// Left square brackets (`[`)
    LSquare,
    /// Right square brackets (`]`)
    RSquare,
    /// Double Colon (`::`)
    DoubleColon,
    /// Semi Colon (`;`)
    SemiColon,
    /// Colon (`:`)
    Colon,
    /// Comma (`,`)
    Comma,
    /// Dot (`.`)
    Dot,
    /// Concatenation (`..`)
    Concat,
    /// Dots (`..`)
    Dots,

    /// Integer
    Integer(i64),
    /// Floating-point number
    Float(f64),
    /// String
    String(&'a str),

    /// Name of value or table key
    Name(&'a str),
}
