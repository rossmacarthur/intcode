use std::path::Path;

use dairy::Cow;
use peter::Stylize;
use thiserror::Error;

use crate::span::Span;

pub type Result<T> = std::result::Result<T, Error>;

/// A parse error.
///
/// This can be an unexpected character, token, or value. The span specifies
/// what will be underlined in the error message. The message is what will be
/// displayed in the formatted output.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{msg}")]
pub struct Error {
    msg: Cow<'static, str>,
    span: Span,
}

fn to_line_col(lines: &[&str], offset: usize) -> (usize, usize) {
    let mut n = 0;
    for (i, line) in lines.iter().enumerate() {
        let len = line.chars().count() + 1;
        if n + len > offset {
            return (i, offset - n);
        }
        n += len;
    }
    (
        lines.len(),
        lines.last().map(|l| l.chars().count()).unwrap_or(0),
    )
}

impl Error {
    pub(crate) fn new(msg: impl Into<Cow<'static, str>>, span: impl Into<Span>) -> Self {
        Self {
            span: span.into(),
            msg: msg.into(),
        }
    }

    pub fn pretty(&self, input: &str, filename: &Path) -> String {
        let Self { span, msg } = self;

        let lines: Vec<_> = input.split_terminator('\n').collect();

        let (line, col) = to_line_col(&lines, span.m);
        let width = span.width();
        let code = lines.get(line).unwrap_or_else(|| lines.last().unwrap());
        let error = format!(
            "{underline:>pad$} {msg}",
            underline = "^".repeat(width).bold().red(),
            msg = msg.bold(),
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
            pad = num.chars().count(),
            arrow = "-->".bold().blue(),
            filename = filename.display(),
            line = line + 1,
            col = col + 1,
            pipe = "|".bold().blue(),
            num = num.bold().blue(),
            code = code,
            error = error,
        )
    }
}
