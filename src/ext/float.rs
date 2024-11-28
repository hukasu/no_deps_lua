use core::{borrow::Borrow, num::FpCategory};

pub trait FloatExt {
    /// Checks if the fraction part is zero
    fn zero_frac(&self) -> bool;
    /// Converts to i16 if fraction part is zero
    fn to_i16(&self) -> Option<i16>;
}

impl<T: Borrow<f64>> FloatExt for T {
    fn zero_frac(&self) -> bool {
        self.borrow().fract().classify() == FpCategory::Zero
    }

    fn to_i16(&self) -> Option<i16> {
        const FLOOR: f64 = i16::MIN as f64;
        const CEILING: f64 = i16::MAX as f64;
        match *self.borrow() {
            f @ FLOOR..=CEILING if f.zero_frac() => Some(f as i16),
            _ => None,
        }
    }
}
