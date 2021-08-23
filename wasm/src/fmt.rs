use std::fmt::{self, Display};

use intcode::fmt::{Context, Mark, Options, Paint};

pub struct Html;

impl Html {
    pub fn new<'i>(input: &'i str) -> Options<'i, Self> {
        Options::new(input, "&lt;input&gt;")
    }
}

impl Paint for Html {
    fn fmt<D: Display>(
        this: D,
        ctx: Context,
        mark: Mark,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let class = match (ctx, mark) {
            (Context::Error, Mark::Message | Mark::Underline) => "hl-error",
            (Context::Warning, Mark::Message | Mark::Underline) => "hl-warning",
            (_, Mark::Margin) => "hl-blue",
            (_, _) => "hl-white",
        };
        write!(f, "<span class='{}'>", class)?;
        Display::fmt(&this, f)?;
        write!(f, "</span>")?;
        Ok(())
    }
}
