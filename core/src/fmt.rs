use std::cmp::max;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Range;
use std::path::Path;

use dairy::Cow;
use unicode_width::UnicodeWidthStr;

use crate::error::{Error, Warning};
use crate::span::Span;

pub trait Paint {
    fn fmt<D: Display>(
        this: D,
        ctx: Context,
        mark: Mark,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result;
}

#[derive(Debug, Clone, Copy)]
pub enum Context {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy)]
pub enum Mark {
    Default,
    Margin,
    Underline,
    Message,
}

pub struct Options<'i, P> {
    input: &'i str,
    filename: Cow<'i, Path>,
    mark: PhantomData<P>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Plain;

impl Paint for Plain {
    fn fmt<D: Display>(this: D, _: Context, _: Mark, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&this, f)
    }
}

impl Plain {
    pub fn new<'i>(input: &'i str) -> Options<'i, Self> {
        Options::new(input, "<input>")
    }
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

impl<'i, P: Paint> Options<'i, P> {
    pub fn new(input: &'i str, filename: impl Into<Cow<'i, Path>>) -> Self {
        Self {
            mark: PhantomData,
            input,
            filename: filename.into(),
        }
    }

    fn fmt(&self, ctx: Context, msg: &Cow<'_, str>, span: Span) -> String {
        struct Painted<P, D> {
            paint: PhantomData<P>,
            display: D,
            ctx: Context,
            mark: Mark,
        }

        impl<P: Paint, D: Display> fmt::Display for Painted<P, D> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                P::fmt(&self.display, self.ctx, self.mark, f)
            }
        }

        let span: Range<usize> = span.into();
        let lines: Vec<_> = self.input.split_terminator('\n').collect();
        let (line, col) = to_line_col(&lines, span.start);
        let width = max(1, self.input[span].width());
        let code = lines.get(line).unwrap_or_else(|| lines.last().unwrap());

        macro_rules! mark {
            ($mark:ident, $display:expr) => {
                Painted {
                    paint: PhantomData::<P>,
                    ctx,
                    mark: Mark::$mark,
                    display: $display,
                }
            };
        }

        let num = (line + 1).to_string();
        let pad = num.width();
        let num = mark!(Margin, num);
        let arrow = mark!(Margin, "-->");
        let pipe = mark!(Margin, "|");
        let underline = mark!(Underline, "^".repeat(width));
        let msg = mark!(Message, msg);

        format!(
            "\n\
            {0:pad$} {arrow} {filename}:{line}:{col}\n \
            {0:pad$} {pipe}\n \
            {num:>} {pipe} {code}\n \
            {0:pad$} {pipe} {underline:>width$} {msg}\n",
            "",
            pad = pad,
            arrow = arrow,
            filename = self.filename.display(),
            line = line + 1,
            col = col + 1,
            pipe = pipe,
            num = num,
            code = code,
            underline = underline,
            width = col + width,
            msg = msg
        )
    }

    pub fn error(&self, error: &Error) -> String {
        self.fmt(Context::Error, &error.msg, error.span)
    }

    pub fn warning(&self, warning: &Warning) -> String {
        self.fmt(Context::Warning, &warning.msg, warning.span)
    }
}
