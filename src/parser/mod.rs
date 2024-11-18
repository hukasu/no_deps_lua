mod error;
mod token;

use core::iter::Peekable;

use alloc::vec::Vec;

use crate::lex::{Lex, Lexeme};

pub use self::{
    error::Error,
    token::{Token, TokenType},
};

type LexemeResToTokenRes =
    &'static dyn Fn(Result<Lexeme, crate::lex::Error>) -> Result<Token, crate::lex::Error>;

macro_rules! make_state {
    (0, $lookahead:ident) => {
        (
            [0],
            Some(Ok(Token {
                tokens: _,
                token_type: make_token_type!($lookahead),
            })),
        )
    };
    ($cur_state:expr, $lookahead:ident) => {
        (
            [_states_head @ .., $cur_state],
            Some(Ok(Token {
                tokens: _,
                token_type: make_token_type!($lookahead),
            })),
        )
    };
}

macro_rules! make_token_type {
    (Integer) => {
        TokenType::Integer(_)
    };
    (Float) => {
        TokenType::Float(_)
    };
    (Name) => {
        TokenType::Name(_)
    };
    (String) => {
        TokenType::String(_)
    };
    ($other:ident) => {
        TokenType::$other
    };
}

macro_rules! make_reduction_push {
    ($parser:ident, $token_type:ident) => {
        {
            $parser.reduction.replace(Ok(Token {
                tokens: [].to_vec(),
                token_type: TokenType::$token_type,
            }));
            Ok(())
        }
    };
    (
        $parser:ident,
        $count:expr,
        $token_type:ident,
        $($var_type:ident),+
    ) => {
        {
            let mut stack_pop = $parser.stack_pop($count);
            stack_pop.reverse();
            if !matches!(
                stack_pop.as_slice(),
                [
                    $(Token {
                        tokens: _,
                        token_type: make_token_type!($var_type),
                    },)+
                ]
            ) {
                log::error!(
                    "Failed to reduce.\n\tExpected: {:?}\n\tGot: {:?}",
                    [$(stringify!($var_type),)+],
                    stack_pop.into_iter().map(|token| token.token_type).collect::<Vec<_>>(),
                );
                Err(Error::Reduction)
            } else {
                $parser.reduction.replace(Ok(Token {
                    tokens: stack_pop,
                    token_type: TokenType::$token_type,
                }));
                Ok(())
            }
        }
    };
}

pub struct Parser<'a> {
    lexeme_stream: Peekable<core::iter::Map<Lex<'a>, LexemeResToTokenRes>>,
    states: Vec<usize>,
    stack: Vec<Token<'a>>,
    reduction: Option<Result<Token<'a>, crate::lex::Error>>,
}

