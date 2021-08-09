use std::convert::TryInto;
pub use std::io::*;

use super::parse_program;

pub trait Io {
    fn input(&self, i: impl BufRead) -> Result<Vec<i64>>;
    fn output(&self, o: impl Write, value: i64) -> Result<()>;
}

pub struct Basic;
pub struct Utf8;

impl Io for Basic {
    fn input(&self, i: impl BufRead) -> Result<Vec<i64>> {
        let mut values = Vec::new();
        for line in i.lines() {
            values.extend(parse_program(&line?));
        }
        Ok(values)
    }

    fn output(&self, mut o: impl Write, value: i64) -> Result<()> {
        o.write_all(format!("{}\n", value).as_bytes())
    }
}

impl Io for Utf8 {
    fn input(&self, i: impl BufRead) -> Result<Vec<i64>> {
        let mut values = Vec::new();
        for line in i.lines() {
            values.extend(Vec::from(line?).into_iter().map(i64::from));
        }
        Ok(values)
    }

    fn output(&self, mut o: impl Write, value: i64) -> Result<()> {
        o.write_all(&[value.try_into().unwrap()])
    }
}
