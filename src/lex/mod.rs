mod error;
mod state;
mod token;

use core::str::Chars;

use state::{State, StateMachine};
pub use {
    error::{Error, ErrorKind},
    token::{Token, TokenType},
};

pub struct Lex<'a> {
    program: &'a str,
    chars: Chars<'a>,
    state: State,
}

impl<'a> Lex<'a> {
    pub fn new(data: &'a str) -> Self {
        let state = State {
            line: 0,
            column: 0,
            seek: 0,
            machine: StateMachine::Start,
            buffer_len: 0,
        };
        Self {
            program: data,
            chars: data.chars(),
            state,
        }
    }

    #[cfg(test)]
    pub fn remaining(&self) -> usize {
        self.program.len() - self.state.seek
    }
}

impl<'a> Iterator for Lex<'a> {
    type Item = Result<(Token<'a>, Option<Token<'a>>), Error>;

    fn next(&mut self) -> Option<Self::Item>
    where
        Self: 'a,
    {
        loop {
            let Some(c) = self.chars.next() else {
                break self.state.process(self.program, '\0');
            };

            if let Some(res) = self.state.process(self.program, c) {
                break Some(res);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let mut lex = Lex::new("");
        assert!(lex.next().is_none());
        assert_eq!(lex.program.len(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new("        \n\n\n\n\t\t\t\t\r\r\r\r");
        assert!(lex.next().is_none());
        assert_eq!(lex.remaining(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);
    }

    #[test]
    fn keywords() {
        let mut lex = Lex::new(
            r#"
and       break     do        else      elseif    end
false     for       function  goto      if        in
local     nil       not       or        repeat    return
then      true      until     while     keyword
"#,
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 3,
                    start_offset: 3,
                    token: TokenType::And
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 15,
                    start_offset: 5,
                    token: TokenType::Break
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 22,
                    start_offset: 2,
                    token: TokenType::Do
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 34,
                    start_offset: 4,
                    token: TokenType::Else
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 46,
                    start_offset: 6,
                    token: TokenType::Elseif
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 53,
                    start_offset: 3,
                    token: TokenType::End
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 2,
                    column: 5,
                    start_offset: 5,
                    token: TokenType::False
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 2,
                    column: 13,
                    start_offset: 3,
                    token: TokenType::For
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 2,
                    column: 28,
                    start_offset: 8,
                    token: TokenType::Function
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 2,
                    column: 34,
                    start_offset: 4,
                    token: TokenType::Goto
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 2,
                    column: 42,
                    start_offset: 2,
                    token: TokenType::If
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 2,
                    column: 52,
                    start_offset: 2,
                    token: TokenType::In
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 3,
                    column: 5,
                    start_offset: 5,
                    token: TokenType::Local
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 3,
                    column: 13,
                    start_offset: 3,
                    token: TokenType::Nil
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 3,
                    column: 23,
                    start_offset: 3,
                    token: TokenType::Not
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 3,
                    column: 32,
                    start_offset: 2,
                    token: TokenType::Or
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 3,
                    column: 46,
                    start_offset: 6,
                    token: TokenType::Repeat
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 3,
                    column: 56,
                    start_offset: 6,
                    token: TokenType::Return
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 4,
                    column: 4,
                    start_offset: 4,
                    token: TokenType::Then
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 4,
                    column: 14,
                    start_offset: 4,
                    token: TokenType::True
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 4,
                    column: 25,
                    start_offset: 5,
                    token: TokenType::Until
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 4,
                    column: 35,
                    start_offset: 5,
                    token: TokenType::While
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 4,
                    column: 47,
                    start_offset: 7,
                    token: TokenType::Name("keyword")
                },
                None
            )))
        );
        assert!(lex.next().is_none());
        assert_eq!(lex.remaining(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);
    }

    #[test]
    fn short_comment() {
        let mut lex = Lex::new("-- abc");
        assert!(lex.next().is_none());
        assert_eq!(lex.remaining(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new("-- Lorem ipsum dolor sit amet, consectetur adipiscing elit.");
        assert!(lex.next().is_none());
        assert_eq!(lex.remaining(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new("--\x01");
        assert!(matches!(
            lex.next(),
            Some(Err(Error {
                kind: ErrorKind::ProhibtedControlCharacterOnComment,
                line: 0,
                column: 2
            }))
        ));

        let mut lex = Lex::new("-- Lorem ipsum dolor sit amet,\x01consectetur adipiscing elit.");
        assert!(matches!(
            lex.next(),
            Some(Err(Error {
                kind: ErrorKind::ProhibtedControlCharacterOnComment,
                line: 0,
                column: 30
            }))
        ));
    }

    #[test]
    fn hello_world() {
        let mut lex = Lex::new(r#"print "hello world""#);
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 0,
                    column: 5,
                    start_offset: 5,
                    token: TokenType::Name("print")
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 0,
                    column: 18,
                    start_offset: 13,
                    token: TokenType::String("hello world")
                },
                None
            )))
        );
        assert!(lex.next().is_none(),);
        assert_eq!(lex.remaining(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new("print \"hello world\"\nprint \"hello again...\"");
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 0,
                    column: 5,
                    start_offset: 5,
                    token: TokenType::Name("print")
                },
                None
            )))
        );
        assert_eq!(lex.state.line, 0);
        assert_eq!(lex.state.column, 6);
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 0,
                    column: 18,
                    start_offset: 13,
                    token: TokenType::String("hello world")
                },
                None
            )))
        );
        assert_eq!(lex.state.line, 0);
        assert_eq!(lex.state.column, 19);
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 5,
                    start_offset: 5,
                    token: TokenType::Name("print")
                },
                None
            )))
        );
        assert_eq!(lex.state.line, 1);
        assert_eq!(lex.state.column, 6);
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 1,
                    column: 21,
                    start_offset: 16,
                    token: TokenType::String("hello again...")
                },
                None
            )))
        );
        assert!(lex.next().is_none(),);
        assert_eq!(lex.remaining(), 0);
        assert_eq!(lex.state.line, 1);
        assert_eq!(lex.state.column, 23);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new("print \"hello world");
        assert_eq!(
            lex.next(),
            Some(Ok((
                Token {
                    line: 0,
                    column: 5,
                    start_offset: 5,
                    token: TokenType::Name("print")
                },
                None
            )))
        );
        assert_eq!(
            lex.next(),
            Some(Err(Error {
                kind: ErrorKind::EofAtString,
                line: 0,
                column: 18
            }))
        );
    }
}
