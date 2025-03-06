use core::{borrow::Borrow, num::FpCategory};

pub trait FloatExt {
    /// Checks if the fraction part is zero
    fn zero_frac(&self) -> bool;
    /// Converts to 17-bit integer if fraction part is zero
    fn to_sbx(&self) -> Option<i32>;
}

impl<T: Borrow<f64>> FloatExt for T {
    fn zero_frac(&self) -> bool {
        self.borrow().fract().classify() == FpCategory::Zero
    }

    fn to_sbx(&self) -> Option<i32> {
        // TODO use proper bounds
        const FLOOR: f64 = i16::MIN as f64;
        const CEILING: f64 = i16::MAX as f64;
        match *self.borrow() {
            f @ FLOOR..=CEILING if f.zero_frac() => Some(f as i32),
            _ => None,
        }
    }
}
