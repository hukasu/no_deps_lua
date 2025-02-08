use alloc::boxed::Box;

use crate::ext::FloatExt;

use super::{compile_context::CompileContext, exp_desc::ExpDesc, ByteCode, Error, Program};

pub struct Binop<'a, 'b> {
    pub expdesc: &'a ExpDesc<'b>,
    pub top: &'a ExpDesc<'b>,
    pub dst: u8,
}

fn arithmetic_errors(exp: &ExpDesc) -> Result<(), Error> {
    match exp {
        ExpDesc::Nil => Err(Error::NilArithmetic),
        ExpDesc::Boolean(_) => Err(Error::BoolArithmetic),
        ExpDesc::String(_) => Err(Error::StringArithmetic),
        ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _) => Err(Error::TableArithmetic),
        _ => Ok(()),
    }
}

fn bitwise_errors(exp: &ExpDesc) -> Result<(), Error> {
    match exp {
        ExpDesc::Float(_) => Err(Error::FloatBitwise),
        ExpDesc::Nil => Err(Error::NilBitwise),
        ExpDesc::Boolean(_) => Err(Error::BoolBitwise),
        ExpDesc::String(_) => Err(Error::StringBitwise),
        ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _) => Err(Error::TableBitwise),
        _ => Ok(()),
    }
}

fn concat_errors(exp: &ExpDesc) -> Result<(), Error> {
    match exp {
        ExpDesc::Nil => Err(Error::NilConcat),
        ExpDesc::Boolean(_) => Err(Error::BoolConcat),
        ExpDesc::TableLocal(_, _) | ExpDesc::TableGlobal(_, _) => Err(Error::TableConcat),
        _ => Ok(()),
    }
}