impl<'a> Parser<'a> {
    pub fn parse(program: &'a str) -> Result<Token<'a>, Error> {
        let map: LexemeResToTokenRes =
            &|res: Result<Lexeme, crate::lex::Error>| res.map(Token::from);
        let mut parser = Parser {
            lexeme_stream: Lex::new(program).map(map).peekable(),
            states: [0].to_vec(),
            stack: [].to_vec(),
            reduction: None,
        };

        loop {
            match parser.state() {
                (_, Some(Err(err))) => {
                    log::error!("Failed to parse due to a lexical error. {}", err);
                    break Err(Error::Lex);
                }
                // State 0
                make_state!(0, Name) => parser.shift(19),
                make_state!(0, Local) => parser.shift(10),
                make_state!(0, Block) => parser.goto(1),
                make_state!(0, BlockStat) => parser.goto(2),
                make_state!(0, Stat) => parser.goto(3),
                make_state!(0, Var) => parser.goto(18),
                make_state!(0, Prefixexp) => parser.goto(20),
                make_state!(0, Functioncall) => parser.goto(16),
                // State 1
                make_state!(1, Eof) => break parser.accept(),
                // State 2
                make_state!(2, Eof) => parser.reduce::<4>()?,
                make_state!(2, BlockRetstat) => parser.goto(22),
                // State 3
                make_state!(3, Name) => parser.shift(19),
                make_state!(3, Break) => parser.shift(4),
                make_state!(3, Do) => parser.shift(5),
                make_state!(3, For) => parser.shift(6),
                make_state!(3, Function) => parser.shift(7),
                make_state!(3, Goto) => parser.shift(8),
                make_state!(3, If) => parser.shift(9),
                make_state!(3, Local) => parser.shift(10),
                make_state!(3, Repeat) => parser.shift(11),
                make_state!(3, Return) => parser.reduce::<2>()?,
                make_state!(3, While) => parser.shift(12),
                make_state!(3, LParen) => parser.shift(21),
                make_state!(3, DoubleColon) => parser.shift(17),
                make_state!(3, SemiColon) => parser.shift(13),
                make_state!(3, Eof) => parser.reduce::<2>()?,
                make_state!(3, BlockStat) => parser.goto(48),
                make_state!(3, Stat) => parser.goto(3),
                make_state!(3, Label) => parser.goto(14),
                make_state!(3, Varlist) => parser.goto(15),
                make_state!(3, Var) => parser.goto(18),
                make_state!(3, Prefixexp) => parser.goto(20),
                make_state!(3, Functioncall) => parser.goto(16),
                // State 10
                make_state!(10, Name) => parser.shift(39),
                make_state!(10, Attnamelist) => parser.goto(34),
                // State 16
                make_state!(16, Name) => parser.reduce::<8>()?,
                make_state!(16, String) => parser.reduce::<93>()?,
                make_state!(16, Break) => parser.reduce::<8>()?,
                make_state!(16, Do) => parser.reduce::<8>()?,
                make_state!(16, For) => parser.reduce::<8>()?,
                make_state!(16, Function) => parser.reduce::<8>()?,
                make_state!(16, Goto) => parser.reduce::<8>()?,
                make_state!(16, If) => parser.reduce::<8>()?,
                make_state!(16, Local) => parser.reduce::<8>()?,
                make_state!(16, Repeat) => parser.reduce::<8>()?,
                make_state!(16, Return) => parser.reduce::<8>()?,
                make_state!(16, While) => parser.reduce::<8>()?,
                // Reduce(8)-Reduce(93) resolved by reducing by the first production
                make_state!(16, LParen) => parser.reduce::<8>()?,
                make_state!(16, LSquare) => parser.reduce::<93>()?,
                make_state!(16, LCurly) => parser.reduce::<93>()?,
                make_state!(16, DoubleColon) => parser.reduce::<8>()?,
                make_state!(16, SemiColon) => parser.reduce::<8>()?,
                make_state!(16, Colon) => parser.reduce::<93>()?,
                make_state!(16, Dot) => parser.reduce::<93>()?,
                make_state!(16, Eof) => parser.reduce::<8>()?,
                // State 18
                make_state!(18, String) => parser.reduce::<92>()?,
                make_state!(18, LParen) => parser.reduce::<92>()?,
                // State 19
                make_state!(19, String) => parser.reduce::<48>()?,
                make_state!(19, LParen) => parser.reduce::<48>()?,
                // State 20
                make_state!(20, String) => parser.shift(85),
                make_state!(20, LParen) => parser.shift(86),
                make_state!(20, Args) => parser.goto(84),
                // State 22
                make_state!(22, Eof) => parser.reduce::<1>()?,
                // State 34
                make_state!(34, Assign) => parser.shift(242),
                make_state!(34, StatAttexplist) => parser.goto(154),
                // State 39
                make_state!(39, Assign) => parser.reduce::<32>()?,
                make_state!(39, Attrib) => parser.goto(155),
                // State 48
                make_state!(48, Eof) => parser.reduce::<3>()?,
                // State 84
                make_state!(84, Name) => parser.reduce::<95>()?,
                make_state!(84, String) => parser.reduce::<95>()?,
                make_state!(84, Break) => parser.reduce::<95>()?,
                make_state!(84, Do) => parser.reduce::<95>()?,
                make_state!(84, For) => parser.reduce::<95>()?,
                make_state!(84, Function) => parser.reduce::<95>()?,
                make_state!(84, Goto) => parser.reduce::<95>()?,
                make_state!(84, If) => parser.reduce::<95>()?,
                make_state!(84, Local) => parser.reduce::<95>()?,
                make_state!(84, Repeat) => parser.reduce::<95>()?,
                make_state!(84, Return) => parser.reduce::<95>()?,
                make_state!(84, While) => parser.reduce::<95>()?,
                make_state!(84, LParen) => parser.reduce::<95>()?,
                make_state!(84, LSquare) => parser.reduce::<95>()?,
                make_state!(84, LCurly) => parser.reduce::<95>()?,
                make_state!(84, DoubleColon) => parser.reduce::<95>()?,
                make_state!(84, SemiColon) => parser.reduce::<95>()?,
                make_state!(84, Colon) => parser.reduce::<95>()?,
                make_state!(84, Dot) => parser.reduce::<95>()?,
                make_state!(84, Eof) => parser.reduce::<95>()?,
                // State 85
                make_state!(85, Name) => parser.reduce::<101>()?,
                make_state!(85, String) => parser.reduce::<101>()?,
                make_state!(85, Break) => parser.reduce::<101>()?,
                make_state!(85, Do) => parser.reduce::<101>()?,
                make_state!(85, For) => parser.reduce::<101>()?,
                make_state!(85, Function) => parser.reduce::<101>()?,
                make_state!(85, Goto) => parser.reduce::<101>()?,
                make_state!(85, If) => parser.reduce::<101>()?,
                make_state!(85, Local) => parser.reduce::<101>()?,
                make_state!(85, Repeat) => parser.reduce::<101>()?,
                make_state!(85, Return) => parser.reduce::<101>()?,
                make_state!(85, While) => parser.reduce::<101>()?,
                make_state!(85, LParen) => parser.reduce::<101>()?,
                make_state!(85, LSquare) => parser.reduce::<101>()?,
                make_state!(85, LCurly) => parser.reduce::<101>()?,
                make_state!(85, DoubleColon) => parser.reduce::<101>()?,
                make_state!(85, SemiColon) => parser.reduce::<101>()?,
                make_state!(85, Colon) => parser.reduce::<101>()?,
                make_state!(85, Dot) => parser.reduce::<101>()?,
                make_state!(85, Eof) => parser.reduce::<101>()?,
                // State 86
                make_state!(86, Name) => parser.shift(186),
                make_state!(86, Integer) => parser.shift(220),
                make_state!(86, Float) => parser.shift(221),
                make_state!(86, False) => parser.shift(222),
                make_state!(86, Nil) => parser.shift(223),
                make_state!(86, True) => parser.shift(305),
                make_state!(86, Var) => parser.goto(367),
                make_state!(86, Explist) => parser.goto(370),
                make_state!(86, Exp) => parser.goto(218),
                make_state!(86, Prefixexp) => parser.goto(192),
                make_state!(86, ArgsExplist) => parser.goto(369),
                // State 154
                make_state!(154, Name) => parser.reduce::<26>()?,
                make_state!(154, Break) => parser.reduce::<26>()?,
                make_state!(154, Do) => parser.reduce::<26>()?,
                make_state!(154, For) => parser.reduce::<26>()?,
                make_state!(154, Function) => parser.reduce::<26>()?,
                make_state!(154, Goto) => parser.reduce::<26>()?,
                make_state!(154, If) => parser.reduce::<26>()?,
                make_state!(154, Local) => parser.reduce::<26>()?,
                make_state!(154, Repeat) => parser.reduce::<26>()?,
                make_state!(154, Return) => parser.reduce::<26>()?,
                make_state!(154, While) => parser.reduce::<26>()?,
                make_state!(154, LParen) => parser.reduce::<26>()?,
                make_state!(154, DoubleColon) => parser.reduce::<26>()?,
                make_state!(154, SemiColon) => parser.reduce::<26>()?,
                make_state!(154, Eof) => parser.reduce::<26>()?,
                // State 155
                make_state!(155, Assign) => parser.reduce::<30>()?,
                make_state!(155, AttnamelistCont) => parser.goto(446),
                // State 176
                make_state!(176, Name) => parser.reduce::<48>()?,
                make_state!(176, String) => parser.reduce::<48>()?,
                make_state!(176, And) => parser.reduce::<48>()?,
                make_state!(176, Break) => parser.reduce::<48>()?,
                make_state!(176, Do) => parser.reduce::<48>()?,
                make_state!(176, For) => parser.reduce::<48>()?,
                make_state!(176, Function) => parser.reduce::<48>()?,
                make_state!(176, Goto) => parser.reduce::<48>()?,
                make_state!(176, If) => parser.reduce::<48>()?,
                make_state!(176, Local) => parser.reduce::<48>()?,
                make_state!(176, Or) => parser.reduce::<48>()?,
                make_state!(176, Repeat) => parser.reduce::<48>()?,
                make_state!(176, Return) => parser.reduce::<48>()?,
                make_state!(176, While) => parser.reduce::<48>()?,
                make_state!(176, Add) => parser.reduce::<48>()?,
                make_state!(176, Sub) => parser.reduce::<48>()?,
                make_state!(176, Mul) => parser.reduce::<48>()?,
                make_state!(176, Div) => parser.reduce::<48>()?,
                make_state!(176, Idiv) => parser.reduce::<48>()?,
                make_state!(176, Mod) => parser.reduce::<48>()?,
                make_state!(176, Pow) => parser.reduce::<48>()?,
                make_state!(176, BitAnd) => parser.reduce::<48>()?,
                make_state!(176, BitXor) => parser.reduce::<48>()?,
                make_state!(176, BitOr) => parser.reduce::<48>()?,
                make_state!(176, ShiftL) => parser.reduce::<48>()?,
                make_state!(176, ShiftR) => parser.reduce::<48>()?,
                make_state!(176, Eq) => parser.reduce::<48>()?,
                make_state!(176, Neq) => parser.reduce::<48>()?,
                make_state!(176, Leq) => parser.reduce::<48>()?,
                make_state!(176, Geq) => parser.reduce::<48>()?,
                make_state!(176, Less) => parser.reduce::<48>()?,
                make_state!(176, Greater) => parser.reduce::<48>()?,
                make_state!(176, LParen) => parser.reduce::<48>()?,
                make_state!(176, LSquare) => parser.reduce::<48>()?,
                make_state!(176, LCurly) => parser.reduce::<48>()?,
                make_state!(176, DoubleColon) => parser.reduce::<48>()?,
                make_state!(176, SemiColon) => parser.reduce::<48>()?,
                make_state!(176, Colon) => parser.reduce::<48>()?,
                make_state!(176, Comma) => parser.reduce::<48>()?,
                make_state!(176, Dot) => parser.reduce::<48>()?,
                make_state!(176, Concat) => parser.reduce::<48>()?,
                make_state!(176, Eof) => parser.reduce::<48>()?,
                // State 183
                make_state!(183, Name) => parser.reduce::<65>()?,
                make_state!(183, String) => parser.shift(663),
                make_state!(183, And) => parser.reduce::<65>()?,
                make_state!(183, Break) => parser.reduce::<65>()?,
                make_state!(183, Do) => parser.reduce::<65>()?,
                make_state!(183, For) => parser.reduce::<65>()?,
                make_state!(183, Function) => parser.reduce::<65>()?,
                make_state!(183, Goto) => parser.reduce::<65>()?,
                make_state!(183, If) => parser.reduce::<65>()?,
                make_state!(183, Local) => parser.reduce::<65>()?,
                make_state!(183, Or) => parser.reduce::<65>()?,
                make_state!(183, Repeat) => parser.reduce::<65>()?,
                make_state!(183, Return) => parser.reduce::<65>()?,
                make_state!(183, While) => parser.reduce::<65>()?,
                make_state!(183, Add) => parser.reduce::<65>()?,
                make_state!(183, Sub) => parser.reduce::<65>()?,
                make_state!(183, Mul) => parser.reduce::<65>()?,
                make_state!(183, Div) => parser.reduce::<65>()?,
                make_state!(183, Idiv) => parser.reduce::<65>()?,
                make_state!(183, Mod) => parser.reduce::<65>()?,
                make_state!(183, Pow) => parser.reduce::<65>()?,
                make_state!(183, BitAnd) => parser.reduce::<65>()?,
                make_state!(183, BitXor) => parser.reduce::<65>()?,
                make_state!(183, BitOr) => parser.reduce::<65>()?,
                make_state!(183, ShiftL) => parser.reduce::<65>()?,
                make_state!(183, ShiftR) => parser.reduce::<65>()?,
                make_state!(183, Eq) => parser.reduce::<65>()?,
                make_state!(183, Neq) => parser.reduce::<65>()?,
                make_state!(183, Leq) => parser.reduce::<65>()?,
                make_state!(183, Geq) => parser.reduce::<65>()?,
                make_state!(183, Less) => parser.reduce::<65>()?,
                make_state!(183, Greater) => parser.reduce::<65>()?,
                // Shift(665)-Reduce(65) manually resolved by shifting
                make_state!(183, LParen) => parser.shift(665),
                make_state!(183, LSquare) => parser.shift(508),
                make_state!(183, LCurly) => parser.shift(703),
                make_state!(183, DoubleColon) => parser.reduce::<65>()?,
                make_state!(183, SemiColon) => parser.reduce::<65>()?,
                make_state!(183, Colon) => parser.shift(661),
                make_state!(183, Comma) => parser.reduce::<65>()?,
                make_state!(183, Dot) => parser.shift(660),
                make_state!(183, Concat) => parser.reduce::<65>()?,
                make_state!(183, Eof) => parser.reduce::<65>()?,
                make_state!(183, Args) => parser.goto(662),
                make_state!(183, Tableconstructor) => parser.goto(701),
                // State 186
                make_state!(186, RParen) => parser.reduce::<48>()?,
                // State 192
                make_state!(192, RParen) => parser.reduce::<65>()?,
                // State 218
                make_state!(218, RParen) => parser.reduce::<55>()?,
                make_state!(218, ExplistCont) => parser.goto(498),
                // State 220
                make_state!(220, RParen) => parser.reduce::<61>()?,
                // State 221
                make_state!(221, RParen) => parser.reduce::<62>()?,
                // State 222
                make_state!(222, RParen) => parser.reduce::<58>()?,
                // State 223
                make_state!(223, RParen) => parser.reduce::<57>()?,
                // State 232
                make_state!(232, Name) => parser.reduce::<55>()?,
                make_state!(232, And) => parser.shift(645),
                make_state!(232, Break) => parser.reduce::<55>()?,
                make_state!(232, Do) => parser.reduce::<55>()?,
                make_state!(232, For) => parser.reduce::<55>()?,
                make_state!(232, Function) => parser.reduce::<55>()?,
                make_state!(232, Goto) => parser.reduce::<55>()?,
                make_state!(232, If) => parser.reduce::<55>()?,
                make_state!(232, Local) => parser.reduce::<55>()?,
                make_state!(232, Or) => parser.shift(718),
                make_state!(232, Repeat) => parser.reduce::<55>()?,
                make_state!(232, Return) => parser.reduce::<55>()?,
                make_state!(232, While) => parser.reduce::<55>()?,
                make_state!(232, Add) => parser.shift(836),
                make_state!(232, Sub) => parser.shift(869),
                make_state!(232, Mul) => parser.shift(892),
                make_state!(232, Div) => parser.shift(909),
                make_state!(232, Idiv) => parser.shift(927),
                make_state!(232, Mod) => parser.shift(943),
                make_state!(232, Pow) => parser.shift(946),
                make_state!(232, BitAnd) => parser.shift(947),
                make_state!(232, BitXor) => parser.shift(948),
                make_state!(232, BitOr) => parser.shift(949),
                make_state!(232, ShiftL) => parser.shift(950),
                make_state!(232, ShiftR) => parser.shift(951),
                make_state!(232, Eq) => parser.shift(952),
                make_state!(232, Neq) => parser.shift(953),
                make_state!(232, Leq) => parser.shift(954),
                make_state!(232, Geq) => parser.shift(955),
                make_state!(232, Less) => parser.shift(956),
                make_state!(232, Greater) => parser.shift(957),
                make_state!(232, LParen) => parser.reduce::<55>()?,
                make_state!(232, DoubleColon) => parser.reduce::<55>()?,
                make_state!(232, SemiColon) => parser.reduce::<55>()?,
                make_state!(232, Comma) => parser.shift(562),
                make_state!(232, Concat) => parser.shift(958),
                make_state!(232, Eof) => parser.reduce::<55>()?,
                make_state!(232, ExplistCont) => parser.goto(532),
                // State 237
                make_state!(237, Name) => parser.reduce::<60>()?,
                make_state!(237, And) => parser.reduce::<60>()?,
                make_state!(237, Break) => parser.reduce::<60>()?,
                make_state!(237, Do) => parser.reduce::<60>()?,
                make_state!(237, For) => parser.reduce::<60>()?,
                make_state!(237, Function) => parser.reduce::<60>()?,
                make_state!(237, Goto) => parser.reduce::<60>()?,
                make_state!(237, If) => parser.reduce::<60>()?,
                make_state!(237, Local) => parser.reduce::<60>()?,
                make_state!(237, Or) => parser.reduce::<60>()?,
                make_state!(237, Repeat) => parser.reduce::<60>()?,
                make_state!(237, Return) => parser.reduce::<60>()?,
                make_state!(237, While) => parser.reduce::<60>()?,
                make_state!(237, Add) => parser.reduce::<60>()?,
                make_state!(237, Sub) => parser.reduce::<60>()?,
                make_state!(237, Mul) => parser.reduce::<60>()?,
                make_state!(237, Div) => parser.reduce::<60>()?,
                make_state!(237, Idiv) => parser.reduce::<60>()?,
                make_state!(237, Mod) => parser.reduce::<60>()?,
                make_state!(237, Pow) => parser.reduce::<60>()?,
                make_state!(237, BitAnd) => parser.reduce::<60>()?,
                make_state!(237, BitXor) => parser.reduce::<60>()?,
                make_state!(237, BitOr) => parser.reduce::<60>()?,
                make_state!(237, ShiftL) => parser.reduce::<60>()?,
                make_state!(237, ShiftR) => parser.reduce::<60>()?,
                make_state!(237, Eq) => parser.reduce::<60>()?,
                make_state!(237, Neq) => parser.reduce::<60>()?,
                make_state!(237, Leq) => parser.reduce::<60>()?,
                make_state!(237, Geq) => parser.reduce::<60>()?,
                make_state!(237, Less) => parser.reduce::<60>()?,
                make_state!(237, Greater) => parser.reduce::<60>()?,
                make_state!(237, LParen) => parser.reduce::<60>()?,
                make_state!(237, DoubleColon) => parser.reduce::<60>()?,
                make_state!(237, SemiColon) => parser.reduce::<60>()?,
                make_state!(237, Comma) => parser.reduce::<60>()?,
                make_state!(237, Concat) => parser.reduce::<60>()?,
                make_state!(237, Eof) => parser.reduce::<60>()?,
                // State 242
                make_state!(242, Name) => parser.shift(176),
                make_state!(242, String) => parser.shift(237),
                make_state!(242, Integer) => parser.shift(238),
                make_state!(242, Float) => parser.shift(239),
                make_state!(242, False) => parser.shift(240),
                make_state!(242, Function) => parser.shift(428),
                make_state!(242, Nil) => parser.shift(241),
                make_state!(242, Not) => parser.shift(247),
                make_state!(242, True) => parser.shift(336),
                make_state!(242, Sub) => parser.shift(338),
                make_state!(242, Len) => parser.shift(393),
                make_state!(242, BitXor) => parser.shift(404),
                make_state!(242, LParen) => parser.shift(420),
                make_state!(242, LCurly) => parser.shift(430),
                make_state!(242, Dots) => parser.shift(414),
                make_state!(242, Var) => parser.goto(425),
                make_state!(242, Explist) => parser.goto(534),
                make_state!(242, Exp) => parser.goto(232),
                make_state!(242, Prefixexp) => parser.goto(183),
                make_state!(242, Functioncall) => parser.goto(426),
                make_state!(242, Functiondef) => parser.goto(418),
                make_state!(242, Tableconstructor) => parser.goto(419),
                // State 305
                make_state!(305, RParen) => parser.reduce::<59>()?,
                // State 367
                make_state!(367, RParen) => parser.reduce::<92>()?,
                // State 369
                make_state!(369, RParen) => parser.shift(599),
                // State 370
                make_state!(370, RParen) => parser.reduce::<99>()?,
                // State 425
                make_state!(425, Name) => parser.reduce::<92>()?,
                make_state!(425, String) => parser.reduce::<92>()?,
                make_state!(425, And) => parser.reduce::<92>()?,
                make_state!(425, Break) => parser.reduce::<92>()?,
                make_state!(425, Do) => parser.reduce::<92>()?,
                make_state!(425, For) => parser.reduce::<92>()?,
                make_state!(425, Function) => parser.reduce::<92>()?,
                make_state!(425, Goto) => parser.reduce::<92>()?,
                make_state!(425, If) => parser.reduce::<92>()?,
                make_state!(425, Local) => parser.reduce::<92>()?,
                make_state!(425, Or) => parser.reduce::<92>()?,
                make_state!(425, Repeat) => parser.reduce::<92>()?,
                make_state!(425, Return) => parser.reduce::<92>()?,
                make_state!(425, While) => parser.reduce::<92>()?,
                make_state!(425, Add) => parser.reduce::<92>()?,
                make_state!(425, Sub) => parser.reduce::<92>()?,
                make_state!(425, Mul) => parser.reduce::<92>()?,
                make_state!(425, Div) => parser.reduce::<92>()?,
                make_state!(425, Idiv) => parser.reduce::<92>()?,
                make_state!(425, Mod) => parser.reduce::<92>()?,
                make_state!(425, Pow) => parser.reduce::<92>()?,
                make_state!(425, BitAnd) => parser.reduce::<92>()?,
                make_state!(425, BitXor) => parser.reduce::<92>()?,
                make_state!(425, BitOr) => parser.reduce::<92>()?,
                make_state!(425, ShiftL) => parser.reduce::<92>()?,
                make_state!(425, ShiftR) => parser.reduce::<92>()?,
                make_state!(425, Eq) => parser.reduce::<92>()?,
                make_state!(425, Neq) => parser.reduce::<92>()?,
                make_state!(425, Leq) => parser.reduce::<92>()?,
                make_state!(425, Geq) => parser.reduce::<92>()?,
                make_state!(425, Less) => parser.reduce::<92>()?,
                make_state!(425, Greater) => parser.reduce::<92>()?,
                make_state!(425, LParen) => parser.reduce::<92>()?,
                make_state!(425, LSquare) => parser.reduce::<92>()?,
                make_state!(425, LCurly) => parser.reduce::<92>()?,
                make_state!(425, DoubleColon) => parser.reduce::<92>()?,
                make_state!(425, SemiColon) => parser.reduce::<92>()?,
                make_state!(425, Colon) => parser.reduce::<92>()?,
                make_state!(425, Comma) => parser.reduce::<92>()?,
                make_state!(425, Dot) => parser.reduce::<92>()?,
                make_state!(425, Concat) => parser.reduce::<92>()?,
                make_state!(425, Eof) => parser.reduce::<92>()?,
                // State 446
                make_state!(446, Assign) => parser.reduce::<29>()?,
                // State 498
                make_state!(498, RParen) => parser.reduce::<54>()?,
                // State 532
                make_state!(532, Name) => parser.reduce::<54>()?,
                make_state!(532, Break) => parser.reduce::<54>()?,
                make_state!(532, Do) => parser.reduce::<54>()?,
                make_state!(532, For) => parser.reduce::<54>()?,
                make_state!(532, Function) => parser.reduce::<54>()?,
                make_state!(532, Goto) => parser.reduce::<54>()?,
                make_state!(532, If) => parser.reduce::<54>()?,
                make_state!(532, Local) => parser.reduce::<54>()?,
                make_state!(532, Repeat) => parser.reduce::<54>()?,
                make_state!(532, Return) => parser.reduce::<54>()?,
                make_state!(532, While) => parser.reduce::<54>()?,
                make_state!(532, LParen) => parser.reduce::<54>()?,
                make_state!(532, DoubleColon) => parser.reduce::<54>()?,
                make_state!(532, SemiColon) => parser.reduce::<54>()?,
                make_state!(532, Eof) => parser.reduce::<54>()?,
                // State 534
                make_state!(534, Name) => parser.reduce::<28>()?,
                make_state!(534, Break) => parser.reduce::<28>()?,
                make_state!(534, Do) => parser.reduce::<28>()?,
                make_state!(534, For) => parser.reduce::<28>()?,
                make_state!(534, Function) => parser.reduce::<28>()?,
                make_state!(534, Goto) => parser.reduce::<28>()?,
                make_state!(534, If) => parser.reduce::<28>()?,
                make_state!(534, Local) => parser.reduce::<28>()?,
                make_state!(534, Repeat) => parser.reduce::<28>()?,
                make_state!(534, Return) => parser.reduce::<28>()?,
                make_state!(534, While) => parser.reduce::<28>()?,
                make_state!(534, LParen) => parser.reduce::<28>()?,
                make_state!(534, DoubleColon) => parser.reduce::<28>()?,
                make_state!(534, SemiColon) => parser.reduce::<28>()?,
                make_state!(534, Eof) => parser.reduce::<28>()?,
                // State 599
                make_state!(599, Name) => parser.reduce::<97>()?,
                make_state!(599, String) => parser.reduce::<97>()?,
                make_state!(599, Break) => parser.reduce::<97>()?,
                make_state!(599, Do) => parser.reduce::<97>()?,
                make_state!(599, For) => parser.reduce::<97>()?,
                make_state!(599, Function) => parser.reduce::<97>()?,
                make_state!(599, Goto) => parser.reduce::<97>()?,
                make_state!(599, If) => parser.reduce::<97>()?,
                make_state!(599, Local) => parser.reduce::<97>()?,
                make_state!(599, Repeat) => parser.reduce::<97>()?,
                make_state!(599, Return) => parser.reduce::<97>()?,
                make_state!(599, While) => parser.reduce::<97>()?,
                make_state!(599, LParen) => parser.reduce::<97>()?,
                make_state!(599, LSquare) => parser.reduce::<97>()?,
                make_state!(599, LCurly) => parser.reduce::<97>()?,
                make_state!(599, DoubleColon) => parser.reduce::<97>()?,
                make_state!(599, SemiColon) => parser.reduce::<97>()?,
                make_state!(599, Colon) => parser.reduce::<97>()?,
                make_state!(599, Dot) => parser.reduce::<97>()?,
                make_state!(599, Eof) => parser.reduce::<97>()?,
                // Errors
                _ => {
                    break Err(Error::Unimplemented);
                }
            }
        }
    }

    fn accept(mut self) -> Result<Token<'a>, Error> {
        let stack_pop = self.stack_pop(1);

        if !matches!(
            stack_pop.as_slice(),
            [Token {
                tokens: _,
                token_type: TokenType::Block
            }]
        ) {
            log::error!(
                "Failed to accept.\n\tExpected: Block\n\tGot: {:?}",
                stack_pop
                    .into_iter()
                    .map(|token| token.token_type)
                    .collect::<Vec<_>>()
            );
            Err(Error::Accept)
        } else {
            Ok(Token {
                tokens: stack_pop,
                token_type: TokenType::Chunk,
            })
        }
    }

