use core::{borrow::Borrow, num::FpCategory};

pub trait FloatExt {
    fn to_i16(&self) -> Option<i16>;
}

impl<T: Borrow<f64>> FloatExt for T {
    fn to_i16(&self) -> Option<i16> {
        const FLOOR: f64 = i16::MIN as f64;
        const CEILING: f64 = i16::MAX as f64;
        match *self.borrow() {
            f @ FLOOR..=CEILING if f.fract().classify() == FpCategory::Zero => Some(f as i16),
            _ => None,
        }
    }
}
