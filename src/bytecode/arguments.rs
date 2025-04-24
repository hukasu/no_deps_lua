use core::{fmt::Display, ops::Deref};

use crate::ext::FloatExt;

const A_MASK: u32 = 0xff << A_SHIFT;
const A_SHIFT: u32 = 7;
const B_MASK: u32 = 0xff << B_SHIFT;
const B_SHIFT: u32 = 16;
const C_MASK: u32 = 0xff << C_SHIFT;
const C_SHIFT: u32 = 24;
const K_MASK: u32 = 0x1 << K_SHIFT;
const K_SHIFT: u32 = 15;

const AX_MAX: u32 = u32::MAX >> AX_SHIFT;
const AX_MASK: u32 = AX_MAX << AX_SHIFT;
const AX_SHIFT: u32 = A_SHIFT;
const BX_MAX: u32 = u32::MAX >> BX_SHIFT;
const BX_MASK: u32 = BX_MAX << BX_SHIFT;
const BX_SHIFT: u32 = K_SHIFT;

const J_MAX: u32 = AX_MAX;
const J_SHIFT: u32 = A_SHIFT;

const I8_OFFSET: u8 = u8::MAX >> 1;
const I17_OFFSET: u32 = BX_MAX >> 1;
const I25_OFFSET: u32 = J_MAX >> 1;

pub trait BytecodeArgument {
    fn write(&self, bytecode: &mut u32);
    fn read(bytecode: u32) -> Self;
}

#[derive(Debug, PartialEq)]
pub enum BytecodeArgumentError {
    AxTooLarge(u32),
    BxTooLarge(u32),
    SbxTooLarge(i64),
    SbxTooSmall(i64),
    SjTooLarge(i32),
    SjTooSmall(i32),
    InvalidK(u8),
    FloatNotInteger(f64),
    Downcast(&'static str, &'static str),
}

