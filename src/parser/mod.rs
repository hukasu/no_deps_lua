mod error;
mod state;
mod token;

use core::iter::Peekable;

use alloc::vec::Vec;

use crate::lex::{Lex, Lexeme};

use self::state::{State, StateProcessor};
pub use self::{
    error::Error,
    token::{Token, TokenType},
};

type LexemeResToTokenRes =
    &'static dyn Fn(Result<Lexeme, crate::lex::Error>) -> Result<Token, crate::lex::Error>;

macro_rules! make_state {
    (0, $lookahead:ident) => {
        (
            Some(0),
            Some(Ok(Token {
                tokens: _,
                token_type: $lookahead,
            })),
        )
    };
    (0, $lookahead:pat) => {
        (
            Some(0),
            Some(Ok(Token {
                tokens: _,
                token_type: $lookahead,
            })),
        )
    };
    ($cur_state:expr, $lookahead:ident) => {
        (
            Some($cur_state),
            Some(Ok(Token {
                tokens: _,
                token_type: $lookahead,
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
    ($reduction:expr, $parser:ident, $token_type:ident) => {
        {
            $parser.reduction.replace(Ok(Token {
                tokens: [].to_vec(),
                token_type: TokenType::$token_type,
            }));
            Ok(())
        }
    };
    (
        $reduction:expr,
        $parser:ident,
        $token_type:ident,
        $count:expr,
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
                    "Failed to reduce with rule {}.\n\tExpected: {:?}\n\tGot: {:?}",
                    $reduction,
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
    #[allow(clippy::too_many_lines)]
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
            let last_state = parser.states.last().copied();
            let token_peek = parser
                .reduction
                .as_ref()
                .or_else(|| parser.lexeme_stream.peek())
                .cloned();
            match (last_state, token_peek) {
                (_, Some(Err(err))) => {
                    log::error!("Failed to parse due to a lexical error. {}", err);
                    Err(Error::Lex)
                }
                make_state!(0, TokenType::Chunk) => break,
                make_state!(0, lookahead) => State::<0>::process_state(&mut parser, lookahead),
                make_state!(1, lookahead) => State::<1>::process_state(&mut parser, lookahead),
                make_state!(2, lookahead) => State::<2>::process_state(&mut parser, lookahead),
                make_state!(3, lookahead) => State::<3>::process_state(&mut parser, lookahead),
                make_state!(4, lookahead) => State::<4>::process_state(&mut parser, lookahead),
                make_state!(5, lookahead) => State::<5>::process_state(&mut parser, lookahead),
                make_state!(6, lookahead) => State::<6>::process_state(&mut parser, lookahead),
                make_state!(7, lookahead) => State::<7>::process_state(&mut parser, lookahead),
                make_state!(8, lookahead) => State::<8>::process_state(&mut parser, lookahead),
                make_state!(9, lookahead) => State::<9>::process_state(&mut parser, lookahead),
                make_state!(10, lookahead) => State::<10>::process_state(&mut parser, lookahead),
                make_state!(11, lookahead) => State::<11>::process_state(&mut parser, lookahead),
                make_state!(12, lookahead) => State::<12>::process_state(&mut parser, lookahead),
                make_state!(13, lookahead) => State::<13>::process_state(&mut parser, lookahead),
                make_state!(14, lookahead) => State::<14>::process_state(&mut parser, lookahead),
                make_state!(15, lookahead) => State::<15>::process_state(&mut parser, lookahead),
                make_state!(16, lookahead) => State::<16>::process_state(&mut parser, lookahead),
                make_state!(17, lookahead) => State::<17>::process_state(&mut parser, lookahead),
                make_state!(18, lookahead) => State::<18>::process_state(&mut parser, lookahead),
                make_state!(19, lookahead) => State::<19>::process_state(&mut parser, lookahead),
                make_state!(20, lookahead) => State::<20>::process_state(&mut parser, lookahead),
                make_state!(21, lookahead) => State::<21>::process_state(&mut parser, lookahead),
                make_state!(22, lookahead) => State::<22>::process_state(&mut parser, lookahead),
                make_state!(23, lookahead) => State::<23>::process_state(&mut parser, lookahead),
                make_state!(24, lookahead) => State::<24>::process_state(&mut parser, lookahead),
                make_state!(25, lookahead) => State::<25>::process_state(&mut parser, lookahead),
                make_state!(26, lookahead) => State::<26>::process_state(&mut parser, lookahead),
                make_state!(27, lookahead) => State::<27>::process_state(&mut parser, lookahead),
                make_state!(28, lookahead) => State::<28>::process_state(&mut parser, lookahead),
                make_state!(29, lookahead) => State::<29>::process_state(&mut parser, lookahead),
                make_state!(30, lookahead) => State::<30>::process_state(&mut parser, lookahead),
                make_state!(31, lookahead) => State::<31>::process_state(&mut parser, lookahead),
                make_state!(32, lookahead) => State::<32>::process_state(&mut parser, lookahead),
                make_state!(33, lookahead) => State::<33>::process_state(&mut parser, lookahead),
                make_state!(34, lookahead) => State::<34>::process_state(&mut parser, lookahead),
                make_state!(35, lookahead) => State::<35>::process_state(&mut parser, lookahead),
                make_state!(36, lookahead) => State::<36>::process_state(&mut parser, lookahead),
                make_state!(37, lookahead) => State::<37>::process_state(&mut parser, lookahead),
                make_state!(38, lookahead) => State::<38>::process_state(&mut parser, lookahead),
                make_state!(39, lookahead) => State::<39>::process_state(&mut parser, lookahead),
                make_state!(40, lookahead) => State::<40>::process_state(&mut parser, lookahead),
                make_state!(41, lookahead) => State::<41>::process_state(&mut parser, lookahead),
                make_state!(42, lookahead) => State::<42>::process_state(&mut parser, lookahead),
                make_state!(43, lookahead) => State::<43>::process_state(&mut parser, lookahead),
                make_state!(44, lookahead) => State::<44>::process_state(&mut parser, lookahead),
                make_state!(45, lookahead) => State::<45>::process_state(&mut parser, lookahead),
                make_state!(46, lookahead) => State::<46>::process_state(&mut parser, lookahead),
                make_state!(47, lookahead) => State::<47>::process_state(&mut parser, lookahead),
                make_state!(48, lookahead) => State::<48>::process_state(&mut parser, lookahead),
                make_state!(49, lookahead) => State::<49>::process_state(&mut parser, lookahead),
                make_state!(50, lookahead) => State::<50>::process_state(&mut parser, lookahead),
                make_state!(51, lookahead) => State::<51>::process_state(&mut parser, lookahead),
                make_state!(52, lookahead) => State::<52>::process_state(&mut parser, lookahead),
                make_state!(53, lookahead) => State::<53>::process_state(&mut parser, lookahead),
                make_state!(54, lookahead) => State::<54>::process_state(&mut parser, lookahead),
                make_state!(55, lookahead) => State::<55>::process_state(&mut parser, lookahead),
                make_state!(56, lookahead) => State::<56>::process_state(&mut parser, lookahead),
                make_state!(57, lookahead) => State::<57>::process_state(&mut parser, lookahead),
                make_state!(58, lookahead) => State::<58>::process_state(&mut parser, lookahead),
                make_state!(59, lookahead) => State::<59>::process_state(&mut parser, lookahead),
                make_state!(60, lookahead) => State::<60>::process_state(&mut parser, lookahead),
                make_state!(61, lookahead) => State::<61>::process_state(&mut parser, lookahead),
                make_state!(62, lookahead) => State::<62>::process_state(&mut parser, lookahead),
                make_state!(63, lookahead) => State::<63>::process_state(&mut parser, lookahead),
                make_state!(64, lookahead) => State::<64>::process_state(&mut parser, lookahead),
                make_state!(65, lookahead) => State::<65>::process_state(&mut parser, lookahead),
                make_state!(66, lookahead) => State::<66>::process_state(&mut parser, lookahead),
                make_state!(67, lookahead) => State::<67>::process_state(&mut parser, lookahead),
                make_state!(68, lookahead) => State::<68>::process_state(&mut parser, lookahead),
                make_state!(69, lookahead) => State::<69>::process_state(&mut parser, lookahead),
                make_state!(70, lookahead) => State::<70>::process_state(&mut parser, lookahead),
                make_state!(71, lookahead) => State::<71>::process_state(&mut parser, lookahead),
                make_state!(72, lookahead) => State::<72>::process_state(&mut parser, lookahead),
                make_state!(73, lookahead) => State::<73>::process_state(&mut parser, lookahead),
                make_state!(74, lookahead) => State::<74>::process_state(&mut parser, lookahead),
                make_state!(75, lookahead) => State::<75>::process_state(&mut parser, lookahead),
                make_state!(76, lookahead) => State::<76>::process_state(&mut parser, lookahead),
                make_state!(77, lookahead) => State::<77>::process_state(&mut parser, lookahead),
                make_state!(78, lookahead) => State::<78>::process_state(&mut parser, lookahead),
                make_state!(79, lookahead) => State::<79>::process_state(&mut parser, lookahead),
                make_state!(80, lookahead) => State::<80>::process_state(&mut parser, lookahead),
                make_state!(81, lookahead) => State::<81>::process_state(&mut parser, lookahead),
                make_state!(82, lookahead) => State::<82>::process_state(&mut parser, lookahead),
                make_state!(83, lookahead) => State::<83>::process_state(&mut parser, lookahead),
                make_state!(84, lookahead) => State::<84>::process_state(&mut parser, lookahead),
                make_state!(85, lookahead) => State::<85>::process_state(&mut parser, lookahead),
                make_state!(86, lookahead) => State::<86>::process_state(&mut parser, lookahead),
                make_state!(87, lookahead) => State::<87>::process_state(&mut parser, lookahead),
                make_state!(88, lookahead) => State::<88>::process_state(&mut parser, lookahead),
                make_state!(89, lookahead) => State::<89>::process_state(&mut parser, lookahead),
                make_state!(90, lookahead) => State::<90>::process_state(&mut parser, lookahead),
                make_state!(91, lookahead) => State::<91>::process_state(&mut parser, lookahead),
                make_state!(92, lookahead) => State::<92>::process_state(&mut parser, lookahead),
                make_state!(93, lookahead) => State::<93>::process_state(&mut parser, lookahead),
                make_state!(94, lookahead) => State::<94>::process_state(&mut parser, lookahead),
                make_state!(95, lookahead) => State::<95>::process_state(&mut parser, lookahead),
                make_state!(96, lookahead) => State::<96>::process_state(&mut parser, lookahead),
                make_state!(97, lookahead) => State::<97>::process_state(&mut parser, lookahead),
                make_state!(98, lookahead) => State::<98>::process_state(&mut parser, lookahead),
                make_state!(99, lookahead) => State::<99>::process_state(&mut parser, lookahead),
                make_state!(100, lookahead) => State::<100>::process_state(&mut parser, lookahead),
                make_state!(101, lookahead) => State::<101>::process_state(&mut parser, lookahead),
                make_state!(102, lookahead) => State::<102>::process_state(&mut parser, lookahead),
                make_state!(103, lookahead) => State::<103>::process_state(&mut parser, lookahead),
                make_state!(104, lookahead) => State::<104>::process_state(&mut parser, lookahead),
                make_state!(105, lookahead) => State::<105>::process_state(&mut parser, lookahead),
                make_state!(106, lookahead) => State::<106>::process_state(&mut parser, lookahead),
                make_state!(107, lookahead) => State::<107>::process_state(&mut parser, lookahead),
                make_state!(108, lookahead) => State::<108>::process_state(&mut parser, lookahead),
                make_state!(109, lookahead) => State::<109>::process_state(&mut parser, lookahead),
                make_state!(110, lookahead) => State::<110>::process_state(&mut parser, lookahead),
                make_state!(111, lookahead) => State::<111>::process_state(&mut parser, lookahead),
                make_state!(112, lookahead) => State::<112>::process_state(&mut parser, lookahead),
                make_state!(113, lookahead) => State::<113>::process_state(&mut parser, lookahead),
                make_state!(114, lookahead) => State::<114>::process_state(&mut parser, lookahead),
                make_state!(115, lookahead) => State::<115>::process_state(&mut parser, lookahead),
                make_state!(116, lookahead) => State::<116>::process_state(&mut parser, lookahead),
                make_state!(117, lookahead) => State::<117>::process_state(&mut parser, lookahead),
                make_state!(118, lookahead) => State::<118>::process_state(&mut parser, lookahead),
                make_state!(119, lookahead) => State::<119>::process_state(&mut parser, lookahead),
                make_state!(120, lookahead) => State::<120>::process_state(&mut parser, lookahead),
                make_state!(121, lookahead) => State::<121>::process_state(&mut parser, lookahead),
                make_state!(122, lookahead) => State::<122>::process_state(&mut parser, lookahead),
                make_state!(123, lookahead) => State::<123>::process_state(&mut parser, lookahead),
                make_state!(124, lookahead) => State::<124>::process_state(&mut parser, lookahead),
                make_state!(125, lookahead) => State::<125>::process_state(&mut parser, lookahead),
                make_state!(126, lookahead) => State::<126>::process_state(&mut parser, lookahead),
                make_state!(127, lookahead) => State::<127>::process_state(&mut parser, lookahead),
                make_state!(128, lookahead) => State::<128>::process_state(&mut parser, lookahead),
                make_state!(129, lookahead) => State::<129>::process_state(&mut parser, lookahead),
                make_state!(130, lookahead) => State::<130>::process_state(&mut parser, lookahead),
                make_state!(131, lookahead) => State::<131>::process_state(&mut parser, lookahead),
                make_state!(132, lookahead) => State::<132>::process_state(&mut parser, lookahead),
                make_state!(133, lookahead) => State::<133>::process_state(&mut parser, lookahead),
                make_state!(134, lookahead) => State::<134>::process_state(&mut parser, lookahead),
                make_state!(135, lookahead) => State::<135>::process_state(&mut parser, lookahead),
                make_state!(136, lookahead) => State::<136>::process_state(&mut parser, lookahead),
                make_state!(137, lookahead) => State::<137>::process_state(&mut parser, lookahead),
                make_state!(138, lookahead) => State::<138>::process_state(&mut parser, lookahead),
                make_state!(139, lookahead) => State::<139>::process_state(&mut parser, lookahead),
                make_state!(140, lookahead) => State::<140>::process_state(&mut parser, lookahead),
                make_state!(141, lookahead) => State::<141>::process_state(&mut parser, lookahead),
                make_state!(142, lookahead) => State::<142>::process_state(&mut parser, lookahead),
                make_state!(143, lookahead) => State::<143>::process_state(&mut parser, lookahead),
                make_state!(144, lookahead) => State::<144>::process_state(&mut parser, lookahead),
                make_state!(145, lookahead) => State::<145>::process_state(&mut parser, lookahead),
                make_state!(146, lookahead) => State::<146>::process_state(&mut parser, lookahead),
                make_state!(147, lookahead) => State::<147>::process_state(&mut parser, lookahead),
                make_state!(148, lookahead) => State::<148>::process_state(&mut parser, lookahead),
                make_state!(149, lookahead) => State::<149>::process_state(&mut parser, lookahead),
                make_state!(150, lookahead) => State::<150>::process_state(&mut parser, lookahead),
                make_state!(151, lookahead) => State::<151>::process_state(&mut parser, lookahead),
                make_state!(152, lookahead) => State::<152>::process_state(&mut parser, lookahead),
                make_state!(153, lookahead) => State::<153>::process_state(&mut parser, lookahead),
                make_state!(154, lookahead) => State::<154>::process_state(&mut parser, lookahead),
                make_state!(155, lookahead) => State::<155>::process_state(&mut parser, lookahead),
                make_state!(156, lookahead) => State::<156>::process_state(&mut parser, lookahead),
                make_state!(157, lookahead) => State::<157>::process_state(&mut parser, lookahead),
                make_state!(158, lookahead) => State::<158>::process_state(&mut parser, lookahead),
                make_state!(159, lookahead) => State::<159>::process_state(&mut parser, lookahead),
                make_state!(160, lookahead) => State::<160>::process_state(&mut parser, lookahead),
                make_state!(161, lookahead) => State::<161>::process_state(&mut parser, lookahead),
                make_state!(162, lookahead) => State::<162>::process_state(&mut parser, lookahead),
                make_state!(163, lookahead) => State::<163>::process_state(&mut parser, lookahead),
                make_state!(164, lookahead) => State::<164>::process_state(&mut parser, lookahead),
                make_state!(165, lookahead) => State::<165>::process_state(&mut parser, lookahead),
                make_state!(166, lookahead) => State::<166>::process_state(&mut parser, lookahead),
                make_state!(167, lookahead) => State::<167>::process_state(&mut parser, lookahead),
                make_state!(168, lookahead) => State::<168>::process_state(&mut parser, lookahead),
                make_state!(169, lookahead) => State::<169>::process_state(&mut parser, lookahead),
                make_state!(170, lookahead) => State::<170>::process_state(&mut parser, lookahead),
                make_state!(171, lookahead) => State::<171>::process_state(&mut parser, lookahead),
                make_state!(172, lookahead) => State::<172>::process_state(&mut parser, lookahead),
                make_state!(173, lookahead) => State::<173>::process_state(&mut parser, lookahead),
                make_state!(174, lookahead) => State::<174>::process_state(&mut parser, lookahead),
                make_state!(175, lookahead) => State::<175>::process_state(&mut parser, lookahead),
                make_state!(176, lookahead) => State::<176>::process_state(&mut parser, lookahead),
                make_state!(177, lookahead) => State::<177>::process_state(&mut parser, lookahead),
                make_state!(178, lookahead) => State::<178>::process_state(&mut parser, lookahead),
                make_state!(179, lookahead) => State::<179>::process_state(&mut parser, lookahead),
                make_state!(180, lookahead) => State::<180>::process_state(&mut parser, lookahead),
                make_state!(181, lookahead) => State::<181>::process_state(&mut parser, lookahead),
                make_state!(182, lookahead) => State::<182>::process_state(&mut parser, lookahead),
                make_state!(183, lookahead) => State::<183>::process_state(&mut parser, lookahead),
                make_state!(184, lookahead) => State::<184>::process_state(&mut parser, lookahead),
                make_state!(185, lookahead) => State::<185>::process_state(&mut parser, lookahead),
                make_state!(186, lookahead) => State::<186>::process_state(&mut parser, lookahead),
                make_state!(187, lookahead) => State::<187>::process_state(&mut parser, lookahead),
                make_state!(188, lookahead) => State::<188>::process_state(&mut parser, lookahead),
                make_state!(189, lookahead) => State::<189>::process_state(&mut parser, lookahead),
                make_state!(190, lookahead) => State::<190>::process_state(&mut parser, lookahead),
                make_state!(191, lookahead) => State::<191>::process_state(&mut parser, lookahead),
                make_state!(192, lookahead) => State::<192>::process_state(&mut parser, lookahead),
                make_state!(193, lookahead) => State::<193>::process_state(&mut parser, lookahead),
                make_state!(194, lookahead) => State::<194>::process_state(&mut parser, lookahead),
                make_state!(195, lookahead) => State::<195>::process_state(&mut parser, lookahead),
                make_state!(196, lookahead) => State::<196>::process_state(&mut parser, lookahead),
                make_state!(197, lookahead) => State::<197>::process_state(&mut parser, lookahead),
                make_state!(198, lookahead) => State::<198>::process_state(&mut parser, lookahead),
                make_state!(199, lookahead) => State::<199>::process_state(&mut parser, lookahead),
                make_state!(200, lookahead) => State::<200>::process_state(&mut parser, lookahead),
                make_state!(201, lookahead) => State::<201>::process_state(&mut parser, lookahead),
                make_state!(202, lookahead) => State::<202>::process_state(&mut parser, lookahead),
                make_state!(203, lookahead) => State::<203>::process_state(&mut parser, lookahead),
                make_state!(204, lookahead) => State::<204>::process_state(&mut parser, lookahead),
                make_state!(205, lookahead) => State::<205>::process_state(&mut parser, lookahead),
                make_state!(206, lookahead) => State::<206>::process_state(&mut parser, lookahead),
                make_state!(207, lookahead) => State::<207>::process_state(&mut parser, lookahead),
                make_state!(208, lookahead) => State::<208>::process_state(&mut parser, lookahead),
                make_state!(209, lookahead) => State::<209>::process_state(&mut parser, lookahead),
                make_state!(210, lookahead) => State::<210>::process_state(&mut parser, lookahead),
                make_state!(211, lookahead) => State::<211>::process_state(&mut parser, lookahead),
                make_state!(212, lookahead) => State::<212>::process_state(&mut parser, lookahead),
                make_state!(213, lookahead) => State::<213>::process_state(&mut parser, lookahead),
                make_state!(214, lookahead) => State::<214>::process_state(&mut parser, lookahead),
                make_state!(215, lookahead) => State::<215>::process_state(&mut parser, lookahead),
                make_state!(216, lookahead) => State::<216>::process_state(&mut parser, lookahead),
                make_state!(217, lookahead) => State::<217>::process_state(&mut parser, lookahead),
                make_state!(218, lookahead) => State::<218>::process_state(&mut parser, lookahead),
                make_state!(219, lookahead) => State::<219>::process_state(&mut parser, lookahead),
                make_state!(220, lookahead) => State::<220>::process_state(&mut parser, lookahead),
                make_state!(221, lookahead) => State::<221>::process_state(&mut parser, lookahead),
                make_state!(222, lookahead) => State::<222>::process_state(&mut parser, lookahead),
                make_state!(223, lookahead) => State::<223>::process_state(&mut parser, lookahead),
                make_state!(224, lookahead) => State::<224>::process_state(&mut parser, lookahead),
                make_state!(225, lookahead) => State::<225>::process_state(&mut parser, lookahead),
                make_state!(226, lookahead) => State::<226>::process_state(&mut parser, lookahead),
                make_state!(227, lookahead) => State::<227>::process_state(&mut parser, lookahead),
                make_state!(228, lookahead) => State::<228>::process_state(&mut parser, lookahead),
                make_state!(229, lookahead) => State::<229>::process_state(&mut parser, lookahead),
                make_state!(230, lookahead) => State::<230>::process_state(&mut parser, lookahead),
                make_state!(231, lookahead) => State::<231>::process_state(&mut parser, lookahead),
                make_state!(232, lookahead) => State::<232>::process_state(&mut parser, lookahead),
                make_state!(233, lookahead) => State::<233>::process_state(&mut parser, lookahead),
                make_state!(234, lookahead) => State::<234>::process_state(&mut parser, lookahead),
                make_state!(235, lookahead) => State::<235>::process_state(&mut parser, lookahead),
                make_state!(236, lookahead) => State::<236>::process_state(&mut parser, lookahead),
                make_state!(237, lookahead) => State::<237>::process_state(&mut parser, lookahead),
                make_state!(238, lookahead) => State::<238>::process_state(&mut parser, lookahead),
                make_state!(239, lookahead) => State::<239>::process_state(&mut parser, lookahead),
                make_state!(240, lookahead) => State::<240>::process_state(&mut parser, lookahead),
                make_state!(241, lookahead) => State::<241>::process_state(&mut parser, lookahead),
                make_state!(242, lookahead) => State::<242>::process_state(&mut parser, lookahead),
                make_state!(243, lookahead) => State::<243>::process_state(&mut parser, lookahead),
                make_state!(244, lookahead) => State::<244>::process_state(&mut parser, lookahead),
                make_state!(245, lookahead) => State::<245>::process_state(&mut parser, lookahead),
                make_state!(246, lookahead) => State::<246>::process_state(&mut parser, lookahead),
                make_state!(247, lookahead) => State::<247>::process_state(&mut parser, lookahead),
                make_state!(248, lookahead) => State::<248>::process_state(&mut parser, lookahead),
                make_state!(249, lookahead) => State::<249>::process_state(&mut parser, lookahead),
                make_state!(250, lookahead) => State::<250>::process_state(&mut parser, lookahead),
                make_state!(251, lookahead) => State::<251>::process_state(&mut parser, lookahead),
                make_state!(252, lookahead) => State::<252>::process_state(&mut parser, lookahead),
                make_state!(253, lookahead) => State::<253>::process_state(&mut parser, lookahead),
                make_state!(254, lookahead) => State::<254>::process_state(&mut parser, lookahead),
                make_state!(255, lookahead) => State::<255>::process_state(&mut parser, lookahead),
                make_state!(256, lookahead) => State::<256>::process_state(&mut parser, lookahead),
                make_state!(257, lookahead) => State::<257>::process_state(&mut parser, lookahead),
                make_state!(258, lookahead) => State::<258>::process_state(&mut parser, lookahead),
                make_state!(259, lookahead) => State::<259>::process_state(&mut parser, lookahead),
                make_state!(260, lookahead) => State::<260>::process_state(&mut parser, lookahead),
                make_state!(261, lookahead) => State::<261>::process_state(&mut parser, lookahead),
                make_state!(262, lookahead) => State::<262>::process_state(&mut parser, lookahead),
                make_state!(263, lookahead) => State::<263>::process_state(&mut parser, lookahead),
                make_state!(264, lookahead) => State::<264>::process_state(&mut parser, lookahead),
                make_state!(265, lookahead) => State::<265>::process_state(&mut parser, lookahead),
                make_state!(266, lookahead) => State::<266>::process_state(&mut parser, lookahead),
                make_state!(267, lookahead) => State::<267>::process_state(&mut parser, lookahead),
                make_state!(268, lookahead) => State::<268>::process_state(&mut parser, lookahead),
                make_state!(269, lookahead) => State::<269>::process_state(&mut parser, lookahead),
                make_state!(270, lookahead) => State::<270>::process_state(&mut parser, lookahead),
                make_state!(271, lookahead) => State::<271>::process_state(&mut parser, lookahead),
                make_state!(272, lookahead) => State::<272>::process_state(&mut parser, lookahead),
                make_state!(273, lookahead) => State::<273>::process_state(&mut parser, lookahead),
                make_state!(274, lookahead) => State::<274>::process_state(&mut parser, lookahead),
                make_state!(275, lookahead) => State::<275>::process_state(&mut parser, lookahead),
                make_state!(276, lookahead) => State::<276>::process_state(&mut parser, lookahead),
                make_state!(277, lookahead) => State::<277>::process_state(&mut parser, lookahead),
                make_state!(278, lookahead) => State::<278>::process_state(&mut parser, lookahead),
                make_state!(279, lookahead) => State::<279>::process_state(&mut parser, lookahead),
                make_state!(280, lookahead) => State::<280>::process_state(&mut parser, lookahead),
                make_state!(281, lookahead) => State::<281>::process_state(&mut parser, lookahead),
                make_state!(282, lookahead) => State::<282>::process_state(&mut parser, lookahead),
                make_state!(283, lookahead) => State::<283>::process_state(&mut parser, lookahead),
                make_state!(284, lookahead) => State::<284>::process_state(&mut parser, lookahead),
                make_state!(285, lookahead) => State::<285>::process_state(&mut parser, lookahead),
                make_state!(286, lookahead) => State::<286>::process_state(&mut parser, lookahead),
                make_state!(287, lookahead) => State::<287>::process_state(&mut parser, lookahead),
                make_state!(288, lookahead) => State::<288>::process_state(&mut parser, lookahead),
                make_state!(289, lookahead) => State::<289>::process_state(&mut parser, lookahead),
                make_state!(290, lookahead) => State::<290>::process_state(&mut parser, lookahead),
                make_state!(291, lookahead) => State::<291>::process_state(&mut parser, lookahead),
                make_state!(292, lookahead) => State::<292>::process_state(&mut parser, lookahead),
                make_state!(293, lookahead) => State::<293>::process_state(&mut parser, lookahead),
                make_state!(294, lookahead) => State::<294>::process_state(&mut parser, lookahead),
                make_state!(295, lookahead) => State::<295>::process_state(&mut parser, lookahead),
                make_state!(296, lookahead) => State::<296>::process_state(&mut parser, lookahead),
                make_state!(297, lookahead) => State::<297>::process_state(&mut parser, lookahead),
                make_state!(298, lookahead) => State::<298>::process_state(&mut parser, lookahead),
                make_state!(299, lookahead) => State::<299>::process_state(&mut parser, lookahead),
                make_state!(300, lookahead) => State::<300>::process_state(&mut parser, lookahead),
                make_state!(301, lookahead) => State::<301>::process_state(&mut parser, lookahead),
                make_state!(302, lookahead) => State::<302>::process_state(&mut parser, lookahead),
                make_state!(303, lookahead) => State::<303>::process_state(&mut parser, lookahead),
                make_state!(304, lookahead) => State::<304>::process_state(&mut parser, lookahead),
                make_state!(305, lookahead) => State::<305>::process_state(&mut parser, lookahead),
                make_state!(306, lookahead) => State::<306>::process_state(&mut parser, lookahead),
                make_state!(307, lookahead) => State::<307>::process_state(&mut parser, lookahead),
                make_state!(308, lookahead) => State::<308>::process_state(&mut parser, lookahead),
                make_state!(309, lookahead) => State::<309>::process_state(&mut parser, lookahead),
                make_state!(310, lookahead) => State::<310>::process_state(&mut parser, lookahead),
                make_state!(311, lookahead) => State::<311>::process_state(&mut parser, lookahead),
                make_state!(312, lookahead) => State::<312>::process_state(&mut parser, lookahead),
                make_state!(313, lookahead) => State::<313>::process_state(&mut parser, lookahead),
                make_state!(314, lookahead) => State::<314>::process_state(&mut parser, lookahead),
                make_state!(315, lookahead) => State::<315>::process_state(&mut parser, lookahead),
                make_state!(316, lookahead) => State::<316>::process_state(&mut parser, lookahead),
                make_state!(317, lookahead) => State::<317>::process_state(&mut parser, lookahead),
                make_state!(318, lookahead) => State::<318>::process_state(&mut parser, lookahead),
                make_state!(319, lookahead) => State::<319>::process_state(&mut parser, lookahead),
                make_state!(320, lookahead) => State::<320>::process_state(&mut parser, lookahead),
                make_state!(321, lookahead) => State::<321>::process_state(&mut parser, lookahead),
                make_state!(322, lookahead) => State::<322>::process_state(&mut parser, lookahead),
                make_state!(323, lookahead) => State::<323>::process_state(&mut parser, lookahead),
                make_state!(324, lookahead) => State::<324>::process_state(&mut parser, lookahead),
                make_state!(325, lookahead) => State::<325>::process_state(&mut parser, lookahead),
                make_state!(326, lookahead) => State::<326>::process_state(&mut parser, lookahead),
                make_state!(327, lookahead) => State::<327>::process_state(&mut parser, lookahead),
                make_state!(328, lookahead) => State::<328>::process_state(&mut parser, lookahead),
                make_state!(329, lookahead) => State::<329>::process_state(&mut parser, lookahead),
                make_state!(330, lookahead) => State::<330>::process_state(&mut parser, lookahead),
                make_state!(331, lookahead) => State::<331>::process_state(&mut parser, lookahead),
                make_state!(332, lookahead) => State::<332>::process_state(&mut parser, lookahead),
                make_state!(333, lookahead) => State::<333>::process_state(&mut parser, lookahead),
                make_state!(334, lookahead) => State::<334>::process_state(&mut parser, lookahead),
                make_state!(335, lookahead) => State::<335>::process_state(&mut parser, lookahead),
                make_state!(336, lookahead) => State::<336>::process_state(&mut parser, lookahead),
                make_state!(337, lookahead) => State::<337>::process_state(&mut parser, lookahead),
                make_state!(338, lookahead) => State::<338>::process_state(&mut parser, lookahead),
                make_state!(339, lookahead) => State::<339>::process_state(&mut parser, lookahead),
                make_state!(340, lookahead) => State::<340>::process_state(&mut parser, lookahead),
                make_state!(341, lookahead) => State::<341>::process_state(&mut parser, lookahead),
                make_state!(342, lookahead) => State::<342>::process_state(&mut parser, lookahead),
                make_state!(343, lookahead) => State::<343>::process_state(&mut parser, lookahead),
                make_state!(344, lookahead) => State::<344>::process_state(&mut parser, lookahead),
                make_state!(345, lookahead) => State::<345>::process_state(&mut parser, lookahead),
                make_state!(346, lookahead) => State::<346>::process_state(&mut parser, lookahead),
                make_state!(347, lookahead) => State::<347>::process_state(&mut parser, lookahead),
                make_state!(348, lookahead) => State::<348>::process_state(&mut parser, lookahead),
                make_state!(349, lookahead) => State::<349>::process_state(&mut parser, lookahead),
                make_state!(350, lookahead) => State::<350>::process_state(&mut parser, lookahead),
                make_state!(351, lookahead) => State::<351>::process_state(&mut parser, lookahead),
                make_state!(352, lookahead) => State::<352>::process_state(&mut parser, lookahead),
                make_state!(353, lookahead) => State::<353>::process_state(&mut parser, lookahead),
                make_state!(354, lookahead) => State::<354>::process_state(&mut parser, lookahead),
                make_state!(355, lookahead) => State::<355>::process_state(&mut parser, lookahead),
                make_state!(356, lookahead) => State::<356>::process_state(&mut parser, lookahead),
                make_state!(357, lookahead) => State::<357>::process_state(&mut parser, lookahead),
                make_state!(358, lookahead) => State::<358>::process_state(&mut parser, lookahead),
                make_state!(359, lookahead) => State::<359>::process_state(&mut parser, lookahead),
                make_state!(360, lookahead) => State::<360>::process_state(&mut parser, lookahead),
                make_state!(361, lookahead) => State::<361>::process_state(&mut parser, lookahead),
                make_state!(362, lookahead) => State::<362>::process_state(&mut parser, lookahead),
                make_state!(363, lookahead) => State::<363>::process_state(&mut parser, lookahead),
                make_state!(364, lookahead) => State::<364>::process_state(&mut parser, lookahead),
                make_state!(365, lookahead) => State::<365>::process_state(&mut parser, lookahead),
                make_state!(366, lookahead) => State::<366>::process_state(&mut parser, lookahead),
                make_state!(367, lookahead) => State::<367>::process_state(&mut parser, lookahead),
                make_state!(368, lookahead) => State::<368>::process_state(&mut parser, lookahead),
                make_state!(369, lookahead) => State::<369>::process_state(&mut parser, lookahead),
                make_state!(370, lookahead) => State::<370>::process_state(&mut parser, lookahead),
                make_state!(371, lookahead) => State::<371>::process_state(&mut parser, lookahead),
                make_state!(372, lookahead) => State::<372>::process_state(&mut parser, lookahead),
                make_state!(373, lookahead) => State::<373>::process_state(&mut parser, lookahead),
                make_state!(374, lookahead) => State::<374>::process_state(&mut parser, lookahead),
                make_state!(375, lookahead) => State::<375>::process_state(&mut parser, lookahead),
                make_state!(376, lookahead) => State::<376>::process_state(&mut parser, lookahead),
                make_state!(377, lookahead) => State::<377>::process_state(&mut parser, lookahead),
                make_state!(378, lookahead) => State::<378>::process_state(&mut parser, lookahead),
                make_state!(379, lookahead) => State::<379>::process_state(&mut parser, lookahead),
                make_state!(380, lookahead) => State::<380>::process_state(&mut parser, lookahead),
                make_state!(381, lookahead) => State::<381>::process_state(&mut parser, lookahead),
                make_state!(382, lookahead) => State::<382>::process_state(&mut parser, lookahead),
                make_state!(383, lookahead) => State::<383>::process_state(&mut parser, lookahead),
                make_state!(384, lookahead) => State::<384>::process_state(&mut parser, lookahead),
                make_state!(385, lookahead) => State::<385>::process_state(&mut parser, lookahead),
                make_state!(386, lookahead) => State::<386>::process_state(&mut parser, lookahead),
                make_state!(387, lookahead) => State::<387>::process_state(&mut parser, lookahead),
                make_state!(388, lookahead) => State::<388>::process_state(&mut parser, lookahead),
                make_state!(389, lookahead) => State::<389>::process_state(&mut parser, lookahead),
                make_state!(390, lookahead) => State::<390>::process_state(&mut parser, lookahead),
                make_state!(391, lookahead) => State::<391>::process_state(&mut parser, lookahead),
                make_state!(392, lookahead) => State::<392>::process_state(&mut parser, lookahead),
                make_state!(393, lookahead) => State::<393>::process_state(&mut parser, lookahead),
                make_state!(394, lookahead) => State::<394>::process_state(&mut parser, lookahead),
                make_state!(395, lookahead) => State::<395>::process_state(&mut parser, lookahead),
                make_state!(396, lookahead) => State::<396>::process_state(&mut parser, lookahead),
                make_state!(397, lookahead) => State::<397>::process_state(&mut parser, lookahead),
                make_state!(398, lookahead) => State::<398>::process_state(&mut parser, lookahead),
                make_state!(399, lookahead) => State::<399>::process_state(&mut parser, lookahead),
                make_state!(400, lookahead) => State::<400>::process_state(&mut parser, lookahead),
                make_state!(401, lookahead) => State::<401>::process_state(&mut parser, lookahead),
                make_state!(402, lookahead) => State::<402>::process_state(&mut parser, lookahead),
                make_state!(403, lookahead) => State::<403>::process_state(&mut parser, lookahead),
                make_state!(404, lookahead) => State::<404>::process_state(&mut parser, lookahead),
                make_state!(405, lookahead) => State::<405>::process_state(&mut parser, lookahead),
                make_state!(406, lookahead) => State::<406>::process_state(&mut parser, lookahead),
                make_state!(407, lookahead) => State::<407>::process_state(&mut parser, lookahead),
                make_state!(408, lookahead) => State::<408>::process_state(&mut parser, lookahead),
                make_state!(409, lookahead) => State::<409>::process_state(&mut parser, lookahead),
                make_state!(410, lookahead) => State::<410>::process_state(&mut parser, lookahead),
                make_state!(411, lookahead) => State::<411>::process_state(&mut parser, lookahead),
                make_state!(412, lookahead) => State::<412>::process_state(&mut parser, lookahead),
                make_state!(413, lookahead) => State::<413>::process_state(&mut parser, lookahead),
                make_state!(414, lookahead) => State::<414>::process_state(&mut parser, lookahead),
                make_state!(415, lookahead) => State::<415>::process_state(&mut parser, lookahead),
                make_state!(416, lookahead) => State::<416>::process_state(&mut parser, lookahead),
                make_state!(417, lookahead) => State::<417>::process_state(&mut parser, lookahead),
                make_state!(418, lookahead) => State::<418>::process_state(&mut parser, lookahead),
                make_state!(419, lookahead) => State::<419>::process_state(&mut parser, lookahead),
                make_state!(420, lookahead) => State::<420>::process_state(&mut parser, lookahead),
                make_state!(421, lookahead) => State::<421>::process_state(&mut parser, lookahead),
                make_state!(422, lookahead) => State::<422>::process_state(&mut parser, lookahead),
                make_state!(423, lookahead) => State::<423>::process_state(&mut parser, lookahead),
                make_state!(424, lookahead) => State::<424>::process_state(&mut parser, lookahead),
                make_state!(425, lookahead) => State::<425>::process_state(&mut parser, lookahead),
                make_state!(426, lookahead) => State::<426>::process_state(&mut parser, lookahead),
                make_state!(427, lookahead) => State::<427>::process_state(&mut parser, lookahead),
                make_state!(428, lookahead) => State::<428>::process_state(&mut parser, lookahead),
                make_state!(429, lookahead) => State::<429>::process_state(&mut parser, lookahead),
                make_state!(430, lookahead) => State::<430>::process_state(&mut parser, lookahead),
                make_state!(431, lookahead) => State::<431>::process_state(&mut parser, lookahead),
                make_state!(432, lookahead) => State::<432>::process_state(&mut parser, lookahead),
                make_state!(433, lookahead) => State::<433>::process_state(&mut parser, lookahead),
                make_state!(434, lookahead) => State::<434>::process_state(&mut parser, lookahead),
                make_state!(435, lookahead) => State::<435>::process_state(&mut parser, lookahead),
                make_state!(436, lookahead) => State::<436>::process_state(&mut parser, lookahead),
                make_state!(437, lookahead) => State::<437>::process_state(&mut parser, lookahead),
                make_state!(438, lookahead) => State::<438>::process_state(&mut parser, lookahead),
                make_state!(439, lookahead) => State::<439>::process_state(&mut parser, lookahead),
                make_state!(440, lookahead) => State::<440>::process_state(&mut parser, lookahead),
                make_state!(441, lookahead) => State::<441>::process_state(&mut parser, lookahead),
                make_state!(442, lookahead) => State::<442>::process_state(&mut parser, lookahead),
                make_state!(443, lookahead) => State::<443>::process_state(&mut parser, lookahead),
                make_state!(444, lookahead) => State::<444>::process_state(&mut parser, lookahead),
                make_state!(445, lookahead) => State::<445>::process_state(&mut parser, lookahead),
                make_state!(446, lookahead) => State::<446>::process_state(&mut parser, lookahead),
                make_state!(447, lookahead) => State::<447>::process_state(&mut parser, lookahead),
                make_state!(448, lookahead) => State::<448>::process_state(&mut parser, lookahead),
                make_state!(449, lookahead) => State::<449>::process_state(&mut parser, lookahead),
                make_state!(450, lookahead) => State::<450>::process_state(&mut parser, lookahead),
                make_state!(451, lookahead) => State::<451>::process_state(&mut parser, lookahead),
                make_state!(452, lookahead) => State::<452>::process_state(&mut parser, lookahead),
                make_state!(453, lookahead) => State::<453>::process_state(&mut parser, lookahead),
                make_state!(454, lookahead) => State::<454>::process_state(&mut parser, lookahead),
                make_state!(455, lookahead) => State::<455>::process_state(&mut parser, lookahead),
                make_state!(456, lookahead) => State::<456>::process_state(&mut parser, lookahead),
                make_state!(457, lookahead) => State::<457>::process_state(&mut parser, lookahead),
                make_state!(458, lookahead) => State::<458>::process_state(&mut parser, lookahead),
                make_state!(459, lookahead) => State::<459>::process_state(&mut parser, lookahead),
                make_state!(460, lookahead) => State::<460>::process_state(&mut parser, lookahead),
                make_state!(461, lookahead) => State::<461>::process_state(&mut parser, lookahead),
                make_state!(462, lookahead) => State::<462>::process_state(&mut parser, lookahead),
                make_state!(463, lookahead) => State::<463>::process_state(&mut parser, lookahead),
                make_state!(464, lookahead) => State::<464>::process_state(&mut parser, lookahead),
                make_state!(465, lookahead) => State::<465>::process_state(&mut parser, lookahead),
                make_state!(466, lookahead) => State::<466>::process_state(&mut parser, lookahead),
                make_state!(467, lookahead) => State::<467>::process_state(&mut parser, lookahead),
                make_state!(468, lookahead) => State::<468>::process_state(&mut parser, lookahead),
                make_state!(469, lookahead) => State::<469>::process_state(&mut parser, lookahead),
                make_state!(470, lookahead) => State::<470>::process_state(&mut parser, lookahead),
                make_state!(471, lookahead) => State::<471>::process_state(&mut parser, lookahead),
                make_state!(472, lookahead) => State::<472>::process_state(&mut parser, lookahead),
                make_state!(473, lookahead) => State::<473>::process_state(&mut parser, lookahead),
                make_state!(474, lookahead) => State::<474>::process_state(&mut parser, lookahead),
                make_state!(475, lookahead) => State::<475>::process_state(&mut parser, lookahead),
                make_state!(476, lookahead) => State::<476>::process_state(&mut parser, lookahead),
                make_state!(477, lookahead) => State::<477>::process_state(&mut parser, lookahead),
                make_state!(478, lookahead) => State::<478>::process_state(&mut parser, lookahead),
                make_state!(479, lookahead) => State::<479>::process_state(&mut parser, lookahead),
                make_state!(480, lookahead) => State::<480>::process_state(&mut parser, lookahead),
                make_state!(481, lookahead) => State::<481>::process_state(&mut parser, lookahead),
                make_state!(482, lookahead) => State::<482>::process_state(&mut parser, lookahead),
                make_state!(483, lookahead) => State::<483>::process_state(&mut parser, lookahead),
                make_state!(484, lookahead) => State::<484>::process_state(&mut parser, lookahead),
                make_state!(485, lookahead) => State::<485>::process_state(&mut parser, lookahead),
                make_state!(486, lookahead) => State::<486>::process_state(&mut parser, lookahead),
                make_state!(487, lookahead) => State::<487>::process_state(&mut parser, lookahead),
                make_state!(488, lookahead) => State::<488>::process_state(&mut parser, lookahead),
                make_state!(489, lookahead) => State::<489>::process_state(&mut parser, lookahead),
                make_state!(490, lookahead) => State::<490>::process_state(&mut parser, lookahead),
                make_state!(491, lookahead) => State::<491>::process_state(&mut parser, lookahead),
                make_state!(492, lookahead) => State::<492>::process_state(&mut parser, lookahead),
                make_state!(493, lookahead) => State::<493>::process_state(&mut parser, lookahead),
                make_state!(494, lookahead) => State::<494>::process_state(&mut parser, lookahead),
                make_state!(495, lookahead) => State::<495>::process_state(&mut parser, lookahead),
                make_state!(496, lookahead) => State::<496>::process_state(&mut parser, lookahead),
                make_state!(497, lookahead) => State::<497>::process_state(&mut parser, lookahead),
                make_state!(498, lookahead) => State::<498>::process_state(&mut parser, lookahead),
                make_state!(499, lookahead) => State::<499>::process_state(&mut parser, lookahead),
                make_state!(500, lookahead) => State::<500>::process_state(&mut parser, lookahead),
                make_state!(501, lookahead) => State::<501>::process_state(&mut parser, lookahead),
                make_state!(502, lookahead) => State::<502>::process_state(&mut parser, lookahead),
                make_state!(503, lookahead) => State::<503>::process_state(&mut parser, lookahead),
                make_state!(504, lookahead) => State::<504>::process_state(&mut parser, lookahead),
                make_state!(505, lookahead) => State::<505>::process_state(&mut parser, lookahead),
                make_state!(506, lookahead) => State::<506>::process_state(&mut parser, lookahead),
                make_state!(507, lookahead) => State::<507>::process_state(&mut parser, lookahead),
                make_state!(508, lookahead) => State::<508>::process_state(&mut parser, lookahead),
                make_state!(509, lookahead) => State::<509>::process_state(&mut parser, lookahead),
                make_state!(510, lookahead) => State::<510>::process_state(&mut parser, lookahead),
                make_state!(511, lookahead) => State::<511>::process_state(&mut parser, lookahead),
                make_state!(512, lookahead) => State::<512>::process_state(&mut parser, lookahead),
                make_state!(513, lookahead) => State::<513>::process_state(&mut parser, lookahead),
                make_state!(514, lookahead) => State::<514>::process_state(&mut parser, lookahead),
                make_state!(515, lookahead) => State::<515>::process_state(&mut parser, lookahead),
                make_state!(516, lookahead) => State::<516>::process_state(&mut parser, lookahead),
                make_state!(517, lookahead) => State::<517>::process_state(&mut parser, lookahead),
                make_state!(518, lookahead) => State::<518>::process_state(&mut parser, lookahead),
                make_state!(519, lookahead) => State::<519>::process_state(&mut parser, lookahead),
                make_state!(520, lookahead) => State::<520>::process_state(&mut parser, lookahead),
                make_state!(521, lookahead) => State::<521>::process_state(&mut parser, lookahead),
                make_state!(522, lookahead) => State::<522>::process_state(&mut parser, lookahead),
                make_state!(523, lookahead) => State::<523>::process_state(&mut parser, lookahead),
                make_state!(524, lookahead) => State::<524>::process_state(&mut parser, lookahead),
                make_state!(525, lookahead) => State::<525>::process_state(&mut parser, lookahead),
                make_state!(526, lookahead) => State::<526>::process_state(&mut parser, lookahead),
                make_state!(527, lookahead) => State::<527>::process_state(&mut parser, lookahead),
                make_state!(528, lookahead) => State::<528>::process_state(&mut parser, lookahead),
                make_state!(529, lookahead) => State::<529>::process_state(&mut parser, lookahead),
                make_state!(530, lookahead) => State::<530>::process_state(&mut parser, lookahead),
                make_state!(531, lookahead) => State::<531>::process_state(&mut parser, lookahead),
                make_state!(532, lookahead) => State::<532>::process_state(&mut parser, lookahead),
                make_state!(533, lookahead) => State::<533>::process_state(&mut parser, lookahead),
                make_state!(534, lookahead) => State::<534>::process_state(&mut parser, lookahead),
                make_state!(535, lookahead) => State::<535>::process_state(&mut parser, lookahead),
                make_state!(536, lookahead) => State::<536>::process_state(&mut parser, lookahead),
                make_state!(537, lookahead) => State::<537>::process_state(&mut parser, lookahead),
                make_state!(538, lookahead) => State::<538>::process_state(&mut parser, lookahead),
                make_state!(539, lookahead) => State::<539>::process_state(&mut parser, lookahead),
                make_state!(540, lookahead) => State::<540>::process_state(&mut parser, lookahead),
                make_state!(541, lookahead) => State::<541>::process_state(&mut parser, lookahead),
                make_state!(542, lookahead) => State::<542>::process_state(&mut parser, lookahead),
                make_state!(543, lookahead) => State::<543>::process_state(&mut parser, lookahead),
                make_state!(544, lookahead) => State::<544>::process_state(&mut parser, lookahead),
                make_state!(545, lookahead) => State::<545>::process_state(&mut parser, lookahead),
                make_state!(546, lookahead) => State::<546>::process_state(&mut parser, lookahead),
                make_state!(547, lookahead) => State::<547>::process_state(&mut parser, lookahead),
                make_state!(548, lookahead) => State::<548>::process_state(&mut parser, lookahead),
                make_state!(549, lookahead) => State::<549>::process_state(&mut parser, lookahead),
                make_state!(550, lookahead) => State::<550>::process_state(&mut parser, lookahead),
                make_state!(551, lookahead) => State::<551>::process_state(&mut parser, lookahead),
                make_state!(552, lookahead) => State::<552>::process_state(&mut parser, lookahead),
                make_state!(553, lookahead) => State::<553>::process_state(&mut parser, lookahead),
                make_state!(554, lookahead) => State::<554>::process_state(&mut parser, lookahead),
                make_state!(555, lookahead) => State::<555>::process_state(&mut parser, lookahead),
                make_state!(556, lookahead) => State::<556>::process_state(&mut parser, lookahead),
                make_state!(557, lookahead) => State::<557>::process_state(&mut parser, lookahead),
                make_state!(558, lookahead) => State::<558>::process_state(&mut parser, lookahead),
                make_state!(559, lookahead) => State::<559>::process_state(&mut parser, lookahead),
                make_state!(560, lookahead) => State::<560>::process_state(&mut parser, lookahead),
                make_state!(561, lookahead) => State::<561>::process_state(&mut parser, lookahead),
                make_state!(562, lookahead) => State::<562>::process_state(&mut parser, lookahead),
                make_state!(563, lookahead) => State::<563>::process_state(&mut parser, lookahead),
                make_state!(564, lookahead) => State::<564>::process_state(&mut parser, lookahead),
                make_state!(565, lookahead) => State::<565>::process_state(&mut parser, lookahead),
                make_state!(566, lookahead) => State::<566>::process_state(&mut parser, lookahead),
                make_state!(567, lookahead) => State::<567>::process_state(&mut parser, lookahead),
                make_state!(568, lookahead) => State::<568>::process_state(&mut parser, lookahead),
                make_state!(569, lookahead) => State::<569>::process_state(&mut parser, lookahead),
                make_state!(570, lookahead) => State::<570>::process_state(&mut parser, lookahead),
                make_state!(571, lookahead) => State::<571>::process_state(&mut parser, lookahead),
                make_state!(572, lookahead) => State::<572>::process_state(&mut parser, lookahead),
                make_state!(573, lookahead) => State::<573>::process_state(&mut parser, lookahead),
                make_state!(574, lookahead) => State::<574>::process_state(&mut parser, lookahead),
                make_state!(575, lookahead) => State::<575>::process_state(&mut parser, lookahead),
                make_state!(576, lookahead) => State::<576>::process_state(&mut parser, lookahead),
                make_state!(577, lookahead) => State::<577>::process_state(&mut parser, lookahead),
                make_state!(578, lookahead) => State::<578>::process_state(&mut parser, lookahead),
                make_state!(579, lookahead) => State::<579>::process_state(&mut parser, lookahead),
                make_state!(580, lookahead) => State::<580>::process_state(&mut parser, lookahead),
                make_state!(581, lookahead) => State::<581>::process_state(&mut parser, lookahead),
                make_state!(582, lookahead) => State::<582>::process_state(&mut parser, lookahead),
                make_state!(583, lookahead) => State::<583>::process_state(&mut parser, lookahead),
                make_state!(584, lookahead) => State::<584>::process_state(&mut parser, lookahead),
                make_state!(585, lookahead) => State::<585>::process_state(&mut parser, lookahead),
                make_state!(586, lookahead) => State::<586>::process_state(&mut parser, lookahead),
                make_state!(587, lookahead) => State::<587>::process_state(&mut parser, lookahead),
                make_state!(588, lookahead) => State::<588>::process_state(&mut parser, lookahead),
                make_state!(589, lookahead) => State::<589>::process_state(&mut parser, lookahead),
                make_state!(590, lookahead) => State::<590>::process_state(&mut parser, lookahead),
                make_state!(591, lookahead) => State::<591>::process_state(&mut parser, lookahead),
                make_state!(592, lookahead) => State::<592>::process_state(&mut parser, lookahead),
                make_state!(593, lookahead) => State::<593>::process_state(&mut parser, lookahead),
                make_state!(594, lookahead) => State::<594>::process_state(&mut parser, lookahead),
                make_state!(595, lookahead) => State::<595>::process_state(&mut parser, lookahead),
                make_state!(596, lookahead) => State::<596>::process_state(&mut parser, lookahead),
                make_state!(597, lookahead) => State::<597>::process_state(&mut parser, lookahead),
                make_state!(598, lookahead) => State::<598>::process_state(&mut parser, lookahead),
                make_state!(599, lookahead) => State::<599>::process_state(&mut parser, lookahead),
                make_state!(600, lookahead) => State::<600>::process_state(&mut parser, lookahead),
                make_state!(601, lookahead) => State::<601>::process_state(&mut parser, lookahead),
                make_state!(602, lookahead) => State::<602>::process_state(&mut parser, lookahead),
                make_state!(603, lookahead) => State::<603>::process_state(&mut parser, lookahead),
                make_state!(604, lookahead) => State::<604>::process_state(&mut parser, lookahead),
                make_state!(605, lookahead) => State::<605>::process_state(&mut parser, lookahead),
                make_state!(606, lookahead) => State::<606>::process_state(&mut parser, lookahead),
                make_state!(607, lookahead) => State::<607>::process_state(&mut parser, lookahead),
                make_state!(608, lookahead) => State::<608>::process_state(&mut parser, lookahead),
                make_state!(609, lookahead) => State::<609>::process_state(&mut parser, lookahead),
                make_state!(610, lookahead) => State::<610>::process_state(&mut parser, lookahead),
                make_state!(611, lookahead) => State::<611>::process_state(&mut parser, lookahead),
                make_state!(612, lookahead) => State::<612>::process_state(&mut parser, lookahead),
                make_state!(613, lookahead) => State::<613>::process_state(&mut parser, lookahead),
                make_state!(614, lookahead) => State::<614>::process_state(&mut parser, lookahead),
                make_state!(615, lookahead) => State::<615>::process_state(&mut parser, lookahead),
                make_state!(616, lookahead) => State::<616>::process_state(&mut parser, lookahead),
                make_state!(617, lookahead) => State::<617>::process_state(&mut parser, lookahead),
                make_state!(618, lookahead) => State::<618>::process_state(&mut parser, lookahead),
                make_state!(619, lookahead) => State::<619>::process_state(&mut parser, lookahead),
                make_state!(620, lookahead) => State::<620>::process_state(&mut parser, lookahead),
                make_state!(621, lookahead) => State::<621>::process_state(&mut parser, lookahead),
                make_state!(622, lookahead) => State::<622>::process_state(&mut parser, lookahead),
                make_state!(623, lookahead) => State::<623>::process_state(&mut parser, lookahead),
                make_state!(624, lookahead) => State::<624>::process_state(&mut parser, lookahead),
                make_state!(625, lookahead) => State::<625>::process_state(&mut parser, lookahead),
                make_state!(626, lookahead) => State::<626>::process_state(&mut parser, lookahead),
                make_state!(627, lookahead) => State::<627>::process_state(&mut parser, lookahead),
                make_state!(628, lookahead) => State::<628>::process_state(&mut parser, lookahead),
                make_state!(629, lookahead) => State::<629>::process_state(&mut parser, lookahead),
                make_state!(630, lookahead) => State::<630>::process_state(&mut parser, lookahead),
                make_state!(631, lookahead) => State::<631>::process_state(&mut parser, lookahead),
                make_state!(632, lookahead) => State::<632>::process_state(&mut parser, lookahead),
                make_state!(633, lookahead) => State::<633>::process_state(&mut parser, lookahead),
                make_state!(634, lookahead) => State::<634>::process_state(&mut parser, lookahead),
                make_state!(635, lookahead) => State::<635>::process_state(&mut parser, lookahead),
                make_state!(636, lookahead) => State::<636>::process_state(&mut parser, lookahead),
                make_state!(637, lookahead) => State::<637>::process_state(&mut parser, lookahead),
                make_state!(638, lookahead) => State::<638>::process_state(&mut parser, lookahead),
                make_state!(639, lookahead) => State::<639>::process_state(&mut parser, lookahead),
                make_state!(640, lookahead) => State::<640>::process_state(&mut parser, lookahead),
                make_state!(641, lookahead) => State::<641>::process_state(&mut parser, lookahead),
                make_state!(642, lookahead) => State::<642>::process_state(&mut parser, lookahead),
                make_state!(643, lookahead) => State::<643>::process_state(&mut parser, lookahead),
                make_state!(644, lookahead) => State::<644>::process_state(&mut parser, lookahead),
                make_state!(645, lookahead) => State::<645>::process_state(&mut parser, lookahead),
                make_state!(646, lookahead) => State::<646>::process_state(&mut parser, lookahead),
                make_state!(647, lookahead) => State::<647>::process_state(&mut parser, lookahead),
                make_state!(648, lookahead) => State::<648>::process_state(&mut parser, lookahead),
                make_state!(649, lookahead) => State::<649>::process_state(&mut parser, lookahead),
                make_state!(650, lookahead) => State::<650>::process_state(&mut parser, lookahead),
                make_state!(651, lookahead) => State::<651>::process_state(&mut parser, lookahead),
                make_state!(652, lookahead) => State::<652>::process_state(&mut parser, lookahead),
                make_state!(653, lookahead) => State::<653>::process_state(&mut parser, lookahead),
                make_state!(654, lookahead) => State::<654>::process_state(&mut parser, lookahead),
                make_state!(655, lookahead) => State::<655>::process_state(&mut parser, lookahead),
                make_state!(656, lookahead) => State::<656>::process_state(&mut parser, lookahead),
                make_state!(657, lookahead) => State::<657>::process_state(&mut parser, lookahead),
                make_state!(658, lookahead) => State::<658>::process_state(&mut parser, lookahead),
                make_state!(659, lookahead) => State::<659>::process_state(&mut parser, lookahead),
                make_state!(660, lookahead) => State::<660>::process_state(&mut parser, lookahead),
                make_state!(661, lookahead) => State::<661>::process_state(&mut parser, lookahead),
                make_state!(662, lookahead) => State::<662>::process_state(&mut parser, lookahead),
                make_state!(663, lookahead) => State::<663>::process_state(&mut parser, lookahead),
                make_state!(664, lookahead) => State::<664>::process_state(&mut parser, lookahead),
                make_state!(665, lookahead) => State::<665>::process_state(&mut parser, lookahead),
                make_state!(666, lookahead) => State::<666>::process_state(&mut parser, lookahead),
                make_state!(667, lookahead) => State::<667>::process_state(&mut parser, lookahead),
                make_state!(668, lookahead) => State::<668>::process_state(&mut parser, lookahead),
                make_state!(669, lookahead) => State::<669>::process_state(&mut parser, lookahead),
                make_state!(670, lookahead) => State::<670>::process_state(&mut parser, lookahead),
                make_state!(671, lookahead) => State::<671>::process_state(&mut parser, lookahead),
                make_state!(672, lookahead) => State::<672>::process_state(&mut parser, lookahead),
                make_state!(673, lookahead) => State::<673>::process_state(&mut parser, lookahead),
                make_state!(674, lookahead) => State::<674>::process_state(&mut parser, lookahead),
                make_state!(675, lookahead) => State::<675>::process_state(&mut parser, lookahead),
                make_state!(676, lookahead) => State::<676>::process_state(&mut parser, lookahead),
                make_state!(677, lookahead) => State::<677>::process_state(&mut parser, lookahead),
                make_state!(678, lookahead) => State::<678>::process_state(&mut parser, lookahead),
                make_state!(679, lookahead) => State::<679>::process_state(&mut parser, lookahead),
                make_state!(680, lookahead) => State::<680>::process_state(&mut parser, lookahead),
                make_state!(681, lookahead) => State::<681>::process_state(&mut parser, lookahead),
                make_state!(682, lookahead) => State::<682>::process_state(&mut parser, lookahead),
                make_state!(683, lookahead) => State::<683>::process_state(&mut parser, lookahead),
                make_state!(684, lookahead) => State::<684>::process_state(&mut parser, lookahead),
                make_state!(685, lookahead) => State::<685>::process_state(&mut parser, lookahead),
                make_state!(686, lookahead) => State::<686>::process_state(&mut parser, lookahead),
                make_state!(687, lookahead) => State::<687>::process_state(&mut parser, lookahead),
                make_state!(688, lookahead) => State::<688>::process_state(&mut parser, lookahead),
                make_state!(689, lookahead) => State::<689>::process_state(&mut parser, lookahead),
                make_state!(690, lookahead) => State::<690>::process_state(&mut parser, lookahead),
                make_state!(691, lookahead) => State::<691>::process_state(&mut parser, lookahead),
                make_state!(692, lookahead) => State::<692>::process_state(&mut parser, lookahead),
                make_state!(693, lookahead) => State::<693>::process_state(&mut parser, lookahead),
                make_state!(694, lookahead) => State::<694>::process_state(&mut parser, lookahead),
                make_state!(695, lookahead) => State::<695>::process_state(&mut parser, lookahead),
                make_state!(696, lookahead) => State::<696>::process_state(&mut parser, lookahead),
                make_state!(697, lookahead) => State::<697>::process_state(&mut parser, lookahead),
                make_state!(698, lookahead) => State::<698>::process_state(&mut parser, lookahead),
                make_state!(699, lookahead) => State::<699>::process_state(&mut parser, lookahead),
                make_state!(700, lookahead) => State::<700>::process_state(&mut parser, lookahead),
                make_state!(701, lookahead) => State::<701>::process_state(&mut parser, lookahead),
                make_state!(702, lookahead) => State::<702>::process_state(&mut parser, lookahead),
                make_state!(703, lookahead) => State::<703>::process_state(&mut parser, lookahead),
                make_state!(704, lookahead) => State::<704>::process_state(&mut parser, lookahead),
                make_state!(705, lookahead) => State::<705>::process_state(&mut parser, lookahead),
                make_state!(706, lookahead) => State::<706>::process_state(&mut parser, lookahead),
                make_state!(707, lookahead) => State::<707>::process_state(&mut parser, lookahead),
                make_state!(708, lookahead) => State::<708>::process_state(&mut parser, lookahead),
                make_state!(709, lookahead) => State::<709>::process_state(&mut parser, lookahead),
                make_state!(710, lookahead) => State::<710>::process_state(&mut parser, lookahead),
                make_state!(711, lookahead) => State::<711>::process_state(&mut parser, lookahead),
                make_state!(712, lookahead) => State::<712>::process_state(&mut parser, lookahead),
                make_state!(713, lookahead) => State::<713>::process_state(&mut parser, lookahead),
                make_state!(714, lookahead) => State::<714>::process_state(&mut parser, lookahead),
                make_state!(715, lookahead) => State::<715>::process_state(&mut parser, lookahead),
                make_state!(716, lookahead) => State::<716>::process_state(&mut parser, lookahead),
                make_state!(717, lookahead) => State::<717>::process_state(&mut parser, lookahead),
                make_state!(718, lookahead) => State::<718>::process_state(&mut parser, lookahead),
                make_state!(719, lookahead) => State::<719>::process_state(&mut parser, lookahead),
                make_state!(720, lookahead) => State::<720>::process_state(&mut parser, lookahead),
                make_state!(721, lookahead) => State::<721>::process_state(&mut parser, lookahead),
                make_state!(722, lookahead) => State::<722>::process_state(&mut parser, lookahead),
                make_state!(723, lookahead) => State::<723>::process_state(&mut parser, lookahead),
                make_state!(724, lookahead) => State::<724>::process_state(&mut parser, lookahead),
                make_state!(725, lookahead) => State::<725>::process_state(&mut parser, lookahead),
                make_state!(726, lookahead) => State::<726>::process_state(&mut parser, lookahead),
                make_state!(727, lookahead) => State::<727>::process_state(&mut parser, lookahead),
                make_state!(728, lookahead) => State::<728>::process_state(&mut parser, lookahead),
                make_state!(729, lookahead) => State::<729>::process_state(&mut parser, lookahead),
                make_state!(730, lookahead) => State::<730>::process_state(&mut parser, lookahead),
                make_state!(731, lookahead) => State::<731>::process_state(&mut parser, lookahead),
                make_state!(732, lookahead) => State::<732>::process_state(&mut parser, lookahead),
                make_state!(733, lookahead) => State::<733>::process_state(&mut parser, lookahead),
                make_state!(734, lookahead) => State::<734>::process_state(&mut parser, lookahead),
                make_state!(735, lookahead) => State::<735>::process_state(&mut parser, lookahead),
                make_state!(736, lookahead) => State::<736>::process_state(&mut parser, lookahead),
                make_state!(737, lookahead) => State::<737>::process_state(&mut parser, lookahead),
                make_state!(738, lookahead) => State::<738>::process_state(&mut parser, lookahead),
                make_state!(739, lookahead) => State::<739>::process_state(&mut parser, lookahead),
                make_state!(740, lookahead) => State::<740>::process_state(&mut parser, lookahead),
                make_state!(741, lookahead) => State::<741>::process_state(&mut parser, lookahead),
                make_state!(742, lookahead) => State::<742>::process_state(&mut parser, lookahead),
                make_state!(743, lookahead) => State::<743>::process_state(&mut parser, lookahead),
                make_state!(744, lookahead) => State::<744>::process_state(&mut parser, lookahead),
                make_state!(745, lookahead) => State::<745>::process_state(&mut parser, lookahead),
                make_state!(746, lookahead) => State::<746>::process_state(&mut parser, lookahead),
                make_state!(747, lookahead) => State::<747>::process_state(&mut parser, lookahead),
                make_state!(748, lookahead) => State::<748>::process_state(&mut parser, lookahead),
                make_state!(749, lookahead) => State::<749>::process_state(&mut parser, lookahead),
                make_state!(750, lookahead) => State::<750>::process_state(&mut parser, lookahead),
                make_state!(751, lookahead) => State::<751>::process_state(&mut parser, lookahead),
                make_state!(752, lookahead) => State::<752>::process_state(&mut parser, lookahead),
                make_state!(753, lookahead) => State::<753>::process_state(&mut parser, lookahead),
                make_state!(754, lookahead) => State::<754>::process_state(&mut parser, lookahead),
                make_state!(755, lookahead) => State::<755>::process_state(&mut parser, lookahead),
                make_state!(756, lookahead) => State::<756>::process_state(&mut parser, lookahead),
                make_state!(757, lookahead) => State::<757>::process_state(&mut parser, lookahead),
                make_state!(758, lookahead) => State::<758>::process_state(&mut parser, lookahead),
                make_state!(759, lookahead) => State::<759>::process_state(&mut parser, lookahead),
                make_state!(760, lookahead) => State::<760>::process_state(&mut parser, lookahead),
                make_state!(761, lookahead) => State::<761>::process_state(&mut parser, lookahead),
                make_state!(762, lookahead) => State::<762>::process_state(&mut parser, lookahead),
                make_state!(763, lookahead) => State::<763>::process_state(&mut parser, lookahead),
                make_state!(764, lookahead) => State::<764>::process_state(&mut parser, lookahead),
                make_state!(765, lookahead) => State::<765>::process_state(&mut parser, lookahead),
                make_state!(766, lookahead) => State::<766>::process_state(&mut parser, lookahead),
                make_state!(767, lookahead) => State::<767>::process_state(&mut parser, lookahead),
                make_state!(768, lookahead) => State::<768>::process_state(&mut parser, lookahead),
                make_state!(769, lookahead) => State::<769>::process_state(&mut parser, lookahead),
                make_state!(770, lookahead) => State::<770>::process_state(&mut parser, lookahead),
                make_state!(771, lookahead) => State::<771>::process_state(&mut parser, lookahead),
                make_state!(772, lookahead) => State::<772>::process_state(&mut parser, lookahead),
                make_state!(773, lookahead) => State::<773>::process_state(&mut parser, lookahead),
                make_state!(774, lookahead) => State::<774>::process_state(&mut parser, lookahead),
                make_state!(775, lookahead) => State::<775>::process_state(&mut parser, lookahead),
                make_state!(776, lookahead) => State::<776>::process_state(&mut parser, lookahead),
                make_state!(777, lookahead) => State::<777>::process_state(&mut parser, lookahead),
                make_state!(778, lookahead) => State::<778>::process_state(&mut parser, lookahead),
                make_state!(779, lookahead) => State::<779>::process_state(&mut parser, lookahead),
                make_state!(780, lookahead) => State::<780>::process_state(&mut parser, lookahead),
                make_state!(781, lookahead) => State::<781>::process_state(&mut parser, lookahead),
                make_state!(782, lookahead) => State::<782>::process_state(&mut parser, lookahead),
                make_state!(783, lookahead) => State::<783>::process_state(&mut parser, lookahead),
                make_state!(784, lookahead) => State::<784>::process_state(&mut parser, lookahead),
                make_state!(785, lookahead) => State::<785>::process_state(&mut parser, lookahead),
                make_state!(786, lookahead) => State::<786>::process_state(&mut parser, lookahead),
                make_state!(787, lookahead) => State::<787>::process_state(&mut parser, lookahead),
                make_state!(788, lookahead) => State::<788>::process_state(&mut parser, lookahead),
                make_state!(789, lookahead) => State::<789>::process_state(&mut parser, lookahead),
                make_state!(790, lookahead) => State::<790>::process_state(&mut parser, lookahead),
                make_state!(791, lookahead) => State::<791>::process_state(&mut parser, lookahead),
                make_state!(792, lookahead) => State::<792>::process_state(&mut parser, lookahead),
                make_state!(793, lookahead) => State::<793>::process_state(&mut parser, lookahead),
                make_state!(794, lookahead) => State::<794>::process_state(&mut parser, lookahead),
                make_state!(795, lookahead) => State::<795>::process_state(&mut parser, lookahead),
                make_state!(796, lookahead) => State::<796>::process_state(&mut parser, lookahead),
                make_state!(797, lookahead) => State::<797>::process_state(&mut parser, lookahead),
                make_state!(798, lookahead) => State::<798>::process_state(&mut parser, lookahead),
                make_state!(799, lookahead) => State::<799>::process_state(&mut parser, lookahead),
                make_state!(800, lookahead) => State::<800>::process_state(&mut parser, lookahead),
                make_state!(801, lookahead) => State::<801>::process_state(&mut parser, lookahead),
                make_state!(802, lookahead) => State::<802>::process_state(&mut parser, lookahead),
                make_state!(803, lookahead) => State::<803>::process_state(&mut parser, lookahead),
                make_state!(804, lookahead) => State::<804>::process_state(&mut parser, lookahead),
                make_state!(805, lookahead) => State::<805>::process_state(&mut parser, lookahead),
                make_state!(806, lookahead) => State::<806>::process_state(&mut parser, lookahead),
                make_state!(807, lookahead) => State::<807>::process_state(&mut parser, lookahead),
                make_state!(808, lookahead) => State::<808>::process_state(&mut parser, lookahead),
                make_state!(809, lookahead) => State::<809>::process_state(&mut parser, lookahead),
                make_state!(810, lookahead) => State::<810>::process_state(&mut parser, lookahead),
                make_state!(811, lookahead) => State::<811>::process_state(&mut parser, lookahead),
                make_state!(812, lookahead) => State::<812>::process_state(&mut parser, lookahead),
                make_state!(813, lookahead) => State::<813>::process_state(&mut parser, lookahead),
                make_state!(814, lookahead) => State::<814>::process_state(&mut parser, lookahead),
                make_state!(815, lookahead) => State::<815>::process_state(&mut parser, lookahead),
                make_state!(816, lookahead) => State::<816>::process_state(&mut parser, lookahead),
                make_state!(817, lookahead) => State::<817>::process_state(&mut parser, lookahead),
                make_state!(818, lookahead) => State::<818>::process_state(&mut parser, lookahead),
                make_state!(819, lookahead) => State::<819>::process_state(&mut parser, lookahead),
                make_state!(820, lookahead) => State::<820>::process_state(&mut parser, lookahead),
                make_state!(821, lookahead) => State::<821>::process_state(&mut parser, lookahead),
                make_state!(822, lookahead) => State::<822>::process_state(&mut parser, lookahead),
                make_state!(823, lookahead) => State::<823>::process_state(&mut parser, lookahead),
                make_state!(824, lookahead) => State::<824>::process_state(&mut parser, lookahead),
                make_state!(825, lookahead) => State::<825>::process_state(&mut parser, lookahead),
                make_state!(826, lookahead) => State::<826>::process_state(&mut parser, lookahead),
                make_state!(827, lookahead) => State::<827>::process_state(&mut parser, lookahead),
                make_state!(828, lookahead) => State::<828>::process_state(&mut parser, lookahead),
                make_state!(829, lookahead) => State::<829>::process_state(&mut parser, lookahead),
                make_state!(830, lookahead) => State::<830>::process_state(&mut parser, lookahead),
                make_state!(831, lookahead) => State::<831>::process_state(&mut parser, lookahead),
                make_state!(832, lookahead) => State::<832>::process_state(&mut parser, lookahead),
                make_state!(833, lookahead) => State::<833>::process_state(&mut parser, lookahead),
                make_state!(834, lookahead) => State::<834>::process_state(&mut parser, lookahead),
                make_state!(835, lookahead) => State::<835>::process_state(&mut parser, lookahead),
                make_state!(836, lookahead) => State::<836>::process_state(&mut parser, lookahead),
                make_state!(837, lookahead) => State::<837>::process_state(&mut parser, lookahead),
                make_state!(838, lookahead) => State::<838>::process_state(&mut parser, lookahead),
                make_state!(839, lookahead) => State::<839>::process_state(&mut parser, lookahead),
                make_state!(840, lookahead) => State::<840>::process_state(&mut parser, lookahead),
                make_state!(841, lookahead) => State::<841>::process_state(&mut parser, lookahead),
                make_state!(842, lookahead) => State::<842>::process_state(&mut parser, lookahead),
                make_state!(843, lookahead) => State::<843>::process_state(&mut parser, lookahead),
                make_state!(844, lookahead) => State::<844>::process_state(&mut parser, lookahead),
                make_state!(845, lookahead) => State::<845>::process_state(&mut parser, lookahead),
                make_state!(846, lookahead) => State::<846>::process_state(&mut parser, lookahead),
                make_state!(847, lookahead) => State::<847>::process_state(&mut parser, lookahead),
                make_state!(848, lookahead) => State::<848>::process_state(&mut parser, lookahead),
                make_state!(849, lookahead) => State::<849>::process_state(&mut parser, lookahead),
                make_state!(850, lookahead) => State::<850>::process_state(&mut parser, lookahead),
                make_state!(851, lookahead) => State::<851>::process_state(&mut parser, lookahead),
                make_state!(852, lookahead) => State::<852>::process_state(&mut parser, lookahead),
                make_state!(853, lookahead) => State::<853>::process_state(&mut parser, lookahead),
                make_state!(854, lookahead) => State::<854>::process_state(&mut parser, lookahead),
                make_state!(855, lookahead) => State::<855>::process_state(&mut parser, lookahead),
                make_state!(856, lookahead) => State::<856>::process_state(&mut parser, lookahead),
                make_state!(857, lookahead) => State::<857>::process_state(&mut parser, lookahead),
                make_state!(858, lookahead) => State::<858>::process_state(&mut parser, lookahead),
                make_state!(859, lookahead) => State::<859>::process_state(&mut parser, lookahead),
                make_state!(860, lookahead) => State::<860>::process_state(&mut parser, lookahead),
                make_state!(861, lookahead) => State::<861>::process_state(&mut parser, lookahead),
                make_state!(862, lookahead) => State::<862>::process_state(&mut parser, lookahead),
                make_state!(863, lookahead) => State::<863>::process_state(&mut parser, lookahead),
                make_state!(864, lookahead) => State::<864>::process_state(&mut parser, lookahead),
                make_state!(865, lookahead) => State::<865>::process_state(&mut parser, lookahead),
                make_state!(866, lookahead) => State::<866>::process_state(&mut parser, lookahead),
                make_state!(867, lookahead) => State::<867>::process_state(&mut parser, lookahead),
                make_state!(868, lookahead) => State::<868>::process_state(&mut parser, lookahead),
                make_state!(869, lookahead) => State::<869>::process_state(&mut parser, lookahead),
                make_state!(870, lookahead) => State::<870>::process_state(&mut parser, lookahead),
                make_state!(871, lookahead) => State::<871>::process_state(&mut parser, lookahead),
                make_state!(872, lookahead) => State::<872>::process_state(&mut parser, lookahead),
                make_state!(873, lookahead) => State::<873>::process_state(&mut parser, lookahead),
                make_state!(874, lookahead) => State::<874>::process_state(&mut parser, lookahead),
                make_state!(875, lookahead) => State::<875>::process_state(&mut parser, lookahead),
                make_state!(876, lookahead) => State::<876>::process_state(&mut parser, lookahead),
                make_state!(877, lookahead) => State::<877>::process_state(&mut parser, lookahead),
                make_state!(878, lookahead) => State::<878>::process_state(&mut parser, lookahead),
                make_state!(879, lookahead) => State::<879>::process_state(&mut parser, lookahead),
                make_state!(880, lookahead) => State::<880>::process_state(&mut parser, lookahead),
                make_state!(881, lookahead) => State::<881>::process_state(&mut parser, lookahead),
                make_state!(882, lookahead) => State::<882>::process_state(&mut parser, lookahead),
                make_state!(883, lookahead) => State::<883>::process_state(&mut parser, lookahead),
                make_state!(884, lookahead) => State::<884>::process_state(&mut parser, lookahead),
                make_state!(885, lookahead) => State::<885>::process_state(&mut parser, lookahead),
                make_state!(886, lookahead) => State::<886>::process_state(&mut parser, lookahead),
                make_state!(887, lookahead) => State::<887>::process_state(&mut parser, lookahead),
                make_state!(888, lookahead) => State::<888>::process_state(&mut parser, lookahead),
                make_state!(889, lookahead) => State::<889>::process_state(&mut parser, lookahead),
                make_state!(890, lookahead) => State::<890>::process_state(&mut parser, lookahead),
                make_state!(891, lookahead) => State::<891>::process_state(&mut parser, lookahead),
                make_state!(892, lookahead) => State::<892>::process_state(&mut parser, lookahead),
                make_state!(893, lookahead) => State::<893>::process_state(&mut parser, lookahead),
                make_state!(894, lookahead) => State::<894>::process_state(&mut parser, lookahead),
                make_state!(895, lookahead) => State::<895>::process_state(&mut parser, lookahead),
                make_state!(896, lookahead) => State::<896>::process_state(&mut parser, lookahead),
                make_state!(897, lookahead) => State::<897>::process_state(&mut parser, lookahead),
                make_state!(898, lookahead) => State::<898>::process_state(&mut parser, lookahead),
                make_state!(899, lookahead) => State::<899>::process_state(&mut parser, lookahead),
                make_state!(900, lookahead) => State::<900>::process_state(&mut parser, lookahead),
                make_state!(901, lookahead) => State::<901>::process_state(&mut parser, lookahead),
                make_state!(902, lookahead) => State::<902>::process_state(&mut parser, lookahead),
                make_state!(903, lookahead) => State::<903>::process_state(&mut parser, lookahead),
                make_state!(904, lookahead) => State::<904>::process_state(&mut parser, lookahead),
                make_state!(905, lookahead) => State::<905>::process_state(&mut parser, lookahead),
                make_state!(906, lookahead) => State::<906>::process_state(&mut parser, lookahead),
                make_state!(907, lookahead) => State::<907>::process_state(&mut parser, lookahead),
                make_state!(908, lookahead) => State::<908>::process_state(&mut parser, lookahead),
                make_state!(909, lookahead) => State::<909>::process_state(&mut parser, lookahead),
                make_state!(910, lookahead) => State::<910>::process_state(&mut parser, lookahead),
                make_state!(911, lookahead) => State::<911>::process_state(&mut parser, lookahead),
                make_state!(912, lookahead) => State::<912>::process_state(&mut parser, lookahead),
                make_state!(913, lookahead) => State::<913>::process_state(&mut parser, lookahead),
                make_state!(914, lookahead) => State::<914>::process_state(&mut parser, lookahead),
                make_state!(915, lookahead) => State::<915>::process_state(&mut parser, lookahead),
                make_state!(916, lookahead) => State::<916>::process_state(&mut parser, lookahead),
                make_state!(917, lookahead) => State::<917>::process_state(&mut parser, lookahead),
                make_state!(918, lookahead) => State::<918>::process_state(&mut parser, lookahead),
                make_state!(919, lookahead) => State::<919>::process_state(&mut parser, lookahead),
                make_state!(920, lookahead) => State::<920>::process_state(&mut parser, lookahead),
                make_state!(921, lookahead) => State::<921>::process_state(&mut parser, lookahead),
                make_state!(922, lookahead) => State::<922>::process_state(&mut parser, lookahead),
                make_state!(923, lookahead) => State::<923>::process_state(&mut parser, lookahead),
                make_state!(924, lookahead) => State::<924>::process_state(&mut parser, lookahead),
                make_state!(925, lookahead) => State::<925>::process_state(&mut parser, lookahead),
                make_state!(926, lookahead) => State::<926>::process_state(&mut parser, lookahead),
                make_state!(927, lookahead) => State::<927>::process_state(&mut parser, lookahead),
                make_state!(928, lookahead) => State::<928>::process_state(&mut parser, lookahead),
                make_state!(929, lookahead) => State::<929>::process_state(&mut parser, lookahead),
                make_state!(930, lookahead) => State::<930>::process_state(&mut parser, lookahead),
                make_state!(931, lookahead) => State::<931>::process_state(&mut parser, lookahead),
                make_state!(932, lookahead) => State::<932>::process_state(&mut parser, lookahead),
                make_state!(933, lookahead) => State::<933>::process_state(&mut parser, lookahead),
                make_state!(934, lookahead) => State::<934>::process_state(&mut parser, lookahead),
                make_state!(935, lookahead) => State::<935>::process_state(&mut parser, lookahead),
                make_state!(936, lookahead) => State::<936>::process_state(&mut parser, lookahead),
                make_state!(937, lookahead) => State::<937>::process_state(&mut parser, lookahead),
                make_state!(938, lookahead) => State::<938>::process_state(&mut parser, lookahead),
                make_state!(939, lookahead) => State::<939>::process_state(&mut parser, lookahead),
                make_state!(940, lookahead) => State::<940>::process_state(&mut parser, lookahead),
                make_state!(941, lookahead) => State::<941>::process_state(&mut parser, lookahead),
                make_state!(942, lookahead) => State::<942>::process_state(&mut parser, lookahead),
                make_state!(943, lookahead) => State::<943>::process_state(&mut parser, lookahead),
                make_state!(944, lookahead) => State::<944>::process_state(&mut parser, lookahead),
                make_state!(945, lookahead) => State::<945>::process_state(&mut parser, lookahead),
                make_state!(946, lookahead) => State::<946>::process_state(&mut parser, lookahead),
                make_state!(947, lookahead) => State::<947>::process_state(&mut parser, lookahead),
                make_state!(948, lookahead) => State::<948>::process_state(&mut parser, lookahead),
                make_state!(949, lookahead) => State::<949>::process_state(&mut parser, lookahead),
                make_state!(950, lookahead) => State::<950>::process_state(&mut parser, lookahead),
                make_state!(951, lookahead) => State::<951>::process_state(&mut parser, lookahead),
                make_state!(952, lookahead) => State::<952>::process_state(&mut parser, lookahead),
                make_state!(953, lookahead) => State::<953>::process_state(&mut parser, lookahead),
                make_state!(954, lookahead) => State::<954>::process_state(&mut parser, lookahead),
                make_state!(955, lookahead) => State::<955>::process_state(&mut parser, lookahead),
                make_state!(956, lookahead) => State::<956>::process_state(&mut parser, lookahead),
                make_state!(957, lookahead) => State::<957>::process_state(&mut parser, lookahead),
                make_state!(958, lookahead) => State::<958>::process_state(&mut parser, lookahead),
                make_state!(959, lookahead) => State::<959>::process_state(&mut parser, lookahead),
                make_state!(960, lookahead) => State::<960>::process_state(&mut parser, lookahead),
                make_state!(961, lookahead) => State::<961>::process_state(&mut parser, lookahead),
                make_state!(962, lookahead) => State::<962>::process_state(&mut parser, lookahead),
                make_state!(963, lookahead) => State::<963>::process_state(&mut parser, lookahead),
                make_state!(964, lookahead) => State::<964>::process_state(&mut parser, lookahead),
                make_state!(965, lookahead) => State::<965>::process_state(&mut parser, lookahead),
                make_state!(966, lookahead) => State::<966>::process_state(&mut parser, lookahead),
                make_state!(967, lookahead) => State::<967>::process_state(&mut parser, lookahead),
                make_state!(968, lookahead) => State::<968>::process_state(&mut parser, lookahead),
                make_state!(969, lookahead) => State::<969>::process_state(&mut parser, lookahead),
                make_state!(970, lookahead) => State::<970>::process_state(&mut parser, lookahead),
                make_state!(971, lookahead) => State::<971>::process_state(&mut parser, lookahead),
                make_state!(972, lookahead) => State::<972>::process_state(&mut parser, lookahead),
                make_state!(973, lookahead) => State::<973>::process_state(&mut parser, lookahead),
                make_state!(974, lookahead) => State::<974>::process_state(&mut parser, lookahead),
                make_state!(975, lookahead) => State::<975>::process_state(&mut parser, lookahead),
                make_state!(976, lookahead) => State::<976>::process_state(&mut parser, lookahead),
                make_state!(977, lookahead) => State::<977>::process_state(&mut parser, lookahead),
                make_state!(978, lookahead) => State::<978>::process_state(&mut parser, lookahead),
                make_state!(979, lookahead) => State::<979>::process_state(&mut parser, lookahead),
                make_state!(980, lookahead) => State::<980>::process_state(&mut parser, lookahead),
                make_state!(981, lookahead) => State::<981>::process_state(&mut parser, lookahead),
                make_state!(982, lookahead) => State::<982>::process_state(&mut parser, lookahead),
                make_state!(983, lookahead) => State::<983>::process_state(&mut parser, lookahead),
                make_state!(984, lookahead) => State::<984>::process_state(&mut parser, lookahead),
                make_state!(985, lookahead) => State::<985>::process_state(&mut parser, lookahead),
                make_state!(986, lookahead) => State::<986>::process_state(&mut parser, lookahead),
                make_state!(987, lookahead) => State::<987>::process_state(&mut parser, lookahead),
                make_state!(988, lookahead) => State::<988>::process_state(&mut parser, lookahead),
                make_state!(989, lookahead) => State::<989>::process_state(&mut parser, lookahead),
                make_state!(990, lookahead) => State::<990>::process_state(&mut parser, lookahead),
                make_state!(991, lookahead) => State::<991>::process_state(&mut parser, lookahead),
                make_state!(992, lookahead) => State::<992>::process_state(&mut parser, lookahead),
                make_state!(993, lookahead) => State::<993>::process_state(&mut parser, lookahead),
                make_state!(994, lookahead) => State::<994>::process_state(&mut parser, lookahead),
                make_state!(995, lookahead) => State::<995>::process_state(&mut parser, lookahead),
                make_state!(996, lookahead) => State::<996>::process_state(&mut parser, lookahead),
                make_state!(997, lookahead) => State::<997>::process_state(&mut parser, lookahead),
                make_state!(998, lookahead) => State::<998>::process_state(&mut parser, lookahead),
                make_state!(999, lookahead) => State::<999>::process_state(&mut parser, lookahead),
                make_state!(1000, lookahead) => {
                    State::<1000>::process_state(&mut parser, lookahead)
                }
                make_state!(1001, lookahead) => {
                    State::<1001>::process_state(&mut parser, lookahead)
                }
                make_state!(1002, lookahead) => {
                    State::<1002>::process_state(&mut parser, lookahead)
                }
                make_state!(1003, lookahead) => {
                    State::<1003>::process_state(&mut parser, lookahead)
                }
                make_state!(1004, lookahead) => {
                    State::<1004>::process_state(&mut parser, lookahead)
                }
                make_state!(1005, lookahead) => {
                    State::<1005>::process_state(&mut parser, lookahead)
                }
                make_state!(1006, lookahead) => {
                    State::<1006>::process_state(&mut parser, lookahead)
                }
                make_state!(1007, lookahead) => {
                    State::<1007>::process_state(&mut parser, lookahead)
                }
                make_state!(1008, lookahead) => {
                    State::<1008>::process_state(&mut parser, lookahead)
                }
                make_state!(1009, lookahead) => {
                    State::<1009>::process_state(&mut parser, lookahead)
                }
                make_state!(1010, lookahead) => {
                    State::<1010>::process_state(&mut parser, lookahead)
                }
                make_state!(1011, lookahead) => {
                    State::<1011>::process_state(&mut parser, lookahead)
                }
                make_state!(1012, lookahead) => {
                    State::<1012>::process_state(&mut parser, lookahead)
                }
                make_state!(1013, lookahead) => {
                    State::<1013>::process_state(&mut parser, lookahead)
                }
                make_state!(1014, lookahead) => {
                    State::<1014>::process_state(&mut parser, lookahead)
                }
                make_state!(1015, lookahead) => {
                    State::<1015>::process_state(&mut parser, lookahead)
                }
                make_state!(1016, lookahead) => {
                    State::<1016>::process_state(&mut parser, lookahead)
                }
                make_state!(1017, lookahead) => {
                    State::<1017>::process_state(&mut parser, lookahead)
                }
                make_state!(1018, lookahead) => {
                    State::<1018>::process_state(&mut parser, lookahead)
                }
                make_state!(1019, lookahead) => {
                    State::<1019>::process_state(&mut parser, lookahead)
                }
                make_state!(1020, lookahead) => {
                    State::<1020>::process_state(&mut parser, lookahead)
                }
                make_state!(1021, lookahead) => {
                    State::<1021>::process_state(&mut parser, lookahead)
                }
                make_state!(1022, lookahead) => {
                    State::<1022>::process_state(&mut parser, lookahead)
                }
                make_state!(1023, lookahead) => {
                    State::<1023>::process_state(&mut parser, lookahead)
                }
                make_state!(1024, lookahead) => {
                    State::<1024>::process_state(&mut parser, lookahead)
                }
                make_state!(1025, lookahead) => {
                    State::<1025>::process_state(&mut parser, lookahead)
                }
                make_state!(1026, lookahead) => {
                    State::<1026>::process_state(&mut parser, lookahead)
                }
                make_state!(1027, lookahead) => {
                    State::<1027>::process_state(&mut parser, lookahead)
                }
                make_state!(1028, lookahead) => {
                    State::<1028>::process_state(&mut parser, lookahead)
                }
                make_state!(1029, lookahead) => {
                    State::<1029>::process_state(&mut parser, lookahead)
                }
                make_state!(1030, lookahead) => {
                    State::<1030>::process_state(&mut parser, lookahead)
                }
                make_state!(1031, lookahead) => {
                    State::<1031>::process_state(&mut parser, lookahead)
                }
                make_state!(1032, lookahead) => {
                    State::<1032>::process_state(&mut parser, lookahead)
                }
                make_state!(1033, lookahead) => {
                    State::<1033>::process_state(&mut parser, lookahead)
                }
                make_state!(1034, lookahead) => {
                    State::<1034>::process_state(&mut parser, lookahead)
                }
                make_state!(1035, lookahead) => {
                    State::<1035>::process_state(&mut parser, lookahead)
                }
                make_state!(1036, lookahead) => {
                    State::<1036>::process_state(&mut parser, lookahead)
                }
                make_state!(1037, lookahead) => {
                    State::<1037>::process_state(&mut parser, lookahead)
                }
                make_state!(1038, lookahead) => {
                    State::<1038>::process_state(&mut parser, lookahead)
                }
                make_state!(1039, lookahead) => {
                    State::<1039>::process_state(&mut parser, lookahead)
                }
                make_state!(1040, lookahead) => {
                    State::<1040>::process_state(&mut parser, lookahead)
                }
                make_state!(1041, lookahead) => {
                    State::<1041>::process_state(&mut parser, lookahead)
                }
                make_state!(1042, lookahead) => {
                    State::<1042>::process_state(&mut parser, lookahead)
                }
                make_state!(1043, lookahead) => {
                    State::<1043>::process_state(&mut parser, lookahead)
                }
                make_state!(1044, lookahead) => {
                    State::<1044>::process_state(&mut parser, lookahead)
                }
                make_state!(1045, lookahead) => {
                    State::<1045>::process_state(&mut parser, lookahead)
                }
                make_state!(1046, lookahead) => {
                    State::<1046>::process_state(&mut parser, lookahead)
                }
                make_state!(1047, lookahead) => {
                    State::<1047>::process_state(&mut parser, lookahead)
                }
                make_state!(1048, lookahead) => {
                    State::<1048>::process_state(&mut parser, lookahead)
                }
                make_state!(1049, lookahead) => {
                    State::<1049>::process_state(&mut parser, lookahead)
                }
                make_state!(1050, lookahead) => {
                    State::<1050>::process_state(&mut parser, lookahead)
                }
                make_state!(1051, lookahead) => {
                    State::<1051>::process_state(&mut parser, lookahead)
                }
                make_state!(1052, lookahead) => {
                    State::<1052>::process_state(&mut parser, lookahead)
                }
                make_state!(1053, lookahead) => {
                    State::<1053>::process_state(&mut parser, lookahead)
                }
                make_state!(1054, lookahead) => {
                    State::<1054>::process_state(&mut parser, lookahead)
                }
                make_state!(1055, lookahead) => {
                    State::<1055>::process_state(&mut parser, lookahead)
                }
                make_state!(1056, lookahead) => {
                    State::<1056>::process_state(&mut parser, lookahead)
                }
                make_state!(1057, lookahead) => {
                    State::<1057>::process_state(&mut parser, lookahead)
                }
                make_state!(1058, lookahead) => {
                    State::<1058>::process_state(&mut parser, lookahead)
                }
                make_state!(1059, lookahead) => {
                    State::<1059>::process_state(&mut parser, lookahead)
                }
                make_state!(1060, lookahead) => {
                    State::<1060>::process_state(&mut parser, lookahead)
                }
                make_state!(1061, lookahead) => {
                    State::<1061>::process_state(&mut parser, lookahead)
                }
                make_state!(1062, lookahead) => {
                    State::<1062>::process_state(&mut parser, lookahead)
                }
                make_state!(1063, lookahead) => {
                    State::<1063>::process_state(&mut parser, lookahead)
                }
                make_state!(1064, lookahead) => {
                    State::<1064>::process_state(&mut parser, lookahead)
                }
                make_state!(1065, lookahead) => {
                    State::<1065>::process_state(&mut parser, lookahead)
                }
                make_state!(1066, lookahead) => {
                    State::<1066>::process_state(&mut parser, lookahead)
                }
                make_state!(1067, lookahead) => {
                    State::<1067>::process_state(&mut parser, lookahead)
                }
                make_state!(1068, lookahead) => {
                    State::<1068>::process_state(&mut parser, lookahead)
                }
                make_state!(1069, lookahead) => {
                    State::<1069>::process_state(&mut parser, lookahead)
                }
                make_state!(1070, lookahead) => {
                    State::<1070>::process_state(&mut parser, lookahead)
                }
                make_state!(1071, lookahead) => {
                    State::<1071>::process_state(&mut parser, lookahead)
                }
                make_state!(1072, lookahead) => {
                    State::<1072>::process_state(&mut parser, lookahead)
                }
                make_state!(1073, lookahead) => {
                    State::<1073>::process_state(&mut parser, lookahead)
                }
                make_state!(1074, lookahead) => {
                    State::<1074>::process_state(&mut parser, lookahead)
                }
                make_state!(1075, lookahead) => {
                    State::<1075>::process_state(&mut parser, lookahead)
                }
                make_state!(1076, lookahead) => {
                    State::<1076>::process_state(&mut parser, lookahead)
                }
                make_state!(1077, lookahead) => {
                    State::<1077>::process_state(&mut parser, lookahead)
                }
                make_state!(1078, lookahead) => {
                    State::<1078>::process_state(&mut parser, lookahead)
                }
                make_state!(1079, lookahead) => {
                    State::<1079>::process_state(&mut parser, lookahead)
                }
                make_state!(1080, lookahead) => {
                    State::<1080>::process_state(&mut parser, lookahead)
                }
                make_state!(1081, lookahead) => {
                    State::<1081>::process_state(&mut parser, lookahead)
                }
                make_state!(1082, lookahead) => {
                    State::<1082>::process_state(&mut parser, lookahead)
                }
                make_state!(1083, lookahead) => {
                    State::<1083>::process_state(&mut parser, lookahead)
                }
                make_state!(1084, lookahead) => {
                    State::<1084>::process_state(&mut parser, lookahead)
                }
                make_state!(1085, lookahead) => {
                    State::<1085>::process_state(&mut parser, lookahead)
                }
                make_state!(1086, lookahead) => {
                    State::<1086>::process_state(&mut parser, lookahead)
                }
                make_state!(1087, lookahead) => {
                    State::<1087>::process_state(&mut parser, lookahead)
                }
                make_state!(1088, lookahead) => {
                    State::<1088>::process_state(&mut parser, lookahead)
                }
                make_state!(1089, lookahead) => {
                    State::<1089>::process_state(&mut parser, lookahead)
                }
                make_state!(1090, lookahead) => {
                    State::<1090>::process_state(&mut parser, lookahead)
                }
                make_state!(1091, lookahead) => {
                    State::<1091>::process_state(&mut parser, lookahead)
                }
                make_state!(1092, lookahead) => {
                    State::<1092>::process_state(&mut parser, lookahead)
                }
                make_state!(1093, lookahead) => {
                    State::<1093>::process_state(&mut parser, lookahead)
                }
                make_state!(1094, lookahead) => {
                    State::<1094>::process_state(&mut parser, lookahead)
                }
                make_state!(1095, lookahead) => {
                    State::<1095>::process_state(&mut parser, lookahead)
                }
                make_state!(1096, lookahead) => {
                    State::<1096>::process_state(&mut parser, lookahead)
                }
                make_state!(1097, lookahead) => {
                    State::<1097>::process_state(&mut parser, lookahead)
                }
                make_state!(1098, lookahead) => {
                    State::<1098>::process_state(&mut parser, lookahead)
                }
                make_state!(1099, lookahead) => {
                    State::<1099>::process_state(&mut parser, lookahead)
                }
                make_state!(1100, lookahead) => {
                    State::<1100>::process_state(&mut parser, lookahead)
                }
                make_state!(1101, lookahead) => {
                    State::<1101>::process_state(&mut parser, lookahead)
                }
                make_state!(1102, lookahead) => {
                    State::<1102>::process_state(&mut parser, lookahead)
                }
                make_state!(1103, lookahead) => {
                    State::<1103>::process_state(&mut parser, lookahead)
                }
                make_state!(1104, lookahead) => {
                    State::<1104>::process_state(&mut parser, lookahead)
                }
                make_state!(1105, lookahead) => {
                    State::<1105>::process_state(&mut parser, lookahead)
                }
                make_state!(1106, lookahead) => {
                    State::<1106>::process_state(&mut parser, lookahead)
                }
                make_state!(1107, lookahead) => {
                    State::<1107>::process_state(&mut parser, lookahead)
                }
                make_state!(1108, lookahead) => {
                    State::<1108>::process_state(&mut parser, lookahead)
                }
                make_state!(1109, lookahead) => {
                    State::<1109>::process_state(&mut parser, lookahead)
                }
                make_state!(1110, lookahead) => {
                    State::<1110>::process_state(&mut parser, lookahead)
                }
                make_state!(1111, lookahead) => {
                    State::<1111>::process_state(&mut parser, lookahead)
                }
                make_state!(1112, lookahead) => {
                    State::<1112>::process_state(&mut parser, lookahead)
                }
                make_state!(1113, lookahead) => {
                    State::<1113>::process_state(&mut parser, lookahead)
                }
                make_state!(1114, lookahead) => {
                    State::<1114>::process_state(&mut parser, lookahead)
                }
                make_state!(1115, lookahead) => {
                    State::<1115>::process_state(&mut parser, lookahead)
                }
                make_state!(1116, lookahead) => {
                    State::<1116>::process_state(&mut parser, lookahead)
                }
                make_state!(1117, lookahead) => {
                    State::<1117>::process_state(&mut parser, lookahead)
                }
                make_state!(1118, lookahead) => {
                    State::<1118>::process_state(&mut parser, lookahead)
                }
                make_state!(1119, lookahead) => {
                    State::<1119>::process_state(&mut parser, lookahead)
                }
                make_state!(1120, lookahead) => {
                    State::<1120>::process_state(&mut parser, lookahead)
                }
                make_state!(1121, lookahead) => {
                    State::<1121>::process_state(&mut parser, lookahead)
                }
                make_state!(1122, lookahead) => {
                    State::<1122>::process_state(&mut parser, lookahead)
                }
                make_state!(1123, lookahead) => {
                    State::<1123>::process_state(&mut parser, lookahead)
                }
                make_state!(1124, lookahead) => {
                    State::<1124>::process_state(&mut parser, lookahead)
                }
                make_state!(1125, lookahead) => {
                    State::<1125>::process_state(&mut parser, lookahead)
                }
                make_state!(1126, lookahead) => {
                    State::<1126>::process_state(&mut parser, lookahead)
                }
                make_state!(1127, lookahead) => {
                    State::<1127>::process_state(&mut parser, lookahead)
                }
                make_state!(1128, lookahead) => {
                    State::<1128>::process_state(&mut parser, lookahead)
                }
                make_state!(1129, lookahead) => {
                    State::<1129>::process_state(&mut parser, lookahead)
                }
                make_state!(1130, lookahead) => {
                    State::<1130>::process_state(&mut parser, lookahead)
                }
                make_state!(1131, lookahead) => {
                    State::<1131>::process_state(&mut parser, lookahead)
                }
                make_state!(1132, lookahead) => {
                    State::<1132>::process_state(&mut parser, lookahead)
                }
                make_state!(1133, lookahead) => {
                    State::<1133>::process_state(&mut parser, lookahead)
                }
                make_state!(1134, lookahead) => {
                    State::<1134>::process_state(&mut parser, lookahead)
                }
                make_state!(1135, lookahead) => {
                    State::<1135>::process_state(&mut parser, lookahead)
                }
                make_state!(1136, lookahead) => {
                    State::<1136>::process_state(&mut parser, lookahead)
                }
                make_state!(1137, lookahead) => {
                    State::<1137>::process_state(&mut parser, lookahead)
                }
                make_state!(1138, lookahead) => {
                    State::<1138>::process_state(&mut parser, lookahead)
                }
                make_state!(1139, lookahead) => {
                    State::<1139>::process_state(&mut parser, lookahead)
                }
                make_state!(1140, lookahead) => {
                    State::<1140>::process_state(&mut parser, lookahead)
                }
                make_state!(1141, lookahead) => {
                    State::<1141>::process_state(&mut parser, lookahead)
                }
                make_state!(1142, lookahead) => {
                    State::<1142>::process_state(&mut parser, lookahead)
                }
                make_state!(1143, lookahead) => {
                    State::<1143>::process_state(&mut parser, lookahead)
                }
                make_state!(1144, lookahead) => {
                    State::<1144>::process_state(&mut parser, lookahead)
                }
                make_state!(1145, lookahead) => {
                    State::<1145>::process_state(&mut parser, lookahead)
                }
                make_state!(1146, lookahead) => {
                    State::<1146>::process_state(&mut parser, lookahead)
                }
                make_state!(1147, lookahead) => {
                    State::<1147>::process_state(&mut parser, lookahead)
                }
                make_state!(1148, lookahead) => {
                    State::<1148>::process_state(&mut parser, lookahead)
                }
                make_state!(1149, lookahead) => {
                    State::<1149>::process_state(&mut parser, lookahead)
                }
                make_state!(1150, lookahead) => {
                    State::<1150>::process_state(&mut parser, lookahead)
                }
                make_state!(1151, lookahead) => {
                    State::<1151>::process_state(&mut parser, lookahead)
                }
                make_state!(1152, lookahead) => {
                    State::<1152>::process_state(&mut parser, lookahead)
                }
                make_state!(1153, lookahead) => {
                    State::<1153>::process_state(&mut parser, lookahead)
                }
                make_state!(1154, lookahead) => {
                    State::<1154>::process_state(&mut parser, lookahead)
                }
                make_state!(1155, lookahead) => {
                    State::<1155>::process_state(&mut parser, lookahead)
                }
                make_state!(1156, lookahead) => {
                    State::<1156>::process_state(&mut parser, lookahead)
                }
                make_state!(1157, lookahead) => {
                    State::<1157>::process_state(&mut parser, lookahead)
                }
                make_state!(1158, lookahead) => {
                    State::<1158>::process_state(&mut parser, lookahead)
                }
                make_state!(1159, lookahead) => {
                    State::<1159>::process_state(&mut parser, lookahead)
                }
                make_state!(1160, lookahead) => {
                    State::<1160>::process_state(&mut parser, lookahead)
                }
                make_state!(1161, lookahead) => {
                    State::<1161>::process_state(&mut parser, lookahead)
                }
                make_state!(1162, lookahead) => {
                    State::<1162>::process_state(&mut parser, lookahead)
                }
                make_state!(1163, lookahead) => {
                    State::<1163>::process_state(&mut parser, lookahead)
                }
                make_state!(1164, lookahead) => {
                    State::<1164>::process_state(&mut parser, lookahead)
                }
                make_state!(1165, lookahead) => {
                    State::<1165>::process_state(&mut parser, lookahead)
                }
                make_state!(1166, lookahead) => {
                    State::<1166>::process_state(&mut parser, lookahead)
                }
                make_state!(1167, lookahead) => {
                    State::<1167>::process_state(&mut parser, lookahead)
                }
                make_state!(1168, lookahead) => {
                    State::<1168>::process_state(&mut parser, lookahead)
                }
                make_state!(1169, lookahead) => {
                    State::<1169>::process_state(&mut parser, lookahead)
                }
                make_state!(1170, lookahead) => {
                    State::<1170>::process_state(&mut parser, lookahead)
                }
                make_state!(1171, lookahead) => {
                    State::<1171>::process_state(&mut parser, lookahead)
                }
                make_state!(1172, lookahead) => {
                    State::<1172>::process_state(&mut parser, lookahead)
                }
                make_state!(1173, lookahead) => {
                    State::<1173>::process_state(&mut parser, lookahead)
                }
                make_state!(1174, lookahead) => {
                    State::<1174>::process_state(&mut parser, lookahead)
                }
                make_state!(1175, lookahead) => {
                    State::<1175>::process_state(&mut parser, lookahead)
                }
                make_state!(1176, lookahead) => {
                    State::<1176>::process_state(&mut parser, lookahead)
                }
                make_state!(1177, lookahead) => {
                    State::<1177>::process_state(&mut parser, lookahead)
                }
                make_state!(1178, lookahead) => {
                    State::<1178>::process_state(&mut parser, lookahead)
                }
                make_state!(1179, lookahead) => {
                    State::<1179>::process_state(&mut parser, lookahead)
                }
                make_state!(1180, lookahead) => {
                    State::<1180>::process_state(&mut parser, lookahead)
                }
                make_state!(1181, lookahead) => {
                    State::<1181>::process_state(&mut parser, lookahead)
                }
                make_state!(1182, lookahead) => {
                    State::<1182>::process_state(&mut parser, lookahead)
                }
                make_state!(1183, lookahead) => {
                    State::<1183>::process_state(&mut parser, lookahead)
                }
                make_state!(1184, lookahead) => {
                    State::<1184>::process_state(&mut parser, lookahead)
                }
                make_state!(1185, lookahead) => {
                    State::<1185>::process_state(&mut parser, lookahead)
                }
                make_state!(1186, lookahead) => {
                    State::<1186>::process_state(&mut parser, lookahead)
                }
                make_state!(1187, lookahead) => {
                    State::<1187>::process_state(&mut parser, lookahead)
                }
                make_state!(1188, lookahead) => {
                    State::<1188>::process_state(&mut parser, lookahead)
                }
                make_state!(1189, lookahead) => {
                    State::<1189>::process_state(&mut parser, lookahead)
                }
                make_state!(1190, lookahead) => {
                    State::<1190>::process_state(&mut parser, lookahead)
                }
                make_state!(1191, lookahead) => {
                    State::<1191>::process_state(&mut parser, lookahead)
                }
                make_state!(1192, lookahead) => {
                    State::<1192>::process_state(&mut parser, lookahead)
                }
                make_state!(1193, lookahead) => {
                    State::<1193>::process_state(&mut parser, lookahead)
                }
                make_state!(1194, lookahead) => {
                    State::<1194>::process_state(&mut parser, lookahead)
                }
                make_state!(1195, lookahead) => {
                    State::<1195>::process_state(&mut parser, lookahead)
                }
                make_state!(1196, lookahead) => {
                    State::<1196>::process_state(&mut parser, lookahead)
                }
                make_state!(1197, lookahead) => {
                    State::<1197>::process_state(&mut parser, lookahead)
                }
                make_state!(1198, lookahead) => {
                    State::<1198>::process_state(&mut parser, lookahead)
                }
                make_state!(1199, lookahead) => {
                    State::<1199>::process_state(&mut parser, lookahead)
                }
                make_state!(1200, lookahead) => {
                    State::<1200>::process_state(&mut parser, lookahead)
                }
                make_state!(1201, lookahead) => {
                    State::<1201>::process_state(&mut parser, lookahead)
                }
                make_state!(1202, lookahead) => {
                    State::<1202>::process_state(&mut parser, lookahead)
                }
                make_state!(1203, lookahead) => {
                    State::<1203>::process_state(&mut parser, lookahead)
                }
                make_state!(1204, lookahead) => {
                    State::<1204>::process_state(&mut parser, lookahead)
                }
                make_state!(1205, lookahead) => {
                    State::<1205>::process_state(&mut parser, lookahead)
                }
                make_state!(1206, lookahead) => {
                    State::<1206>::process_state(&mut parser, lookahead)
                }
                make_state!(1207, lookahead) => {
                    State::<1207>::process_state(&mut parser, lookahead)
                }
                make_state!(1208, lookahead) => {
                    State::<1208>::process_state(&mut parser, lookahead)
                }
                make_state!(1209, lookahead) => {
                    State::<1209>::process_state(&mut parser, lookahead)
                }
                make_state!(1210, lookahead) => {
                    State::<1210>::process_state(&mut parser, lookahead)
                }
                make_state!(1211, lookahead) => {
                    State::<1211>::process_state(&mut parser, lookahead)
                }
                make_state!(1212, lookahead) => {
                    State::<1212>::process_state(&mut parser, lookahead)
                }
                make_state!(1213, lookahead) => {
                    State::<1213>::process_state(&mut parser, lookahead)
                }
                make_state!(1214, lookahead) => {
                    State::<1214>::process_state(&mut parser, lookahead)
                }
                make_state!(1215, lookahead) => {
                    State::<1215>::process_state(&mut parser, lookahead)
                }
                make_state!(1216, lookahead) => {
                    State::<1216>::process_state(&mut parser, lookahead)
                }
                make_state!(1217, lookahead) => {
                    State::<1217>::process_state(&mut parser, lookahead)
                }
                make_state!(1218, lookahead) => {
                    State::<1218>::process_state(&mut parser, lookahead)
                }
                make_state!(1219, lookahead) => {
                    State::<1219>::process_state(&mut parser, lookahead)
                }
                make_state!(1220, lookahead) => {
                    State::<1220>::process_state(&mut parser, lookahead)
                }
                make_state!(1221, lookahead) => {
                    State::<1221>::process_state(&mut parser, lookahead)
                }
                make_state!(1222, lookahead) => {
                    State::<1222>::process_state(&mut parser, lookahead)
                }
                make_state!(1223, lookahead) => {
                    State::<1223>::process_state(&mut parser, lookahead)
                }
                make_state!(1224, lookahead) => {
                    State::<1224>::process_state(&mut parser, lookahead)
                }
                make_state!(1225, lookahead) => {
                    State::<1225>::process_state(&mut parser, lookahead)
                }
                make_state!(1226, lookahead) => {
                    State::<1226>::process_state(&mut parser, lookahead)
                }
                make_state!(1227, lookahead) => {
                    State::<1227>::process_state(&mut parser, lookahead)
                }
                make_state!(1228, lookahead) => {
                    State::<1228>::process_state(&mut parser, lookahead)
                }
                make_state!(1229, lookahead) => {
                    State::<1229>::process_state(&mut parser, lookahead)
                }
                make_state!(1230, lookahead) => {
                    State::<1230>::process_state(&mut parser, lookahead)
                }
                make_state!(1231, lookahead) => {
                    State::<1231>::process_state(&mut parser, lookahead)
                }
                make_state!(1232, lookahead) => {
                    State::<1232>::process_state(&mut parser, lookahead)
                }
                make_state!(1233, lookahead) => {
                    State::<1233>::process_state(&mut parser, lookahead)
                }
                make_state!(1234, lookahead) => {
                    State::<1234>::process_state(&mut parser, lookahead)
                }
                make_state!(1235, lookahead) => {
                    State::<1235>::process_state(&mut parser, lookahead)
                }
                make_state!(1236, lookahead) => {
                    State::<1236>::process_state(&mut parser, lookahead)
                }
                make_state!(1237, lookahead) => {
                    State::<1237>::process_state(&mut parser, lookahead)
                }
                make_state!(1238, lookahead) => {
                    State::<1238>::process_state(&mut parser, lookahead)
                }
                make_state!(1239, lookahead) => {
                    State::<1239>::process_state(&mut parser, lookahead)
                }
                make_state!(1240, lookahead) => {
                    State::<1240>::process_state(&mut parser, lookahead)
                }
                make_state!(1241, lookahead) => {
                    State::<1241>::process_state(&mut parser, lookahead)
                }
                make_state!(1242, lookahead) => {
                    State::<1242>::process_state(&mut parser, lookahead)
                }
                make_state!(1243, lookahead) => {
                    State::<1243>::process_state(&mut parser, lookahead)
                }
                make_state!(1244, lookahead) => {
                    State::<1244>::process_state(&mut parser, lookahead)
                }
                make_state!(1245, lookahead) => {
                    State::<1245>::process_state(&mut parser, lookahead)
                }
                make_state!(1246, lookahead) => {
                    State::<1246>::process_state(&mut parser, lookahead)
                }
                make_state!(1247, lookahead) => {
                    State::<1247>::process_state(&mut parser, lookahead)
                }
                make_state!(1248, lookahead) => {
                    State::<1248>::process_state(&mut parser, lookahead)
                }
                make_state!(1249, lookahead) => {
                    State::<1249>::process_state(&mut parser, lookahead)
                }
                make_state!(1250, lookahead) => {
                    State::<1250>::process_state(&mut parser, lookahead)
                }
                make_state!(1251, lookahead) => {
                    State::<1251>::process_state(&mut parser, lookahead)
                }
                make_state!(1252, lookahead) => {
                    State::<1252>::process_state(&mut parser, lookahead)
                }
                make_state!(1253, lookahead) => {
                    State::<1253>::process_state(&mut parser, lookahead)
                }
                make_state!(1254, lookahead) => {
                    State::<1254>::process_state(&mut parser, lookahead)
                }
                make_state!(1255, lookahead) => {
                    State::<1255>::process_state(&mut parser, lookahead)
                }
                make_state!(1256, lookahead) => {
                    State::<1256>::process_state(&mut parser, lookahead)
                }
                make_state!(1257, lookahead) => {
                    State::<1257>::process_state(&mut parser, lookahead)
                }
                make_state!(1258, lookahead) => {
                    State::<1258>::process_state(&mut parser, lookahead)
                }
                make_state!(1259, lookahead) => {
                    State::<1259>::process_state(&mut parser, lookahead)
                }
                make_state!(1260, lookahead) => {
                    State::<1260>::process_state(&mut parser, lookahead)
                }
                make_state!(1261, lookahead) => {
                    State::<1261>::process_state(&mut parser, lookahead)
                }
                make_state!(1262, lookahead) => {
                    State::<1262>::process_state(&mut parser, lookahead)
                }
                make_state!(1263, lookahead) => {
                    State::<1263>::process_state(&mut parser, lookahead)
                }
                make_state!(1264, lookahead) => {
                    State::<1264>::process_state(&mut parser, lookahead)
                }
                make_state!(1265, lookahead) => {
                    State::<1265>::process_state(&mut parser, lookahead)
                }
                make_state!(1266, lookahead) => {
                    State::<1266>::process_state(&mut parser, lookahead)
                }
                make_state!(1267, lookahead) => {
                    State::<1267>::process_state(&mut parser, lookahead)
                }
                make_state!(1268, lookahead) => {
                    State::<1268>::process_state(&mut parser, lookahead)
                }
                make_state!(1269, lookahead) => {
                    State::<1269>::process_state(&mut parser, lookahead)
                }
                make_state!(1270, lookahead) => {
                    State::<1270>::process_state(&mut parser, lookahead)
                }
                make_state!(1271, lookahead) => {
                    State::<1271>::process_state(&mut parser, lookahead)
                }
                make_state!(1272, lookahead) => {
                    State::<1272>::process_state(&mut parser, lookahead)
                }
                make_state!(1273, lookahead) => {
                    State::<1273>::process_state(&mut parser, lookahead)
                }
                make_state!(1274, lookahead) => {
                    State::<1274>::process_state(&mut parser, lookahead)
                }
                make_state!(1275, lookahead) => {
                    State::<1275>::process_state(&mut parser, lookahead)
                }
                make_state!(1276, lookahead) => {
                    State::<1276>::process_state(&mut parser, lookahead)
                }
                make_state!(1277, lookahead) => {
                    State::<1277>::process_state(&mut parser, lookahead)
                }
                make_state!(1278, lookahead) => {
                    State::<1278>::process_state(&mut parser, lookahead)
                }
                make_state!(1279, lookahead) => {
                    State::<1279>::process_state(&mut parser, lookahead)
                }
                make_state!(1280, lookahead) => {
                    State::<1280>::process_state(&mut parser, lookahead)
                }
                make_state!(1281, lookahead) => {
                    State::<1281>::process_state(&mut parser, lookahead)
                }
                make_state!(1282, lookahead) => {
                    State::<1282>::process_state(&mut parser, lookahead)
                }
                make_state!(1283, lookahead) => {
                    State::<1283>::process_state(&mut parser, lookahead)
                }
                make_state!(1284, lookahead) => {
                    State::<1284>::process_state(&mut parser, lookahead)
                }
                make_state!(1285, lookahead) => {
                    State::<1285>::process_state(&mut parser, lookahead)
                }
                make_state!(1286, lookahead) => {
                    State::<1286>::process_state(&mut parser, lookahead)
                }
                make_state!(1287, lookahead) => {
                    State::<1287>::process_state(&mut parser, lookahead)
                }
                make_state!(1288, lookahead) => {
                    State::<1288>::process_state(&mut parser, lookahead)
                }
                make_state!(1289, lookahead) => {
                    State::<1289>::process_state(&mut parser, lookahead)
                }
                make_state!(1290, lookahead) => {
                    State::<1290>::process_state(&mut parser, lookahead)
                }
                make_state!(1291, lookahead) => {
                    State::<1291>::process_state(&mut parser, lookahead)
                }
                make_state!(1292, lookahead) => {
                    State::<1292>::process_state(&mut parser, lookahead)
                }
                make_state!(1293, lookahead) => {
                    State::<1293>::process_state(&mut parser, lookahead)
                }
                make_state!(1294, lookahead) => {
                    State::<1294>::process_state(&mut parser, lookahead)
                }
                make_state!(1295, lookahead) => {
                    State::<1295>::process_state(&mut parser, lookahead)
                }
                make_state!(1296, lookahead) => {
                    State::<1296>::process_state(&mut parser, lookahead)
                }
                make_state!(1297, lookahead) => {
                    State::<1297>::process_state(&mut parser, lookahead)
                }
                make_state!(1298, lookahead) => {
                    State::<1298>::process_state(&mut parser, lookahead)
                }
                make_state!(1299, lookahead) => {
                    State::<1299>::process_state(&mut parser, lookahead)
                }
                make_state!(1300, lookahead) => {
                    State::<1300>::process_state(&mut parser, lookahead)
                }
                make_state!(1301, lookahead) => {
                    State::<1301>::process_state(&mut parser, lookahead)
                }
                make_state!(1302, lookahead) => {
                    State::<1302>::process_state(&mut parser, lookahead)
                }
                make_state!(1303, lookahead) => {
                    State::<1303>::process_state(&mut parser, lookahead)
                }
                make_state!(1304, lookahead) => {
                    State::<1304>::process_state(&mut parser, lookahead)
                }
                make_state!(1305, lookahead) => {
                    State::<1305>::process_state(&mut parser, lookahead)
                }
                make_state!(1306, lookahead) => {
                    State::<1306>::process_state(&mut parser, lookahead)
                }
                make_state!(1307, lookahead) => {
                    State::<1307>::process_state(&mut parser, lookahead)
                }
                make_state!(1308, lookahead) => {
                    State::<1308>::process_state(&mut parser, lookahead)
                }
                make_state!(1309, lookahead) => {
                    State::<1309>::process_state(&mut parser, lookahead)
                }
                make_state!(1310, lookahead) => {
                    State::<1310>::process_state(&mut parser, lookahead)
                }
                make_state!(1311, lookahead) => {
                    State::<1311>::process_state(&mut parser, lookahead)
                }
                make_state!(1312, lookahead) => {
                    State::<1312>::process_state(&mut parser, lookahead)
                }
                make_state!(1313, lookahead) => {
                    State::<1313>::process_state(&mut parser, lookahead)
                }
                make_state!(1314, lookahead) => {
                    State::<1314>::process_state(&mut parser, lookahead)
                }
                make_state!(1315, lookahead) => {
                    State::<1315>::process_state(&mut parser, lookahead)
                }
                make_state!(1316, lookahead) => {
                    State::<1316>::process_state(&mut parser, lookahead)
                }
                make_state!(1317, lookahead) => {
                    State::<1317>::process_state(&mut parser, lookahead)
                }
                make_state!(1318, lookahead) => {
                    State::<1318>::process_state(&mut parser, lookahead)
                }
                make_state!(1319, lookahead) => {
                    State::<1319>::process_state(&mut parser, lookahead)
                }
                make_state!(1320, lookahead) => {
                    State::<1320>::process_state(&mut parser, lookahead)
                }
                make_state!(1321, lookahead) => {
                    State::<1321>::process_state(&mut parser, lookahead)
                }
                make_state!(1322, lookahead) => {
                    State::<1322>::process_state(&mut parser, lookahead)
                }
                make_state!(1323, lookahead) => {
                    State::<1323>::process_state(&mut parser, lookahead)
                }
                make_state!(1324, lookahead) => {
                    State::<1324>::process_state(&mut parser, lookahead)
                }
                make_state!(1325, lookahead) => {
                    State::<1325>::process_state(&mut parser, lookahead)
                }
                make_state!(1326, lookahead) => {
                    State::<1326>::process_state(&mut parser, lookahead)
                }
                make_state!(1327, lookahead) => {
                    State::<1327>::process_state(&mut parser, lookahead)
                }
                make_state!(1328, lookahead) => {
                    State::<1328>::process_state(&mut parser, lookahead)
                }
                make_state!(1329, lookahead) => {
                    State::<1329>::process_state(&mut parser, lookahead)
                }
                make_state!(1330, lookahead) => {
                    State::<1330>::process_state(&mut parser, lookahead)
                }
                make_state!(1331, lookahead) => {
                    State::<1331>::process_state(&mut parser, lookahead)
                }
                make_state!(1332, lookahead) => {
                    State::<1332>::process_state(&mut parser, lookahead)
                }
                make_state!(1333, lookahead) => {
                    State::<1333>::process_state(&mut parser, lookahead)
                }
                make_state!(1334, lookahead) => {
                    State::<1334>::process_state(&mut parser, lookahead)
                }
                make_state!(1335, lookahead) => {
                    State::<1335>::process_state(&mut parser, lookahead)
                }
                make_state!(1336, lookahead) => {
                    State::<1336>::process_state(&mut parser, lookahead)
                }
                make_state!(1337, lookahead) => {
                    State::<1337>::process_state(&mut parser, lookahead)
                }
                make_state!(1338, lookahead) => {
                    State::<1338>::process_state(&mut parser, lookahead)
                }
                make_state!(1339, lookahead) => {
                    State::<1339>::process_state(&mut parser, lookahead)
                }
                make_state!(1340, lookahead) => {
                    State::<1340>::process_state(&mut parser, lookahead)
                }
                make_state!(1341, lookahead) => {
                    State::<1341>::process_state(&mut parser, lookahead)
                }
                make_state!(1342, lookahead) => {
                    State::<1342>::process_state(&mut parser, lookahead)
                }
                make_state!(1343, lookahead) => {
                    State::<1343>::process_state(&mut parser, lookahead)
                }
                make_state!(1344, lookahead) => {
                    State::<1344>::process_state(&mut parser, lookahead)
                }
                make_state!(1345, lookahead) => {
                    State::<1345>::process_state(&mut parser, lookahead)
                }
                make_state!(1346, lookahead) => {
                    State::<1346>::process_state(&mut parser, lookahead)
                }
                make_state!(1347, lookahead) => {
                    State::<1347>::process_state(&mut parser, lookahead)
                }
                make_state!(1348, lookahead) => {
                    State::<1348>::process_state(&mut parser, lookahead)
                }
                make_state!(1349, lookahead) => {
                    State::<1349>::process_state(&mut parser, lookahead)
                }
                make_state!(1350, lookahead) => {
                    State::<1350>::process_state(&mut parser, lookahead)
                }
                make_state!(1351, lookahead) => {
                    State::<1351>::process_state(&mut parser, lookahead)
                }
                make_state!(1352, lookahead) => {
                    State::<1352>::process_state(&mut parser, lookahead)
                }
                make_state!(1353, lookahead) => {
                    State::<1353>::process_state(&mut parser, lookahead)
                }
                make_state!(1354, lookahead) => {
                    State::<1354>::process_state(&mut parser, lookahead)
                }
                make_state!(1355, lookahead) => {
                    State::<1355>::process_state(&mut parser, lookahead)
                }
                make_state!(1356, lookahead) => {
                    State::<1356>::process_state(&mut parser, lookahead)
                }
                make_state!(1357, lookahead) => {
                    State::<1357>::process_state(&mut parser, lookahead)
                }
                make_state!(1358, lookahead) => {
                    State::<1358>::process_state(&mut parser, lookahead)
                }
                make_state!(1359, lookahead) => {
                    State::<1359>::process_state(&mut parser, lookahead)
                }
                make_state!(1360, lookahead) => {
                    State::<1360>::process_state(&mut parser, lookahead)
                }
                make_state!(1361, lookahead) => {
                    State::<1361>::process_state(&mut parser, lookahead)
                }
                make_state!(1362, lookahead) => {
                    State::<1362>::process_state(&mut parser, lookahead)
                }
                make_state!(1363, lookahead) => {
                    State::<1363>::process_state(&mut parser, lookahead)
                }
                make_state!(1364, lookahead) => {
                    State::<1364>::process_state(&mut parser, lookahead)
                }
                make_state!(1365, lookahead) => {
                    State::<1365>::process_state(&mut parser, lookahead)
                }
                make_state!(1366, lookahead) => {
                    State::<1366>::process_state(&mut parser, lookahead)
                }
                make_state!(1367, lookahead) => {
                    State::<1367>::process_state(&mut parser, lookahead)
                }
                make_state!(1368, lookahead) => {
                    State::<1368>::process_state(&mut parser, lookahead)
                }
                make_state!(1369, lookahead) => {
                    State::<1369>::process_state(&mut parser, lookahead)
                }
                make_state!(1370, lookahead) => {
                    State::<1370>::process_state(&mut parser, lookahead)
                }
                make_state!(1371, lookahead) => {
                    State::<1371>::process_state(&mut parser, lookahead)
                }
                make_state!(1372, lookahead) => {
                    State::<1372>::process_state(&mut parser, lookahead)
                }
                make_state!(1373, lookahead) => {
                    State::<1373>::process_state(&mut parser, lookahead)
                }
                make_state!(1374, lookahead) => {
                    State::<1374>::process_state(&mut parser, lookahead)
                }
                make_state!(1375, lookahead) => {
                    State::<1375>::process_state(&mut parser, lookahead)
                }
                make_state!(1376, lookahead) => {
                    State::<1376>::process_state(&mut parser, lookahead)
                }
                make_state!(1377, lookahead) => {
                    State::<1377>::process_state(&mut parser, lookahead)
                }
                make_state!(1378, lookahead) => {
                    State::<1378>::process_state(&mut parser, lookahead)
                }
                make_state!(1379, lookahead) => {
                    State::<1379>::process_state(&mut parser, lookahead)
                }
                make_state!(1380, lookahead) => {
                    State::<1380>::process_state(&mut parser, lookahead)
                }
                make_state!(1381, lookahead) => {
                    State::<1381>::process_state(&mut parser, lookahead)
                }
                make_state!(1382, lookahead) => {
                    State::<1382>::process_state(&mut parser, lookahead)
                }
                make_state!(1383, lookahead) => {
                    State::<1383>::process_state(&mut parser, lookahead)
                }
                make_state!(1384, lookahead) => {
                    State::<1384>::process_state(&mut parser, lookahead)
                }
                make_state!(1385, lookahead) => {
                    State::<1385>::process_state(&mut parser, lookahead)
                }
                make_state!(1386, lookahead) => {
                    State::<1386>::process_state(&mut parser, lookahead)
                }
                make_state!(1387, lookahead) => {
                    State::<1387>::process_state(&mut parser, lookahead)
                }
                make_state!(1388, lookahead) => {
                    State::<1388>::process_state(&mut parser, lookahead)
                }
                make_state!(1389, lookahead) => {
                    State::<1389>::process_state(&mut parser, lookahead)
                }
                make_state!(1390, lookahead) => {
                    State::<1390>::process_state(&mut parser, lookahead)
                }
                make_state!(1391, lookahead) => {
                    State::<1391>::process_state(&mut parser, lookahead)
                }
                make_state!(1392, lookahead) => {
                    State::<1392>::process_state(&mut parser, lookahead)
                }
                make_state!(1393, lookahead) => {
                    State::<1393>::process_state(&mut parser, lookahead)
                }
                make_state!(1394, lookahead) => {
                    State::<1394>::process_state(&mut parser, lookahead)
                }
                make_state!(1395, lookahead) => {
                    State::<1395>::process_state(&mut parser, lookahead)
                }
                make_state!(1396, lookahead) => {
                    State::<1396>::process_state(&mut parser, lookahead)
                }
                make_state!(1397, lookahead) => {
                    State::<1397>::process_state(&mut parser, lookahead)
                }
                make_state!(1398, lookahead) => {
                    State::<1398>::process_state(&mut parser, lookahead)
                }
                make_state!(1399, lookahead) => {
                    State::<1399>::process_state(&mut parser, lookahead)
                }
                make_state!(1400, lookahead) => {
                    State::<1400>::process_state(&mut parser, lookahead)
                }
                make_state!(1401, lookahead) => {
                    State::<1401>::process_state(&mut parser, lookahead)
                }
                make_state!(1402, lookahead) => {
                    State::<1402>::process_state(&mut parser, lookahead)
                }
                make_state!(1403, lookahead) => {
                    State::<1403>::process_state(&mut parser, lookahead)
                }
                make_state!(1404, lookahead) => {
                    State::<1404>::process_state(&mut parser, lookahead)
                }
                make_state!(1405, lookahead) => {
                    State::<1405>::process_state(&mut parser, lookahead)
                }
                make_state!(1406, lookahead) => {
                    State::<1406>::process_state(&mut parser, lookahead)
                }
                make_state!(1407, lookahead) => {
                    State::<1407>::process_state(&mut parser, lookahead)
                }
                make_state!(1408, lookahead) => {
                    State::<1408>::process_state(&mut parser, lookahead)
                }
                make_state!(1409, lookahead) => {
                    State::<1409>::process_state(&mut parser, lookahead)
                }
                make_state!(1410, lookahead) => {
                    State::<1410>::process_state(&mut parser, lookahead)
                }
                make_state!(1411, lookahead) => {
                    State::<1411>::process_state(&mut parser, lookahead)
                }
                make_state!(1412, lookahead) => {
                    State::<1412>::process_state(&mut parser, lookahead)
                }
                make_state!(1413, lookahead) => {
                    State::<1413>::process_state(&mut parser, lookahead)
                }
                make_state!(1414, lookahead) => {
                    State::<1414>::process_state(&mut parser, lookahead)
                }
                make_state!(1415, lookahead) => {
                    State::<1415>::process_state(&mut parser, lookahead)
                }
                make_state!(1416, lookahead) => {
                    State::<1416>::process_state(&mut parser, lookahead)
                }
                make_state!(1417, lookahead) => {
                    State::<1417>::process_state(&mut parser, lookahead)
                }
                make_state!(1418, lookahead) => {
                    State::<1418>::process_state(&mut parser, lookahead)
                }
                make_state!(1419, lookahead) => {
                    State::<1419>::process_state(&mut parser, lookahead)
                }
                make_state!(1420, lookahead) => {
                    State::<1420>::process_state(&mut parser, lookahead)
                }
                make_state!(1421, lookahead) => {
                    State::<1421>::process_state(&mut parser, lookahead)
                }
                make_state!(1422, lookahead) => {
                    State::<1422>::process_state(&mut parser, lookahead)
                }
                make_state!(1423, lookahead) => {
                    State::<1423>::process_state(&mut parser, lookahead)
                }
                make_state!(1424, lookahead) => {
                    State::<1424>::process_state(&mut parser, lookahead)
                }
                make_state!(1425, lookahead) => {
                    State::<1425>::process_state(&mut parser, lookahead)
                }
                make_state!(1426, lookahead) => {
                    State::<1426>::process_state(&mut parser, lookahead)
                }
                make_state!(1427, lookahead) => {
                    State::<1427>::process_state(&mut parser, lookahead)
                }
                make_state!(1428, lookahead) => {
                    State::<1428>::process_state(&mut parser, lookahead)
                }
                make_state!(1429, lookahead) => {
                    State::<1429>::process_state(&mut parser, lookahead)
                }
                make_state!(1430, lookahead) => {
                    State::<1430>::process_state(&mut parser, lookahead)
                }
                make_state!(1431, lookahead) => {
                    State::<1431>::process_state(&mut parser, lookahead)
                }
                make_state!(1432, lookahead) => {
                    State::<1432>::process_state(&mut parser, lookahead)
                }
                make_state!(1433, lookahead) => {
                    State::<1433>::process_state(&mut parser, lookahead)
                }
                make_state!(1434, lookahead) => {
                    State::<1434>::process_state(&mut parser, lookahead)
                }
                make_state!(1435, lookahead) => {
                    State::<1435>::process_state(&mut parser, lookahead)
                }
                make_state!(1436, lookahead) => {
                    State::<1436>::process_state(&mut parser, lookahead)
                }
                make_state!(1437, lookahead) => {
                    State::<1437>::process_state(&mut parser, lookahead)
                }
                make_state!(1438, lookahead) => {
                    State::<1438>::process_state(&mut parser, lookahead)
                }
                make_state!(1439, lookahead) => {
                    State::<1439>::process_state(&mut parser, lookahead)
                }
                make_state!(1440, lookahead) => {
                    State::<1440>::process_state(&mut parser, lookahead)
                }
                make_state!(1441, lookahead) => {
                    State::<1441>::process_state(&mut parser, lookahead)
                }
                make_state!(1442, lookahead) => {
                    State::<1442>::process_state(&mut parser, lookahead)
                }
                make_state!(1443, lookahead) => {
                    State::<1443>::process_state(&mut parser, lookahead)
                }
                make_state!(1444, lookahead) => {
                    State::<1444>::process_state(&mut parser, lookahead)
                }
                make_state!(1445, lookahead) => {
                    State::<1445>::process_state(&mut parser, lookahead)
                }
                make_state!(1446, lookahead) => {
                    State::<1446>::process_state(&mut parser, lookahead)
                }
                make_state!(1447, lookahead) => {
                    State::<1447>::process_state(&mut parser, lookahead)
                }
                make_state!(1448, lookahead) => {
                    State::<1448>::process_state(&mut parser, lookahead)
                }
                make_state!(1449, lookahead) => {
                    State::<1449>::process_state(&mut parser, lookahead)
                }
                make_state!(1450, lookahead) => {
                    State::<1450>::process_state(&mut parser, lookahead)
                }
                make_state!(1451, lookahead) => {
                    State::<1451>::process_state(&mut parser, lookahead)
                }
                make_state!(1452, lookahead) => {
                    State::<1452>::process_state(&mut parser, lookahead)
                }
                make_state!(1453, lookahead) => {
                    State::<1453>::process_state(&mut parser, lookahead)
                }
                make_state!(1454, lookahead) => {
                    State::<1454>::process_state(&mut parser, lookahead)
                }
                make_state!(1455, lookahead) => {
                    State::<1455>::process_state(&mut parser, lookahead)
                }
                make_state!(1456, lookahead) => {
                    State::<1456>::process_state(&mut parser, lookahead)
                }
                make_state!(1457, lookahead) => {
                    State::<1457>::process_state(&mut parser, lookahead)
                }
                make_state!(1458, lookahead) => {
                    State::<1458>::process_state(&mut parser, lookahead)
                }
                make_state!(1459, lookahead) => {
                    State::<1459>::process_state(&mut parser, lookahead)
                }
                make_state!(1460, lookahead) => {
                    State::<1460>::process_state(&mut parser, lookahead)
                }
                make_state!(1461, lookahead) => {
                    State::<1461>::process_state(&mut parser, lookahead)
                }
                make_state!(1462, lookahead) => {
                    State::<1462>::process_state(&mut parser, lookahead)
                }
                make_state!(1463, lookahead) => {
                    State::<1463>::process_state(&mut parser, lookahead)
                }
                make_state!(1464, lookahead) => {
                    State::<1464>::process_state(&mut parser, lookahead)
                }
                make_state!(1465, lookahead) => {
                    State::<1465>::process_state(&mut parser, lookahead)
                }
                make_state!(1466, lookahead) => {
                    State::<1466>::process_state(&mut parser, lookahead)
                }
                make_state!(1467, lookahead) => {
                    State::<1467>::process_state(&mut parser, lookahead)
                }
                make_state!(1468, lookahead) => {
                    State::<1468>::process_state(&mut parser, lookahead)
                }
                make_state!(1469, lookahead) => {
                    State::<1469>::process_state(&mut parser, lookahead)
                }
                make_state!(1470, lookahead) => {
                    State::<1470>::process_state(&mut parser, lookahead)
                }
                make_state!(1471, lookahead) => {
                    State::<1471>::process_state(&mut parser, lookahead)
                }
                // Errors
                _ => Err(Error::Unimplemented),
            }?;
        }

        if let Some(chunk) = parser.reduction.take() {
            chunk.map_err(Error::from)
        } else {
            Err(Error::Accept)
        }
    }

    fn shift(&mut self, next_state: usize) -> Result<(), Error> {
        let Some(Ok(token)) = self.lexeme_stream.next() else {
            unreachable!();
        };
        self.states.push(next_state);
        self.stack.push(token);
        Ok(())
    }

    fn goto(&mut self, next_state: usize) -> Result<(), Error> {
        let Some(Ok(token)) = self.reduction.take() else {
            unreachable!();
        };
        self.states.push(next_state);
        self.stack.push(token);
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn reduce<const PRODUCTION: usize>(&mut self) -> Result<(), Error> {
        match PRODUCTION {
            0 => make_reduction_push!(0, self, Chunk, 1, Block),
            1 => make_reduction_push!(1, self, Block, 2, BlockStat, BlockRetstat),
            2 => make_reduction_push!(2, self, BlockStat),
            3 => make_reduction_push!(3, self, BlockStat, 2, Stat, BlockStat),
            4 => make_reduction_push!(4, self, BlockRetstat),
            5 => make_reduction_push!(5, self, BlockRetstat, 1, Retstat),
            6 => make_reduction_push!(6, self, Stat, 1, SemiColon),
            7 => make_reduction_push!(7, self, Stat, 3, Varlist, Assign, Explist),
            8 => make_reduction_push!(8, self, Stat, 1, Functioncall),
            9 => make_reduction_push!(9, self, Stat, 1, Label),
            10 => make_reduction_push!(10, self, Stat, 1, Break),
            11 => make_reduction_push!(11, self, Stat, 2, Goto, Name),
            12 => make_reduction_push!(12, self, Stat, 3, Do, Block, End),
            13 => make_reduction_push!(13, self, Stat, 5, While, Exp, Do, Block, End),
            14 => make_reduction_push!(14, self, Stat, 4, Repeat, Block, Until, Exp),
            15 => {
                make_reduction_push!(15, self, Stat, 6, If, Exp, Then, Block, StatIf, End)
            }
            16 => make_reduction_push!(16, self, StatIf),
            17 => make_reduction_push!(17, self, StatIf, 5, Elseif, Exp, Then, Block, StatIf),
            18 => make_reduction_push!(18, self, StatIf, 2, Else, Block),
            19 => {
                make_reduction_push!(
                    19, self, Stat, 10, For, Name, Assign, Exp, Comma, Exp, StatForexp, Do, Block,
                    End
                )
            }
            20 => make_reduction_push!(20, self, StatForexp),
            21 => make_reduction_push!(21, self, StatForexp, 2, Comma, Exp),
            22 => {
                make_reduction_push!(22, self, Stat, 7, For, Namelist, In, Explist, Do, Block, End)
            }
            23 => make_reduction_push!(23, self, Stat, 3, Function, Funcname, Funcbody),
            24 => make_reduction_push!(24, self, Stat, 4, Local, Function, Name, Funcbody),
            25 => make_reduction_push!(25, self, Stat, 3, Local, Attnamelist, StatAttexplist),
            26 => make_reduction_push!(26, self, StatAttexplist),
            27 => make_reduction_push!(27, self, StatAttexplist, 2, Assign, Explist),
            28 => make_reduction_push!(28, self, Attnamelist, 3, Name, Attrib, AttnamelistCont),
            29 => make_reduction_push!(29, self, AttnamelistCont),
            30 => {
                make_reduction_push!(
                    30,
                    self,
                    AttnamelistCont,
                    4,
                    Comma,
                    Name,
                    Attrib,
                    AttnamelistCont
                )
            }
            31 => make_reduction_push!(31, self, Attrib),
            32 => make_reduction_push!(32, self, Attrib, 3, Less, Name, Greater),
            33 => make_reduction_push!(33, self, Retstat, 3, Return, RetstatExplist, RetstatEnd),
            34 => make_reduction_push!(34, self, RetstatExplist),
            35 => make_reduction_push!(35, self, RetstatExplist, 1, Explist),
            36 => make_reduction_push!(36, self, RetstatEnd),
            37 => make_reduction_push!(37, self, RetstatEnd, 1, SemiColon),
            38 => make_reduction_push!(38, self, Label, 3, DoubleColon, Name, DoubleColon),
            39 => make_reduction_push!(39, self, Funcname, 3, Name, FuncnameCont, FuncnameEnd),
            40 => make_reduction_push!(40, self, FuncnameCont),
            41 => make_reduction_push!(41, self, FuncnameCont, 3, Dot, Name, FuncnameCont),
            42 => make_reduction_push!(42, self, FuncnameEnd),
            43 => make_reduction_push!(43, self, FuncnameEnd, 2, Colon, Name),
            44 => make_reduction_push!(44, self, Varlist, 2, Var, VarlistCont),
            45 => make_reduction_push!(45, self, VarlistCont),
            46 => make_reduction_push!(46, self, VarlistCont, 3, Comma, Var, VarlistCont),
            47 => make_reduction_push!(47, self, Var, 1, Name),
            48 => make_reduction_push!(48, self, Var, 4, Prefixexp, LSquare, Exp, RSquare),
            49 => make_reduction_push!(49, self, Var, 3, Prefixexp, Dot, Name),
            50 => make_reduction_push!(50, self, Namelist, 2, Name, NamelistCont),
            51 => make_reduction_push!(51, self, NamelistCont),
            52 => make_reduction_push!(52, self, NamelistCont, 3, Comma, Name, NamelistCont),
            53 => make_reduction_push!(53, self, Explist, 2, Exp, ExplistCont),
            54 => make_reduction_push!(54, self, ExplistCont),
            55 => make_reduction_push!(55, self, ExplistCont, 3, Comma, Exp, ExplistCont),
            56 => make_reduction_push!(56, self, Exp, 1, Nil),
            57 => make_reduction_push!(57, self, Exp, 1, False),
            58 => make_reduction_push!(58, self, Exp, 1, True),
            59 => make_reduction_push!(59, self, Exp, 1, String),
            60 => make_reduction_push!(60, self, Exp, 1, Integer),
            61 => make_reduction_push!(61, self, Exp, 1, Float),
            62 => make_reduction_push!(62, self, Exp, 1, Dots),
            63 => make_reduction_push!(63, self, Exp, 1, Functiondef),
            64 => make_reduction_push!(64, self, Exp, 1, Prefixexp),
            65 => make_reduction_push!(65, self, Exp, 1, Tableconstructor),
            66 => make_reduction_push!(66, self, Exp, 3, Exp, Binop, Exp),
            67 => make_reduction_push!(67, self, Exp, 2, Unop, Exp),
            68 => make_reduction_push!(68, self, Prefixexp, 1, Var),
            69 => make_reduction_push!(69, self, Prefixexp, 1, Functioncall),
            70 => make_reduction_push!(70, self, Prefixexp, 3, LParen, Exp, RParen),
            71 => make_reduction_push!(71, self, Functioncall, 2, Prefixexp, Args),
            72 => make_reduction_push!(72, self, Functioncall, 4, Prefixexp, Colon, Name, Args),
            73 => make_reduction_push!(73, self, Args, 3, LParen, ArgsExplist, RParen),
            74 => make_reduction_push!(74, self, ArgsExplist),
            75 => make_reduction_push!(75, self, ArgsExplist, 1, Explist),
            76 => make_reduction_push!(76, self, Args, 1, Tableconstructor),
            77 => make_reduction_push!(77, self, Args, 1, String),
            78 => make_reduction_push!(78, self, Functiondef, 2, Function, Funcbody),
            79 => {
                make_reduction_push!(
                    79,
                    self,
                    Funcbody,
                    5,
                    LParen,
                    FuncbodyParlist,
                    RParen,
                    Block,
                    End
                )
            }
            80 => make_reduction_push!(80, self, FuncbodyParlist),
            81 => make_reduction_push!(81, self, FuncbodyParlist, 1, Parlist),
            82 => make_reduction_push!(82, self, Parlist, 2, Namelist, ParlistCont),
            83 => make_reduction_push!(83, self, ParlistCont),
            84 => make_reduction_push!(84, self, ParlistCont, 3, Comma, Name, ParlistCont),
            85 => make_reduction_push!(85, self, ParlistCont, 2, Comma, Dots),
            86 => make_reduction_push!(86, self, Parlist, 1, Dots),
            87 => {
                make_reduction_push!(
                    87,
                    self,
                    Tableconstructor,
                    3,
                    LCurly,
                    TableconstructorFieldlist,
                    RCurly
                )
            }
            88 => make_reduction_push!(88, self, TableconstructorFieldlist),
            89 => make_reduction_push!(89, self, TableconstructorFieldlist, 1, Fieldlist),
            90 => make_reduction_push!(90, self, Fieldlist, 2, Field, FieldlistCont),
            91 => make_reduction_push!(91, self, FieldlistCont),
            92 => make_reduction_push!(92, self, FieldlistCont, 3, Fieldsep, Field, FieldlistCont),
            93 => make_reduction_push!(93, self, FieldlistCont, 1, Fieldsep),
            94 => make_reduction_push!(94, self, Field, 5, LSquare, Exp, RSquare, Assign, Exp),
            95 => make_reduction_push!(95, self, Field, 3, Name, Assign, Exp),
            96 => make_reduction_push!(96, self, Field, 1, Exp),
            97 => make_reduction_push!(97, self, Fieldsep, 1, Comma),
            98 => make_reduction_push!(98, self, Fieldsep, 1, SemiColon),
            99 => make_reduction_push!(99, self, Binop, 1, Or),
            100 => make_reduction_push!(100, self, Binop, 1, And),
            101 => make_reduction_push!(101, self, Binop, 1, Less),
            102 => make_reduction_push!(102, self, Binop, 1, Greater),
            103 => make_reduction_push!(103, self, Binop, 1, Leq),
            104 => make_reduction_push!(104, self, Binop, 1, Geq),
            105 => make_reduction_push!(105, self, Binop, 1, Eq),
            106 => make_reduction_push!(106, self, Binop, 1, Neq),
            107 => make_reduction_push!(107, self, Binop, 1, BitOr),
            108 => make_reduction_push!(108, self, Binop, 1, BitXor),
            109 => make_reduction_push!(109, self, Binop, 1, BitAnd),
            110 => make_reduction_push!(110, self, Binop, 1, ShiftL),
            111 => make_reduction_push!(111, self, Binop, 1, ShiftR),
            112 => make_reduction_push!(112, self, Binop, 1, Concat),
            113 => make_reduction_push!(113, self, Binop, 1, Add),
            114 => make_reduction_push!(114, self, Binop, 1, Sub),
            115 => make_reduction_push!(115, self, Binop, 1, Mul),
            116 => make_reduction_push!(116, self, Binop, 1, Div),
            117 => make_reduction_push!(117, self, Binop, 1, Idiv),
            118 => make_reduction_push!(118, self, Binop, 1, Mod),
            119 => make_reduction_push!(119, self, Binop, 1, Pow),
            120 => make_reduction_push!(120, self, Unop, 1, Not),
            121 => make_reduction_push!(121, self, Unop, 1, Len),
            122 => make_reduction_push!(122, self, Unop, 1, Sub),
            123 => make_reduction_push!(123, self, Unop, 1, BitXor),
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
