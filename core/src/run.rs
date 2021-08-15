mod io;

use std::cmp::max;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::prelude::*;
use std::result;

use thiserror::Error;

type Result<T> = result::Result<T, Error>;

fn cast(num: i64) -> usize {
    usize::try_from(num).unwrap()
}

fn parse_program(input: &str) -> Vec<i64> {
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
enum State {
    /// An output.
    Yielded(i64),
    /// Waiting for input.
    Waiting,
    /// Program execution has finished.
    Complete,
}

#[derive(Debug)]
struct Computer {
    mem: Vec<i64>,
    ptr: usize,
    relative_base: i64,
    input: VecDeque<i64>,
}

impl Computer {
    fn new(program: Vec<i64>) -> Self {
        Self {
            mem: program,
            ptr: 0,
            relative_base: 0,
            input: VecDeque::new(),
        }
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

    fn next(&mut self) -> Result<State> {
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

fn run(input: &str, io: impl io::Io) -> Result<()> {
    let mut c = Computer::new(parse_program(input));
    let mut stdin = io::BufReader::new(io::stdin());
    let mut stdout = io::BufWriter::new(io::stdout());
    loop {
        match c.next()? {
            State::Yielded(value) => {
                io.output(&mut stdout, value)?;
                stdout.flush()?;
            }
            State::Waiting => {
                stdout.flush()?;
                c.input.extend(io.input(&mut stdin)?);
            }
            State::Complete => {
                stdout.flush()?;
                break Ok(());
            }
        }
    }
}

/// Run the provided intcode program.
pub fn intcode(input: &str) -> Result<()> {
    run(input, io::Basic)
}

/// Run the provided intcode program.
pub fn intcode_utf8(input: &str) -> Result<()> {
    run(input, io::Utf8)
}
