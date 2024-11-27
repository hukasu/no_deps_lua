use core::{borrow::Borrow, num::FpCategory};

pub trait FloatExt {
    fn to_i16(&self) -> Option<i16>;
}

impl<T: Borrow<f64>> FloatExt for T {
    fn to_i16(&self) -> Option<i16> {
        let float: f64 = *self.borrow();

        if float.fract().classify() == FpCategory::Zero {
            if ((i64::from(i16::MIN))..=(i64::from(i16::MAX))).contains(&(float as i64)) {
                Some(float as i16)
            } else {
                None
            }
        } else {
            None
        }
    }
}
