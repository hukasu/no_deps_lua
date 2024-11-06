mod error;
mod state;
#[cfg(test)]
mod tests;
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
