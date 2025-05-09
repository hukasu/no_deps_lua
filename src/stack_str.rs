use core::{fmt::Display, ops::Deref};

use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackStr<const N: usize> {
    buffer: [u8; N],
}

impl<const N: usize> StackStr<N> {
    pub fn new(string: impl Deref<Target = str>) -> Result<Self, Error> {
        if string.len() <= N {
            let mut iter = string.as_bytes().iter();
            let this = Self {
                buffer: core::array::from_fn(|_| iter.next().copied().unwrap_or(b'\0')),
            };
            assert!(iter.next().is_none());
            Ok(this)
        } else {
            Err(Error::StringTooBig)
        }
    }

    pub fn len(&self) -> usize {
        self.buffer
            .iter()
            .rposition(|c| *c != b'\0')
            .map(|pos| pos + 1)
            .unwrap_or_default()
    }
}

impl<const N: usize> Display for StackStr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.buffer).trim_end_matches("\0")
        )
    }
}

impl<const N: usize> Deref for StackStr<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<const N: usize> From<[u8; N]> for StackStr<N> {
    fn from(buffer: [u8; N]) -> Self {
        Self { buffer }
    }
}

#[derive(Debug)]
pub enum Error {
    StringTooBig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        StackStr::<15>::new("").expect("Should work");
        StackStr::<15>::new("a").expect("Should work");
        StackStr::<15>::new("aa").expect("Should work");
        StackStr::<15>::new("aaa").expect("Should work");
        StackStr::<15>::new("aaaa").expect("Should work");
        StackStr::<15>::new("aaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaaaaaaaa").expect("Should work");
        StackStr::<15>::new("aaaaaaaaaaaaaaaa").expect_err("Should not work");
    }

    #[test]
    fn test_len() {
        let stack_str = StackStr::<15>::new("aaaa").unwrap();
        assert_eq!(stack_str.len(), 4);

        let stack_str = StackStr::<15>::new("aaaaaa").unwrap();
        assert_eq!(stack_str.len(), 6);

        let stack_str = StackStr::<15>::new("aaaaaaaa").unwrap();
        assert_eq!(stack_str.len(), 8);

        let stack_str = StackStr::<15>::new("aaaaaaaaaa").unwrap();
        assert_eq!(stack_str.len(), 10);

        let stack_str = StackStr::<15>::new("aaaaaaaaaaaaaaa").unwrap();
        assert_eq!(stack_str.len(), 15);

        let stack_str = StackStr::<15>::new("aaaaaaa\0aaaaaaa").unwrap();
        assert_eq!(stack_str.len(), 15);

        let stack_str = StackStr::<15>::new("aaaaaaaa\0aaaaaa").unwrap();
        assert_eq!(stack_str.len(), 15);
    }

    #[test]
    fn test_equality() {
        let a = StackStr::<15>::new("aaaa").unwrap();
        let b = StackStr::<15>::new("aaaa").unwrap();
        let c = StackStr::<15>::new("aaaaa").unwrap();
        let d = StackStr::<15>::new("bbbb").unwrap();
        let e = StackStr::<15>::new("aaab").unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(a, e);
        assert_ne!(b, c);
        assert_ne!(b, d);
        assert_ne!(b, e);
        assert_ne!(c, d);
        assert_ne!(c, e);
        assert_ne!(d, e);
    }
}