impl Display for BytecodeArgumentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AxTooLarge(value) => write!(
                f,
                "Value `{}` does not fit in a 25-bit unsigned integer.",
                value
            ),
            Self::BxTooLarge(value) => {
                write!(
                    f,
                    "Value `{}` does not fit in a 17-bit unsigned integer.",
                    value
                )
            }
            Self::SbxTooLarge(value) => write!(
                f,
                "Value `{}` is to large for a 17-bit signed integer.",
                value
            ),
            Self::SbxTooSmall(value) => write!(
                f,
                "Value `{}` is to small for a 17-bit signed integer.",
                value
            ),
            Self::SjTooLarge(value) => write!(
                f,
                "Value `{}` is to large for a 25-bit signed integer.",
                value
            ),
            Self::SjTooSmall(value) => write!(
                f,
                "Value `{}` is to small for a 25-bit signed integer.",
                value
            ),
            Self::InvalidK(value) => {
                write!(f, "K only accepts `0` or `1`, but was given {}.", value)
            }
            Self::FloatNotInteger(value) => {
                write!(f, "Float {} can't be expressed as an integer.", value)
            }
            Self::Downcast(src, dst) => {
                write!(f, "Could not downcast `{}` to `{}`.", src, dst)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct A(u8);

impl A {
    pub const ZERO: Self = Self(0);
}

impl BytecodeArgument for A {
    fn write(&self, bytecode: &mut u32) {
        *bytecode |= (self.0 as u32) << A_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        Self(((bytecode & A_MASK) >> A_SHIFT) as u8)
    }
}

impl Deref for A {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for A {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ax(u32);

impl BytecodeArgument for Ax {
    fn write(&self, bytecode: &mut u32) {
        *bytecode |= self.0 << AX_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        Self((bytecode & AX_MASK) >> AX_SHIFT)
    }
}

impl Deref for Ax {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<u32> for Ax {
    type Error = BytecodeArgumentError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value <= AX_MAX {
            Ok(Self(value))
        } else {
            Err(BytecodeArgumentError::AxTooLarge(value))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct B(u8);

impl B {
    pub const ZERO: Self = Self(0);
}

impl BytecodeArgument for B {
    fn write(&self, bytecode: &mut u32) {
        *bytecode |= (self.0 as u32) << B_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        Self(((bytecode & B_MASK) >> B_SHIFT) as u8)
    }
}

impl Deref for B {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for B {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sb(i8);

impl Sb {
    pub const ZERO: Self = Self(0);
}

impl BytecodeArgument for Sb {
    fn write(&self, bytecode: &mut u32) {
        let b = I8_OFFSET.saturating_add_signed(self.0);
        *bytecode |= (b as u32) << B_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        let B(b) = B::read(bytecode);
        if b > I8_OFFSET {
            Self((b - I8_OFFSET) as i8)
        } else {
            Self(b as i8 - I8_OFFSET as i8)
        }
    }
}

impl Deref for Sb {
    type Target = i8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i8> for Sb {
    fn from(value: i8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bx(u32);

impl Bx {
    pub const ZERO: Self = Self(0);
}

impl BytecodeArgument for Bx {
    fn write(&self, bytecode: &mut u32) {
        *bytecode |= self.0 << BX_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        Self((bytecode & BX_MASK) >> BX_SHIFT)
    }
}

impl Deref for Bx {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for Bx {
    fn from(value: u8) -> Self {
        Self(u32::from(value))
    }
}

impl From<u16> for Bx {
    fn from(value: u16) -> Self {
        Self(u32::from(value))
    }
}

impl TryFrom<u32> for Bx {
    type Error = BytecodeArgumentError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value <= BX_MAX {
            Ok(Self(value))
        } else {
            Err(BytecodeArgumentError::BxTooLarge(value))
        }
    }
}

impl TryFrom<usize> for Bx {
    type Error = BytecodeArgumentError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value)
            .map_err(|_| BytecodeArgumentError::Downcast("usize", "u32"))
            .and_then(|value| value.try_into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sbx(i32);

impl Sbx {
    pub const ZERO: Self = Self(0);
}

impl BytecodeArgument for Sbx {
    fn write(&self, bytecode: &mut u32) {
        let bx = I17_OFFSET.saturating_add_signed(self.0);
        *bytecode |= bx << BX_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        let Bx(bx) = Bx::read(bytecode);
        if bx > I17_OFFSET {
            Self((bx - I17_OFFSET) as i32)
        } else {
            Self(bx as i32 - I17_OFFSET as i32)
        }
    }
}

impl Deref for Sbx {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i8> for Sbx {
    fn from(value: i8) -> Self {
        Self(i32::from(value))
    }
}

impl From<i16> for Sbx {
    fn from(value: i16) -> Self {
        Self(i32::from(value))
    }
}

impl TryFrom<i32> for Sbx {
    type Error = BytecodeArgumentError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if I17_OFFSET.saturating_add_signed(value) < BX_MAX {
            Ok(Self(value))
        } else if value < 0 {
            Err(BytecodeArgumentError::SbxTooSmall(value.into()))
        } else {
            Err(BytecodeArgumentError::SbxTooLarge(value.into()))
        }
    }
}

impl TryFrom<i64> for Sbx {
    type Error = BytecodeArgumentError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if let Ok(value) = i32::try_from(value) {
            if I17_OFFSET.saturating_add_signed(value) < BX_MAX {
                Ok(Self(value))
            } else if value < 0 {
                Err(BytecodeArgumentError::SbxTooSmall(value.into()))
            } else {
                Err(BytecodeArgumentError::SbxTooLarge(value.into()))
            }
        } else if value > i64::from(i32::MAX) {
            Err(BytecodeArgumentError::SbxTooLarge(value))
        } else {
            Err(BytecodeArgumentError::SbxTooSmall(value))
        }
    }
}

impl TryFrom<f64> for Sbx {
    type Error = BytecodeArgumentError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.zero_frac() {
            let value = value as i64;
            value.try_into()
        } else {
            Err(BytecodeArgumentError::FloatNotInteger(value))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C(u8);

impl C {
    pub const ZERO: Self = Self(0);
}

impl BytecodeArgument for C {
    fn write(&self, bytecode: &mut u32) {
        *bytecode |= (self.0 as u32) << C_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        C(((bytecode & C_MASK) >> C_SHIFT) as u8)
    }
}

impl Deref for C {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for C {
    fn from(value: u8) -> Self {
        C(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sc(i8);

impl Sc {
    pub const ZERO: Sc = Sc(0);
}

impl BytecodeArgument for Sc {
    fn write(&self, bytecode: &mut u32) {
        let c = I8_OFFSET.saturating_add_signed(self.0);
        *bytecode |= (c as u32) << C_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        let C(c) = C::read(bytecode);
        if c > I8_OFFSET {
            Sc((c - I8_OFFSET) as i8)
        } else {
            Sc((c as i8) - (I8_OFFSET as i8))
        }
    }
}

impl Deref for Sc {
    type Target = i8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i8> for Sc {
    fn from(value: i8) -> Self {
        Sc(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct K(bool);

impl K {
    pub const ZERO: K = K(false);
    pub const ONE: K = K(true);

    pub fn flip(&self) -> Self {
        Self(!self.0)
    }
}

impl BytecodeArgument for K {
    fn write(&self, bytecode: &mut u32) {
        *bytecode |= (self.0 as u32) << K_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        K(((bytecode & K_MASK) >> K_SHIFT) == 1)
    }
}

impl Deref for K {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<bool> for K {
    fn from(value: bool) -> Self {
        K(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sj(i32);

impl Sj {
    pub const ZERO: Self = Self(0);
}

impl BytecodeArgument for Sj {
    fn write(&self, bytecode: &mut u32) {
        let j = I25_OFFSET.saturating_add_signed(self.0);
        *bytecode |= j << J_SHIFT;
    }

    fn read(bytecode: u32) -> Self {
        let Ax(ax) = Ax::read(bytecode);
        if ax > I25_OFFSET {
            Self((ax - I25_OFFSET) as i32)
        } else {
            Self(ax as i32 - I25_OFFSET as i32)
        }
    }
}

impl Deref for Sj {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i8> for Sj {
    fn from(value: i8) -> Self {
        Self(i32::from(value))
    }
}

impl From<i16> for Sj {
    fn from(value: i16) -> Self {
        Self(i32::from(value))
    }
}

impl TryFrom<i32> for Sj {
    type Error = BytecodeArgumentError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if I25_OFFSET.saturating_add_signed(value) < J_MAX {
            Ok(Self(value))
        } else if value < 0 {
            Err(BytecodeArgumentError::SjTooSmall(value))
        } else {
            Err(BytecodeArgumentError::SjTooLarge(value))
        }
    }
}
