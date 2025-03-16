use core::num::FpCategory;

pub trait FloatExt {
    /// Checks if the fraction part is zero
    fn zero_frac(&self) -> bool;
}

impl FloatExt for f64 {
    fn zero_frac(&self) -> bool {
        self.fract().classify() == FpCategory::Zero
    }
}
