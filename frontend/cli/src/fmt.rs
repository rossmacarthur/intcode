use std::fmt;
use std::fmt::Display;
use std::path::Path;

use intcode::error::fmt::{Context, Mark, Options, Paint};

pub struct Ansi;

impl Ansi {
    pub fn new<'i>(input: &'i str, path: &'i Path) -> Options<'i, Self> {
        Options::new(input, path)
    }
}

impl Paint for Ansi {
    fn fmt<D: Display>(
        this: D,
        ctx: Context,
        mark: Mark,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let paint = match (ctx, mark) {
            (_, Mark::Margin) => yansi::Paint::blue(this).bold(),
            (Context::Warning, Mark::Underline) => yansi::Paint::yellow(this).bold(),
            (Context::Error, Mark::Underline) => yansi::Paint::red(this).bold(),
            (_, Mark::Message) => yansi::Paint::default(this).bold(),
            (_, _) => yansi::Paint::default(this),
        };
        Display::fmt(&paint, f)?;
        Ok(())
    }
}
