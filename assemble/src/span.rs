use std::fmt::Debug;
use std::ops;
use std::str::FromStr;

/// Represents a location in the original input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The start index.
    pub m: usize,
    /// The end index.
    pub n: usize,
}

impl Span {
    pub fn new(m: usize, n: usize) -> Self {
        Self { m, n }
    }

    pub fn width(&self) -> usize {
        self.n - self.m
    }

    pub fn range(&self) -> ops::Range<usize> {
        self.m..self.n
    }

    pub fn as_str<'i>(&self, input: &'i str) -> &'i str {
        &input[self.range()]
    }

    pub fn parse<T>(&self, input: &str) -> T
    where
        T: FromStr,
        T::Err: Debug,
    {
        self.as_str(input).parse().unwrap()
    }
}

impl From<usize> for Span {
    fn from(m: usize) -> Self {
        Self { m, n: m + 1 }
    }
}

impl From<ops::Range<usize>> for Span {
    fn from(r: ops::Range<usize>) -> Self {
        Self {
            m: r.start,
            n: r.end,
        }
    }
}
