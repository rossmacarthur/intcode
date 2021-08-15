use std::cmp::max;
use std::ops::Range;
use std::path::Path;

use dairy::Cow;
use unicode_width::UnicodeWidthStr;
use yansi::{Color, Paint};

#[derive(Debug)]
pub struct Pretty<'i> {
    input: &'i str,
    filename: Cow<'i, Path>,
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

impl<'i> Pretty<'i> {
    pub fn new(input: &'i str) -> Self {
        Self {
            input,
            filename: "<input>".into(),
        }
    }

    pub fn filename(mut self, path: impl Into<Cow<'i, Path>>) -> Self {
        self.filename = path.into();
        self
    }

    fn with_color<'a>(
        &self,
        color: Color,
        msg: impl Into<Cow<'a, str>>,
        span: impl Into<Range<usize>>,
    ) -> String {
        let msg = msg.into();
        let span = span.into();

        let lines: Vec<_> = self.input.split_terminator('\n').collect();
        let (line, col) = to_line_col(&lines, span.start);
        let width = max(1, self.input[span].width());
        let code = lines.get(line).unwrap_or_else(|| lines.last().unwrap());
        let error = format!(
            "{underline:>pad$} {msg}",
            underline = Paint::new("^".repeat(width)).fg(color).bold(),
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
            filename = self.filename.display(),
            line = line + 1,
            col = col + 1,
            pipe = Paint::blue("|").bold(),
            num = Paint::blue(num).bold(),
            code = code,
            error = error,
        )
    }

    pub fn error<'a>(&self, msg: impl Into<Cow<'a, str>>, span: impl Into<Range<usize>>) -> String {
        self.with_color(Color::Red, msg, span)
    }

    pub fn warn<'a>(&self, msg: impl Into<Cow<'a, str>>, span: impl Into<Range<usize>>) -> String {
        self.with_color(Color::Yellow, msg, span)
    }
}
