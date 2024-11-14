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

macro_rules! make_reduction_pop {
    ($parser:ident, $($var_name:ident),+) => {
        $(
            let Some($var_name) = $parser.stack.pop() else {
                unreachable!("Stack should have a {}.", stringify!($varname));
            };
            let Some(_) = $parser.states.pop() else {
                unreachable!("States shouldn't be empty.");
            };
        )+
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
                make_state!(0, Block) => parser.goto(1),
                make_state!(0, BlockStat) => parser.goto(2),
                make_state!(0, Stat) => parser.goto(3),
                make_state!(0, Var) => parser.goto(18),
                make_state!(0, Prefixexp) => parser.goto(20),
                make_state!(0, Functioncall) => parser.goto(16),
                // State 1
                make_state!(1, Eof) => {
                    break parser.accept();
                }
                // State 2
                make_state!(2, Eof) => parser.reduce::<4>()?,
                make_state!(2, BlockRetstat) => parser.goto(23),
                // State 3
                make_state!(3, Name) => parser.shift(19),
                make_state!(3, Eof) => parser.reduce::<2>()?,
                make_state!(3, BlockStat) => parser.goto(46),
                make_state!(3, Stat) => parser.goto(3),
                make_state!(3, Var) => parser.goto(18),
                make_state!(3, Prefixexp) => parser.goto(20),
                make_state!(3, Functioncall) => parser.goto(16),
                // State 16
                make_state!(16, Name) => parser.reduce::<8>()?,
                make_state!(16, Eof) => parser.reduce::<8>()?,
                // State 18
                make_state!(18, String) => parser.reduce::<91>()?,
                // State 19
                make_state!(19, String) => parser.reduce::<48>()?,
                // State 20
                make_state!(20, String) => parser.shift(82),
                make_state!(20, Args) => parser.goto(81),
                // State 23
                make_state!(23, Eof) => parser.reduce::<1>()?,
                // State 46
                make_state!(46, Eof) => parser.reduce::<3>()?,
                // State 81
                make_state!(81, Name) => parser.reduce::<94>()?,
                make_state!(81, Eof) => parser.reduce::<94>()?,
                // State 82
                make_state!(82, Name) => parser.reduce::<100>()?,
                make_state!(82, Eof) => parser.reduce::<100>()?,
                // Errors
                _ => {
                    break Err(Error::Unimplemented);
                }
            }
        }
    }

    fn accept(mut self) -> Result<Token<'a>, Error> {
        make_reduction_pop!(self, block);

        if !matches!(block.token_type, TokenType::Block) {
            log::error!(
                "Failed to accept.\n\tExpected: Block\n\tGot: {:?}",
                block.token_type
            );
            Err(Error::Accept)
        } else {
            Ok(Token {
                tokens: [block].to_vec(),
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
            1 => {
                make_reduction_pop!(self, blockretstat, blockstat);

                if !matches!(
                    (&blockstat, &blockretstat),
                    (
                        Token {
                            tokens: _,
                            token_type: TokenType::BlockStat,
                        },
                        Token {
                            tokens: _,
                            token_type: TokenType::BlockRetstat,
                        },
                    )
                ) {
                    log::error!(
                        "Failed to reduce.\n\tExpected: BlockStat BlockRetstat\n\tGot: {:?} {:?}",
                        blockstat.token_type,
                        blockretstat.token_type
                    );
                    Err(Error::Reduction)
                } else {
                    self.reduction.replace(Ok(Token {
                        tokens: [blockstat, blockretstat].to_vec(),
                        token_type: TokenType::Block,
                    }));
                    Ok(())
                }
            }
            2 => {
                self.reduction.replace(Ok(Token {
                    tokens: [].to_vec(),
                    token_type: TokenType::BlockStat,
                }));
                Ok(())
            }
            3 => {
                make_reduction_pop!(self, blockstat, stat);

                if !matches!(
                    (&stat, &blockstat),
                    (
                        Token {
                            tokens: _,
                            token_type: TokenType::Stat,
                        },
                        Token {
                            tokens: _,
                            token_type: TokenType::BlockStat,
                        },
                    )
                ) {
                    log::error!(
                        "Failed to reduce.\n\tExpected: Stat Blockstat\n\tGot: {:?} {:?}",
                        stat.token_type,
                        blockstat.token_type
                    );
                    Err(Error::Reduction)
                } else {
                    self.reduction.replace(Ok(Token {
                        tokens: [stat, blockstat].to_vec(),
                        token_type: TokenType::BlockStat,
                    }));
                    Ok(())
                }
            }
            4 => {
                self.reduction.replace(Ok(Token {
                    tokens: [].to_vec(),
                    token_type: TokenType::BlockRetstat,
                }));
                Ok(())
            }
            8 => {
                make_reduction_pop!(self, functioncall);

                if !matches!(functioncall.token_type, TokenType::Functioncall) {
                    log::error!(
                        "Failed to reduce.\n\tExpected: Functioncall\n\tGot: {:?}",
                        functioncall.token_type
                    );
                    Err(Error::Reduction)
                } else {
                    self.reduction.replace(Ok(Token {
                        tokens: [functioncall].to_vec(),
                        token_type: TokenType::Stat,
                    }));
                    Ok(())
                }
            }
            48 => {
                make_reduction_pop!(self, name);

                if !matches!(name.token_type, TokenType::Name(_)) {
                    log::error!(
                        "Failed to reduce.\n\tExpected: Name\n\tGot: {:?}",
                        name.token_type
                    );
                    Err(Error::Reduction)
                } else {
                    self.reduction.replace(Ok(Token {
                        tokens: [name].to_vec(),
                        token_type: TokenType::Var,
                    }));
                    Ok(())
                }
            }
            91 => {
                make_reduction_pop!(self, var);

                if !matches!(var.token_type, TokenType::Var) {
                    log::error!(
                        "Failed to reduce.\n\tExpected: Var\n\tGot: {:?}",
                        var.token_type
                    );
                    Err(Error::Reduction)
                } else {
                    self.reduction.replace(Ok(Token {
                        tokens: [var].to_vec(),
                        token_type: TokenType::Prefixexp,
                    }));
                    Ok(())
                }
            }
            94 => {
                make_reduction_pop!(self, args, prefixexp);

                if !matches!(
                    (&prefixexp, &args),
                    (
                        Token {
                            tokens: _,
                            token_type: TokenType::Prefixexp,
                        },
                        Token {
                            tokens: _,
                            token_type: TokenType::Args,
                        },
                    )
                ) {
                    log::error!(
                        "Failed to reduce.\n\tExpected: Prefixexp Args\n\tGot: {:?} {:?}",
                        prefixexp.token_type,
                        args.token_type
                    );
                    Err(Error::Reduction)
                } else {
                    self.reduction.replace(Ok(Token {
                        tokens: [prefixexp, args].to_vec(),
                        token_type: TokenType::Functioncall,
                    }));
                    Ok(())
                }
            }
            100 => {
                make_reduction_pop!(self, string);

                if !matches!(string.token_type, TokenType::String(_)) {
                    log::error!(
                        "Failed to reduce.\n\tExpected: String\n\tGot: {:?}",
                        string.token_type
                    );
                    Err(Error::Reduction)
                } else {
                    self.reduction.replace(Ok(Token {
                        tokens: [string].to_vec(),
                        token_type: TokenType::Args,
                    }));
                    Ok(())
                }
            }
            _ => {
                unreachable!()
            }
        }
    }
}