    fn state(&mut self) -> (&[usize], Option<&Result<Token, crate::lex::Error>>) {
        (
            self.states.as_slice(),
            self.reduction
                .as_ref()
                .or_else(|| self.lexeme_stream.peek()),
        )
    }

    fn shift(&mut self, next_state: usize) {
        let Some(Ok(token)) = self.lexeme_stream.next() else {
            unreachable!();
        };
        self.states.push(next_state);
        self.stack.push(token);
    }

    fn goto(&mut self, next_state: usize) {
        let Some(Ok(token)) = self.reduction.take() else {
            unreachable!();
        };
        self.states.push(next_state);
        self.stack.push(token);
    }

    fn reduce<const PRODUCTION: usize>(&mut self) -> Result<(), Error> {
        match PRODUCTION {
            1 => make_reduction_push!(self, 2, Block, BlockStat, BlockRetstat),
            2 => make_reduction_push!(self, BlockStat),
            3 => make_reduction_push!(self, 2, BlockStat, Stat, BlockStat),
            4 => make_reduction_push!(self, BlockRetstat),
            5 => make_reduction_push!(self, 1, BlockRetstat, Retstat),
            6 => make_reduction_push!(self, 1, Stat, SemiColon),
            7 => make_reduction_push!(self, 3, Stat, Varlist, Assign, Explist),
            8 => make_reduction_push!(self, 1, Stat, Functioncall),
            9 => make_reduction_push!(self, 1, Stat, Label),
            10 => make_reduction_push!(self, 1, Stat, Break),
            11 => make_reduction_push!(self, 2, Stat, Break, Name),
            12 => make_reduction_push!(self, 3, Stat, Do, Block, End),
            13 => make_reduction_push!(self, 5, Stat, While, Exp, Do, Block, End),
            14 => make_reduction_push!(self, 4, Stat, Repeat, Block, Until, Exp),
            15 => {
                make_reduction_push!(self, 7, Stat, If, Exp, Then, Block, StatElseif, StatElse, End)
            }
            16 => make_reduction_push!(self, StatElseif),
            17 => make_reduction_push!(self, 5, StatElseif, Elseif, Exp, Then, Block, StatElseif),
            18 => make_reduction_push!(self, StatElse),
            19 => make_reduction_push!(self, 2, StatElse, Else, Block),
            20 => {
                make_reduction_push!(
                    self, 10, Stat, For, Name, Assign, Exp, Comma, Exp, StatForexp, Do, Block, End
                )
            }
            21 => make_reduction_push!(self, StatForexp),
            22 => make_reduction_push!(self, 2, StatForexp, Comma, Exp),
            23 => make_reduction_push!(self, 7, Stat, For, Namelist, In, Explist, Do, Block, End),
            24 => make_reduction_push!(self, 3, Stat, Function, Funcname, Funcbody),
            25 => make_reduction_push!(self, 4, Stat, Local, Function, Name, Funcbody),
            26 => make_reduction_push!(self, 3, Stat, Local, Attnamelist, StatAttexplist),
            27 => make_reduction_push!(self, StatAttexplist),
            28 => make_reduction_push!(self, 2, StatAttexplist, Assign, Explist),
            29 => make_reduction_push!(self, 3, Attnamelist, Name, Attrib, AttnamelistCont),
            30 => make_reduction_push!(self, AttnamelistCont),
            31 => {
                make_reduction_push!(
                    self,
                    4,
                    AttnamelistCont,
                    Comma,
                    Name,
                    Attrib,
                    AttnamelistCont
                )
            }
            32 => make_reduction_push!(self, Attrib),
            33 => make_reduction_push!(self, 3, Attrib, Less, Name, Greater),
            34 => make_reduction_push!(self, 3, Retstat, Return, RetstatExplist, RetstatEnd),
            35 => make_reduction_push!(self, RetstatExplist),
            36 => make_reduction_push!(self, 1, RetstatExplist, Explist),
            37 => make_reduction_push!(self, RetstatEnd),
            38 => make_reduction_push!(self, 1, RetstatEnd, SemiColon),
            39 => make_reduction_push!(self, 3, Label, DoubleColon, Name, DoubleColon),
            40 => make_reduction_push!(self, 3, Funcname, Name, FuncnameCont, FuncnameEnd),
            41 => make_reduction_push!(self, FuncnameCont),
            42 => make_reduction_push!(self, 3, FuncnameCont, Dot, Name, FuncnameCont),
            43 => make_reduction_push!(self, FuncnameEnd),
            44 => make_reduction_push!(self, 2, FuncnameEnd, Colon, Name),
            45 => make_reduction_push!(self, 2, Varlist, Var, VarlistCont),
            46 => make_reduction_push!(self, VarlistCont),
            47 => make_reduction_push!(self, 3, VarlistCont, Comma, Var, VarlistCont),
            48 => make_reduction_push!(self, 1, Var, Name),
            49 => make_reduction_push!(self, 4, Var, Prefixexp, LSquare, Exp, RSquare),
            50 => make_reduction_push!(self, 3, Var, Prefixexp, Dot, Name),
            51 => make_reduction_push!(self, 2, Namelist, Name, NamelistCont),
            52 => make_reduction_push!(self, NamelistCont),
            53 => make_reduction_push!(self, 3, NamelistCont, Comma, Name, NamelistCont),
            54 => make_reduction_push!(self, 2, Explist, Exp, ExplistCont),
            55 => make_reduction_push!(self, ExplistCont),
            56 => make_reduction_push!(self, 3, ExplistCont, Comma, Exp, ExplistCont),
            57 => make_reduction_push!(self, 1, Exp, Nil),
            58 => make_reduction_push!(self, 1, Exp, False),
            59 => make_reduction_push!(self, 1, Exp, True),
            60 => make_reduction_push!(self, 1, Exp, String),
            61 => make_reduction_push!(self, 1, Exp, Integer),
            62 => make_reduction_push!(self, 1, Exp, Float),
            63 => make_reduction_push!(self, 1, Exp, Dots),
            64 => make_reduction_push!(self, 1, Exp, Functiondef),
            65 => make_reduction_push!(self, 1, Exp, Prefixexp),
            66 => make_reduction_push!(self, 1, Exp, Tableconstructor),
            67 => make_reduction_push!(self, 3, Exp, Exp, Or, Exp),
            68 => make_reduction_push!(self, 3, Exp, Exp, And, Exp),
            69 => make_reduction_push!(self, 3, Exp, Exp, Less, Exp),
            70 => make_reduction_push!(self, 3, Exp, Exp, Greater, Exp),
            71 => make_reduction_push!(self, 3, Exp, Exp, Leq, Exp),
            72 => make_reduction_push!(self, 3, Exp, Exp, Geq, Exp),
            73 => make_reduction_push!(self, 3, Exp, Exp, Eq, Exp),
            74 => make_reduction_push!(self, 3, Exp, Exp, Neq, Exp),
            75 => make_reduction_push!(self, 3, Exp, Exp, BitOr, Exp),
            76 => make_reduction_push!(self, 3, Exp, Exp, BitXor, Exp),
            77 => make_reduction_push!(self, 3, Exp, Exp, BitAnd, Exp),
            78 => make_reduction_push!(self, 3, Exp, Exp, ShiftL, Exp),
            79 => make_reduction_push!(self, 3, Exp, Exp, ShiftR, Exp),
            80 => make_reduction_push!(self, 3, Exp, Exp, Concat, Exp),
            81 => make_reduction_push!(self, 3, Exp, Exp, Add, Exp),
            82 => make_reduction_push!(self, 3, Exp, Exp, Sub, Exp),
            83 => make_reduction_push!(self, 3, Exp, Exp, Mul, Exp),
            84 => make_reduction_push!(self, 3, Exp, Exp, Div, Exp),
            85 => make_reduction_push!(self, 3, Exp, Exp, Idiv, Exp),
            86 => make_reduction_push!(self, 3, Exp, Exp, Mod, Exp),
            87 => make_reduction_push!(self, 2, Exp, Not, Exp),
            88 => make_reduction_push!(self, 2, Exp, Len, Exp),
            89 => make_reduction_push!(self, 2, Exp, Sub, Exp),
            90 => make_reduction_push!(self, 2, Exp, BitXor, Exp),
            91 => make_reduction_push!(self, 3, Exp, Exp, Pow, Exp),
            92 => make_reduction_push!(self, 1, Prefixexp, Var),
            93 => make_reduction_push!(self, 1, Prefixexp, Functioncall),
            94 => make_reduction_push!(self, 3, Prefixexp, LParen, Exp, RParen),
            95 => make_reduction_push!(self, 2, Functioncall, Prefixexp, Args),
            96 => make_reduction_push!(self, 4, Prefixexp, Prefixexp, Colon, Name, Args),
            97 => make_reduction_push!(self, 3, Args, LParen, ArgsExplist, RParen),
            98 => make_reduction_push!(self, ArgsExplist),
            99 => make_reduction_push!(self, 1, ArgsExplist, Explist),
            100 => make_reduction_push!(self, 1, Args, Tableconstructor),
            101 => make_reduction_push!(self, 1, Args, String),
            102 => make_reduction_push!(self, 2, Functiondef, Function, Funcbody),
            103 => {
                make_reduction_push!(
                    self,
                    5,
                    Funcbody,
                    LParen,
                    FuncbodyParlist,
                    RParen,
                    Block,
                    End
                )
            }
            104 => make_reduction_push!(self, FuncbodyParlist),
            105 => make_reduction_push!(self, 1, FuncbodyParlist, Parlist),
            106 => make_reduction_push!(self, 2, Parlist, Namelist, ParlistCont),
            107 => make_reduction_push!(self, ParlistCont),
            108 => make_reduction_push!(self, 2, ParlistCont, Comma, Dots),
            109 => make_reduction_push!(self, 1, Parlist, Dots),
            110 => {
                make_reduction_push!(
                    self,
                    3,
                    Tableconstructor,
                    LCurly,
                    TableconstructorFieldlist,
                    RCurly
                )
            }
            111 => make_reduction_push!(self, TableconstructorFieldlist),
            112 => make_reduction_push!(self, 1, TableconstructorFieldlist, Fieldlist),
            113 => make_reduction_push!(self, 2, Fieldlist, Field, FieldlistCont),
            114 => make_reduction_push!(self, FieldlistCont),
            115 => make_reduction_push!(self, 3, FieldlistCont, Fieldsep, Field, FieldlistCont),
            116 => make_reduction_push!(self, 1, FieldlistCont, Fieldsep),
            117 => make_reduction_push!(self, 5, Field, LSquare, Exp, RSquare, Assign, Exp),
            118 => make_reduction_push!(self, 3, Field, Name, Assign, Exp),
            119 => make_reduction_push!(self, 1, Field, Exp),
            120 => make_reduction_push!(self, 1, Fieldsep, Comma),
            121 => make_reduction_push!(self, 1, Fieldsep, SemiColon),
            _ => {
                unreachable!()
            }
        }
    }

    fn stack_pop(&mut self, count: usize) -> Vec<Token<'a>> {
        (0..count)
            .map(|_| {
                let Some(top) = self.stack.pop() else {
                    unreachable!("Stack shouldn't be empty.");
                };
                let Some(_) = self.states.pop() else {
                    unreachable!("States shouldn't be empty.");
                };
                top
            })
            .collect()
    }
}
