mod error;
mod state;

use alloc::{string::String, vec::Vec};

pub use {
    error::{Error, ErrorKind},
    state::{State, StateMachine},
};

pub struct Lex<'a> {
    data: &'a [u8],
    state: State,
}

impl<'a> Lex<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let state = State {
            line: 0,
            column: 0,
            machine: StateMachine::Start,
            buffer: Vec::new(),
        };
        Self { data, state }
    }
}

impl<'a> Iterator for Lex<'a> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(c) = self.data.first() else {
                break self.state.process(0);
            };
            self.data = &self.data[1..];

            if let Some(res) = self.state.process(*c) {
                break Some(res);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Name(String),
    String(String),
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;

    use super::*;

    #[test]
    fn empty_input() {
        let mut lex = Lex::new(b"");
        assert!(lex.next().is_none());
        assert_eq!(lex.data.len(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new(b"        \n\n\n\n\t\t\t\t\r\r\r\r");
        assert!(lex.next().is_none());
        assert_eq!(lex.data.len(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);
    }

    #[test]
    fn short_comment() {
        let mut lex = Lex::new(b"-- abc");
        assert!(lex.next().is_none());
        assert_eq!(lex.data.len(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new(b"-- Lorem ipsum dolor sit amet, consectetur adipiscing elit.");
        assert!(lex.next().is_none());
        assert_eq!(lex.data.len(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new(b"--\x01");
        assert!(matches!(
            lex.next(),
            Some(Err(Error {
                kind: ErrorKind::ProhibtedControlCharacterOnComment,
                line: 0,
                column: 2
            }))
        ));

        let mut lex = Lex::new(b"-- Lorem ipsum dolor sit amet,\x01consectetur adipiscing elit.");
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
        let mut lex = Lex::new(b"print \"hello world\"");
        assert_eq!(lex.next(), Some(Ok(Token::Name("print".to_owned()))));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::String("hello world".to_owned())))
        );
        assert!(lex.next().is_none(),);
        assert_eq!(lex.data.len(), 0);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new(b"print \"hello world\"\nprint \"hello again...\"");
        assert_eq!(lex.next(), Some(Ok(Token::Name("print".to_owned()))));
        assert_eq!(lex.state.line, 0);
        assert_eq!(lex.state.column, 6);
        assert_eq!(
            lex.next(),
            Some(Ok(Token::String("hello world".to_owned())))
        );
        assert_eq!(lex.state.line, 0);
        assert_eq!(lex.state.column, 19);
        assert_eq!(lex.next(), Some(Ok(Token::Name("print".to_owned()))));
        assert_eq!(lex.state.line, 1);
        assert_eq!(lex.state.column, 6);
        assert_eq!(
            lex.next(),
            Some(Ok(Token::String("hello again...".to_owned())))
        );
        assert!(lex.next().is_none(),);
        assert_eq!(lex.data.len(), 0);
        assert_eq!(lex.state.line, 1);
        assert_eq!(lex.state.column, 23);
        assert_eq!(lex.state.machine, StateMachine::End);

        let mut lex = Lex::new(b"print \"hello world");
        assert_eq!(lex.next(), Some(Ok(Token::Name("print".to_owned()))));
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
