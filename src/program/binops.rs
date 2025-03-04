use crate::parser::TokenType;

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Binop {
    Add,
    Sub,
    Mul,
    Mod,
    Pow,
    Div,
    Idiv,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    Concat,
    Or,
    And,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
}

impl Binop {
    pub fn arithmetic_binary_operator(&self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Sub
                | Self::Mul
                | Self::Mod
                | Self::Pow
                | Self::Div
                | Self::Idiv
                | Self::BitAnd
                | Self::BitOr
                | Self::BitXor
                | Self::ShiftLeft
                | Self::ShiftRight
        )
    }

    pub fn arithmetic_binary_operator_constant(&self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Sub
                | Self::Mul
                | Self::Mod
                | Self::Pow
                | Self::Div
                | Self::Idiv
                | Self::BitAnd
                | Self::BitOr
                | Self::BitXor
        )
    }

    pub fn conditional_binary_operator(&self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    pub fn relational_binary_operator(&self) -> bool {
        matches!(
            self,
            Self::Equal
                | Self::NotEqual
                | Self::LessThan
                | Self::GreaterThan
                | Self::LessEqual
                | Self::GreaterEqual
        )
    }

    pub fn relational_binary_operator_integer(&self) -> bool {
        matches!(
            self,
            Self::Equal | Self::LessThan | Self::GreaterThan | Self::LessEqual | Self::GreaterEqual
        )
    }

    pub fn relational_binary_operator_constant(&self) -> bool {
        matches!(self, Self::Equal)
    }
}

impl TryFrom<TokenType<'_>> for Binop {
    type Error = Error;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        match value {
            TokenType::Add => Ok(Self::Add),
            TokenType::Sub => Ok(Self::Sub),
            TokenType::Mul => Ok(Self::Mul),
            TokenType::Mod => Ok(Self::Mod),
            TokenType::Pow => Ok(Self::Pow),
            TokenType::Div => Ok(Self::Div),
            TokenType::Idiv => Ok(Self::Idiv),
            TokenType::BitAnd => Ok(Self::BitAnd),
            TokenType::BitOr => Ok(Self::BitOr),
            TokenType::BitXor => Ok(Self::BitXor),
            TokenType::ShiftL => Ok(Self::ShiftLeft),
            TokenType::ShiftR => Ok(Self::ShiftRight),
            TokenType::Concat => Ok(Self::Concat),
            TokenType::Or => Ok(Self::Or),
            TokenType::And => Ok(Self::And),
            TokenType::Less => Ok(Self::LessThan),
            TokenType::Greater => Ok(Self::GreaterThan),
            TokenType::Leq => Ok(Self::LessEqual),
            TokenType::Geq => Ok(Self::GreaterEqual),
            TokenType::Eq => Ok(Self::Equal),
            TokenType::Neq => Ok(Self::NotEqual),
            other => {
                log::error!("{:?} is not a binary operator", other);
                Err(Error::NotBinaryOperator)
            }
        }
    }
}
