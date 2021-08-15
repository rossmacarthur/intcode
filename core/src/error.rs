//! An error type for the lexer and parser.

use dairy::Cow;
use thiserror::Error;

use crate::span::Span;

pub type Result<T> = std::result::Result<T, Error>;

/// A parse error.
///
/// Depending on the context, This can be an unexpected character, token, or
/// value. The span specifies what will be underlined in the error message. The
/// message is what will be displayed in the formatted output.
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

impl Warning {
    pub(crate) fn new(msg: impl Into<Cow<'static, str>>, span: impl Into<Span>) -> Self {
        Self {
            span: span.into(),
            msg: msg.into(),
        }
    }
}

impl Error {
    pub(crate) fn new(msg: impl Into<Cow<'static, str>>, span: impl Into<Span>) -> Self {
        Self {
            span: span.into(),
            msg: msg.into(),
        }
    }
}