pub fn binop_add<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    arithmetic_errors(lhs.expdesc).and_then(|()| arithmetic_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => Ok(ExpDesc::Integer(lhs_i + rhs_i)),
        (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => Ok(ExpDesc::Float(lhs_f + rhs_f)),
        (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float(*lhs_i as f64 + rhs_f))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float(lhs_f + *rhs_i as f64))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::Add,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_sub<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    arithmetic_errors(lhs.expdesc).and_then(|()| arithmetic_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => Ok(ExpDesc::Integer(lhs_i - rhs_i)),
        (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => Ok(ExpDesc::Float(lhs_f - rhs_f)),
        (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float(*lhs_i as f64 - rhs_f))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float(lhs_f - *rhs_i as f64))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::Sub,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_mul<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    arithmetic_errors(lhs.expdesc).and_then(|()| arithmetic_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => Ok(ExpDesc::Integer(lhs_i * rhs_i)),
        (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => Ok(ExpDesc::Float(lhs_f * rhs_f)),
        (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float(*lhs_i as f64 * rhs_f))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float(lhs_f * *rhs_i as f64))
        }
        (ExpDesc::Local(lhs_dst), float @ ExpDesc::Float(_)) => Ok(ExpDesc::BinopConstant(
            ByteCode::MulConstant,
            *lhs_dst,
            Box::new(float.clone()),
        )),
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::Mul,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_mod<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    arithmetic_errors(lhs.expdesc).and_then(|()| arithmetic_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => Ok(ExpDesc::Integer(lhs_i % rhs_i)),
        (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => Ok(ExpDesc::Float(lhs_f % rhs_f)),
        (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float(*lhs_i as f64 % rhs_f))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float(lhs_f % *rhs_i as f64))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::Mod,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_pow<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    arithmetic_errors(lhs.expdesc).and_then(|()| arithmetic_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float({ *lhs_i as f64 }.powf(*rhs_i as f64)))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => Ok(ExpDesc::Float(lhs_f.powf(*rhs_f))),
        (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float((*lhs_i as f64).powf(*rhs_f)))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float(lhs_f.powf(*rhs_i as f64)))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::Pow,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_div<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    arithmetic_errors(lhs.expdesc).and_then(|()| arithmetic_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float(*lhs_i as f64 / *rhs_i as f64))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => Ok(ExpDesc::Float(lhs_f / rhs_f)),
        (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float(*lhs_i as f64 / rhs_f))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float(lhs_f / *rhs_i as f64))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::Div,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_idiv<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    arithmetic_errors(lhs.expdesc).and_then(|()| arithmetic_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs_i), ExpDesc::Integer(rhs_i)) => Ok(ExpDesc::Integer(lhs_i / rhs_i)),
        (ExpDesc::Float(lhs_f), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float((lhs_f / rhs_f).trunc()))
        }
        (ExpDesc::Integer(lhs_i), ExpDesc::Float(rhs_f)) => {
            Ok(ExpDesc::Float((*lhs_i as f64 / rhs_f).trunc()))
        }
        (ExpDesc::Float(lhs_f), ExpDesc::Integer(rhs_i)) => {
            Ok(ExpDesc::Float((lhs_f / *rhs_i as f64).trunc()))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::Idiv,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_bitand<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    bitwise_errors(lhs.expdesc).and_then(|()| bitwise_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs), ExpDesc::Integer(rhs)) => Ok(ExpDesc::Integer(lhs & rhs)),
        (ExpDesc::Integer(lhs), ExpDesc::Float(rhs)) if rhs.zero_frac() => {
            Ok(ExpDesc::Integer(lhs & (*rhs as i64)))
        }
        (ExpDesc::Float(lhs), ExpDesc::Integer(rhs)) if lhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) & rhs))
        }
        (ExpDesc::Float(lhs), ExpDesc::Float(rhs)) if lhs.zero_frac() && rhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) & (*rhs as i64)))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::BitAnd,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_bitor<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    bitwise_errors(lhs.expdesc).and_then(|()| bitwise_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs), ExpDesc::Integer(rhs)) => Ok(ExpDesc::Integer(lhs | rhs)),
        (ExpDesc::Integer(lhs), ExpDesc::Float(rhs)) if rhs.zero_frac() => {
            Ok(ExpDesc::Integer(lhs | (*rhs as i64)))
        }
        (ExpDesc::Float(lhs), ExpDesc::Integer(rhs)) if lhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) | rhs))
        }
        (ExpDesc::Float(lhs), ExpDesc::Float(rhs)) if lhs.zero_frac() && rhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) | (*rhs as i64)))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::BitOr,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_bitxor<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    bitwise_errors(lhs.expdesc).and_then(|()| bitwise_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs), ExpDesc::Integer(rhs)) => Ok(ExpDesc::Integer(lhs ^ rhs)),
        (ExpDesc::Integer(lhs), ExpDesc::Float(rhs)) if rhs.zero_frac() => {
            Ok(ExpDesc::Integer(lhs ^ (*rhs as i64)))
        }
        (ExpDesc::Float(lhs), ExpDesc::Integer(rhs)) if lhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) ^ rhs))
        }
        (ExpDesc::Float(lhs), ExpDesc::Float(rhs)) if lhs.zero_frac() && rhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) ^ (*rhs as i64)))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::BitXor,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_shiftl<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    bitwise_errors(lhs.expdesc).and_then(|()| bitwise_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs), ExpDesc::Integer(rhs)) => Ok(ExpDesc::Integer(lhs << rhs)),
        (ExpDesc::Integer(lhs), ExpDesc::Float(rhs)) if rhs.zero_frac() => {
            Ok(ExpDesc::Integer(lhs << (*rhs as i64)))
        }
        (ExpDesc::Float(lhs), ExpDesc::Integer(rhs)) if lhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) << rhs))
        }
        (ExpDesc::Float(lhs), ExpDesc::Float(rhs)) if lhs.zero_frac() && rhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) << (*rhs as i64)))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::ShiftL,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_shiftr<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    bitwise_errors(lhs.expdesc).and_then(|()| bitwise_errors(rhs.expdesc))?;
    match (lhs.expdesc, rhs.expdesc) {
        (ExpDesc::Integer(lhs), ExpDesc::Integer(rhs)) => Ok(ExpDesc::Integer(lhs >> rhs)),
        (ExpDesc::Integer(lhs), ExpDesc::Float(rhs)) if rhs.zero_frac() => {
            Ok(ExpDesc::Integer(lhs >> (*rhs as i64)))
        }
        (ExpDesc::Float(lhs), ExpDesc::Integer(rhs)) if lhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) >> rhs))
        }
        (ExpDesc::Float(lhs), ExpDesc::Float(rhs)) if lhs.zero_frac() && rhs.zero_frac() => {
            Ok(ExpDesc::Integer((*lhs as i64) >> (*rhs as i64)))
        }
        (lhs_expdesc, rhs_expdesc) => {
            lhs_expdesc.discharge(lhs.top, program, compile_context)?;
            rhs_expdesc.discharge(rhs.top, program, compile_context)?;

            Ok(ExpDesc::Binop(
                ByteCode::ShiftR,
                usize::from(lhs.dst),
                usize::from(rhs.dst),
            ))
        }
    }
}

pub fn binop_concat<'a, 'b>(
    program: &mut Program,
    compile_context: &mut CompileContext,
    lhs: Binop<'a, 'b>,
    rhs: Binop<'a, 'b>,
) -> Result<ExpDesc<'b>, Error> {
    concat_errors(lhs.expdesc).and_then(|()| concat_errors(rhs.expdesc))?;
    lhs.expdesc.discharge(lhs.top, program, compile_context)?;
    rhs.expdesc.discharge(rhs.top, program, compile_context)?;

    Ok(ExpDesc::Binop(
        ByteCode::Concat,
        usize::from(lhs.dst),
        usize::from(rhs.dst),
    ))
}
