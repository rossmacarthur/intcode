//! An error type for the lexer and parser.

use std::cmp::max;
use std::path::Path;

use dairy::Cow;
use thiserror::Error;
use unicode_width::UnicodeWidthStr;
use yansi::Paint;

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
    msg: Cow<'static, str>,
    span: Span,
}

fn to_line_col(lines: &[&str], offset: usize) -> (usize, usize) {
    let mut n = 0;
    for (i, line) in lines.iter().enumerate() {
        let len = line.width() + 1;
        if n + len > offset {
            return (i, offset - n);
        }
        n += len;
    }
    (lines.len(), lines.last().map(|l| l.width()).unwrap_or(0))
}

impl Error {
    pub(crate) fn new(msg: impl Into<Cow<'static, str>>, span: impl Into<Span>) -> Self {
        Self {
            span: span.into(),
            msg: msg.into(),
        }
    }

    pub fn pretty(&self, input: &str, filename: impl AsRef<Path>) -> String {
        let Self { span, msg } = self;

        let lines: Vec<_> = input.split_terminator('\n').collect();

        let (line, col) = to_line_col(&lines, span.m);
        let width = max(1, span.as_str(input).width());
        let code = lines.get(line).unwrap_or_else(|| lines.last().unwrap());
        let error = format!(
            "{underline:>pad$} {msg}",
            underline = Paint::red("^".repeat(width)).bold(),
            msg = Paint::default(msg).bold(),
            pad = col + width,
        );

        let num = (line + 1).to_string();
        format!(
            "\n\
             {0:pad$} {arrow} {filename}:{line}:{col}\n \
             {0:pad$} {pipe}\n \
             {num:>} {pipe} {code}\n \
             {0:pad$} {pipe} {error}\n",
            "",
            pad = num.width(),
            arrow = Paint::blue("-->").bold(),
            filename = filename.as_ref().display(),
            line = line + 1,
            col = col + 1,
            pipe = Paint::blue("|").bold(),
            num = Paint::blue(num).bold(),
            code = code,
            error = error,
        )
    }
}
