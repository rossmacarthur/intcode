//! Defines an error type for the compiler.

pub mod fmt;
pub mod span;

use dairy::Cow;
use thiserror::Error;

use crate::span::Span;

pub type Result<T> = std::result::Result<T, Error>;
pub type ResultSet<T> = std::result::Result<T, ErrorSet>;

/// A parse error.
///
/// Depending on the context, This can be an unexpected character, token, or
/// value. The message is what will be displayed in the formatted output. The
/// span specifies what will be underlined in the formatted output.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{msg}")]
pub struct Error {
    pub msg: Cow<'static, str>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub msg: Cow<'static, str>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct ErrorSet {
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Error {
    pub fn new(msg: impl Into<Cow<'static, str>>, span: impl Into<Span>) -> Self {
        Self {
            span: span.into(),
            msg: msg.into(),
        }
    }
}

impl Warning {
    pub fn new(msg: impl Into<Cow<'static, str>>, span: impl Into<Span>) -> Self {
        Self {
            span: span.into(),
            msg: msg.into(),
        }
    }
}
