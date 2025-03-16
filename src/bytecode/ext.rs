use crate::ext::FloatExt;

pub trait BytecodeExt {
    fn to_sbx(&self) -> Option<i32>;
}

impl BytecodeExt for i64 {
    fn to_sbx(&self) -> Option<i32> {
        i32::try_from(*self)
            .ok()
            .filter(|integer| super::I17_OFFSET.saturating_add_signed(*integer) < super::BX_MAX)
    }
}

impl BytecodeExt for f64 {
    fn to_sbx(&self) -> Option<i32> {
        if self.zero_frac() {
            (*self as i64).to_sbx()
        } else {
            None
        }
    }
}
