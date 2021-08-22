mod io;

use std::cmp::max;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::prelude::*;
use std::result;

use thiserror::Error;

type Result<T> = result::Result<T, Error>;

pub fn parse_program(input: &str) -> Vec<i64> {
    input
        .trim()
        .split(',')
        .map(str::parse)
        .map(result::Result::unwrap)
        .collect()
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown mode `{}`", .mode)]
    UnknownMode { mode: i64 },
    #[error("unknown opcode `{}`", .opcode)]
    UnknownOpcode { opcode: i64 },
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// The state of the computer.
#[derive(Debug)]
pub enum State {
    /// An output.
    Yielded(i64),
    /// Waiting for input.
    Waiting,
    /// Program execution has finished.
    Complete,
}

#[derive(Debug)]
pub struct Computer {
    mem: Vec<i64>,
    ptr: usize,
    relative_base: i64,
    input: VecDeque<i64>,
}

fn cast(num: i64) -> usize {
    usize::try_from(num).unwrap()
}

impl Computer {
    pub fn new(program: Vec<i64>) -> Self {
        Self {
            mem: program,
            ptr: 0,
            relative_base: 0,
            input: VecDeque::new(),
        }
    }

    pub fn feed(&mut self, iter: impl Iterator<Item = i64>) {
        self.input.extend(iter)
    }

    fn mem_get(&self, addr: usize) -> i64 {
        self.mem.get(addr).copied().unwrap_or(0)
    }

    fn mem_get_mut(&mut self, addr: usize) -> &mut i64 {
        self.mem.resize(max(self.mem.len(), addr + 1), 0);
        &mut self.mem[addr]
    }

    fn param_ptr(&self, i: usize) -> Result<usize> {
        let opcode = self.mem_get(self.ptr);
        let ptr = self.ptr + i;
        match opcode / (10i64.pow((1 + i) as u32)) % 10 {
            0 => Ok(cast(self.mem_get(ptr))),
            1 => Ok(ptr),
            2 => Ok(cast(self.relative_base + self.mem_get(ptr))),
            mode => Err(Error::UnknownMode { mode }),
        }
    }

    fn param(&self, i: usize) -> Result<i64> {
        self.param_ptr(i).map(move |ptr| self.mem_get(ptr))
    }

    fn param_mut(&mut self, i: usize) -> Result<&mut i64> {
        self.param_ptr(i).map(move |ptr| self.mem_get_mut(ptr))
    }

    pub fn next(&mut self) -> Result<State> {
        loop {
            match self.mem_get(self.ptr) % 100 {
                1 => {
                    *self.param_mut(3)? = self.param(1)? + self.param(2)?;
                    self.ptr += 4;
                }
                2 => {
                    *self.param_mut(3)? = self.param(1)? * self.param(2)?;
                    self.ptr += 4;
                }
                3 => {
                    if let Some(input) = self.input.pop_front() {
                        *self.param_mut(1)? = input;
                        self.ptr += 2;
                    } else {
                        break Ok(State::Waiting);
                    }
                }
                4 => {
                    let output = self.param(1)?;
                    self.ptr += 2;
                    break Ok(State::Yielded(output));
                }
                5 => {
                    if self.param(1)? != 0 {
                        self.ptr = cast(self.param(2)?);
                    } else {
                        self.ptr += 3;
                    }
                }
                6 => {
                    if self.param(1)? == 0 {
                        self.ptr = cast(self.param(2)?);
                    } else {
                        self.ptr += 3;
                    }
                }
                7 => {
                    *self.param_mut(3)? = (self.param(1)? < self.param(2)?) as i64;
                    self.ptr += 4;
                }
                8 => {
                    *self.param_mut(3)? = (self.param(1)? == self.param(2)?) as i64;
                    self.ptr += 4;
                }
                9 => {
                    self.relative_base += self.param(1)?;
                    self.ptr += 2;
                }
                99 => break Ok(State::Complete),
                opcode => break Err(Error::UnknownOpcode { opcode }),
            }
        }
    }
}

pub struct Runner<I, O: Write> {
    input: io::BufReader<I>,
    output: io::BufWriter<O>,
}

impl<I: Read, O: Write> Runner<I, O> {
    pub fn new(i: I, o: O) -> Self {
        Self {
            input: io::BufReader::new(i),
            output: io::BufWriter::new(o),
        }
    }

    fn run<T: io::Kind>(mut self, intcode: Vec<i64>) -> Result<()> {
        let mut c = Computer::new(intcode);
        loop {
            match c.next()? {
                State::Yielded(value) => {
                    T::output(&mut self.output, value)?;
                }
                State::Waiting => {
                    self.output.flush()?;
                    c.input.extend(T::input(&mut self.input)?);
                }
                State::Complete => {
                    self.output.flush()?;
                    break Ok(());
                }
            }
        }
    }

    pub fn run_basic(self, intcode: Vec<i64>) -> Result<()> {
        self.run::<io::Basic>(intcode)
    }

    pub fn run_utf8(self, intcode: Vec<i64>) -> Result<()> {
        self.run::<io::Utf8>(intcode)
    }
}
