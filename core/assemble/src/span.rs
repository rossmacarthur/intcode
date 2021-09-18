//! Represents a span in the input.

use std::fmt::Debug;
use std::ops;

/// Represents a spanned `T`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct S<T>(pub T, pub Span);

/// Represents a location in the original input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    /// The start index.
    pub m: usize,
    /// The end index.
    pub n: usize,
}

pub fn s<T>(t: T, span: impl Into<Span>) -> S<T> {
    S(t, span.into())
}

impl<T> ops::Deref for S<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Span {
    pub fn include(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        Self {
            m: self.m,
            n: other.n,
        }
    }

    pub fn as_str<'i>(&self, input: &'i str) -> &'i str {
        &input[self.m..self.n]
    }
}

impl From<Span> for ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.m..span.n
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
