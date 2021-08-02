use std::fmt::Debug;
use std::ops;

/// Represents a location in the original input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The start index.
    pub m: usize,
    /// The end index.
    pub n: usize,
}

impl Span {
    pub fn include(self, other: Self) -> Self {
        Self {
            m: self.m,
            n: other.n,
        }
    }

    pub fn as_str<'i>(&self, input: &'i str) -> &'i str {
        &input[self.m..self.n]
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
