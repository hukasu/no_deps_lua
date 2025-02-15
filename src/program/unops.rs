use alloc::boxed::Box;

use super::{exp_desc::ExpDesc, ByteCode, Error};

// TODO compile time optimizations

pub fn unop_not<'a>(rhs: &ExpDesc<'a>) -> Result<ExpDesc<'a>, Error> {
    match rhs {
        ExpDesc::Nil => Ok(ExpDesc::Boolean(true)),
        ExpDesc::Boolean(bool) => Ok(ExpDesc::Boolean(!bool)),
        other => Ok(ExpDesc::Unop(ByteCode::Not, Box::new(other.clone()))),
    }
}

pub fn unop_len<'a>(rhs: &ExpDesc<'a>) -> Result<ExpDesc<'a>, Error> {
    match rhs {
        ExpDesc::String(string) => Ok(ExpDesc::Integer(i64::try_from(string.len())?)),
        other => Ok(ExpDesc::Unop(ByteCode::Len, Box::new(other.clone()))),
    }
}

pub fn unop_neg<'a>(rhs: &ExpDesc<'a>) -> Result<ExpDesc<'a>, Error> {
    match rhs {
        ExpDesc::Integer(int) => Ok(ExpDesc::Integer(-int)),
        ExpDesc::Float(float) => Ok(ExpDesc::Float(-float)),
        other => Ok(ExpDesc::Unop(ByteCode::Neg, Box::new(other.clone()))),
    }
}

pub fn unop_bitnot<'a>(rhs: &ExpDesc<'a>) -> Result<ExpDesc<'a>, Error> {
    match rhs {
        ExpDesc::Integer(int) => Ok(ExpDesc::Integer(!int)),
        other => Ok(ExpDesc::Unop(ByteCode::BitNot, Box::new(other.clone()))),
    }
}
