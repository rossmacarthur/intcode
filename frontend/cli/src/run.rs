use std::convert::TryInto;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};

use anyhow::Result;
use intcode::run::{Computer, State};

use crate::parse_program;

pub fn basic(intcode: Vec<i64>) -> Result<()> {
    let mut c = Computer::new(intcode);
    let mut r = BufReader::new(io::stdin());
    let mut w = BufWriter::new(io::stdout());
    loop {
        match c.next()? {
            State::Yielded(value) => {
                writeln!(w, "{}", value)?;
            }
            State::Waiting => {
                w.flush()?;
                let mut line = String::new();
                r.read_line(&mut line)?;
                c.feed(parse_program(&line)?);
            }
            State::Complete => {
                break Ok(w.flush()?);
            }
        }
    }
}

pub fn utf8(intcode: Vec<i64>) -> Result<()> {
    let mut c = Computer::new(intcode);
    let mut r = BufReader::new(io::stdin());
    let mut w = BufWriter::new(io::stdout());
    loop {
        match c.next()? {
            State::Yielded(value) => {
                w.write_all(&[value.try_into()?])?;
            }
            State::Waiting => {
                w.flush()?;
                let mut line = String::new();
                r.read_line(&mut line)?;
                c.feed(line.bytes().map(i64::from));
            }
            State::Complete => {
                break Ok(w.flush()?);
            }
        }
    }
}
