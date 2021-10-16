mod ast;
mod fmt;
mod mem;

use std::cmp::max;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::{iter, result};

use crate::ast::{Mode, Program};
use crate::mem::{Memory, Opcode};

use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown mode `{}`", .mode)]
    UnknownMode { mode: i64 },
    #[error("unknown opcode `{}`", .opcode)]
    UnknownOpcode { opcode: i64 },
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
    mem: Memory,
    ptr: usize,
    relative_base: i64,
    input: VecDeque<i64>,
}

fn cast(num: i64) -> usize {
    usize::try_from(num).unwrap()
}

impl Computer {
    fn new(intcode: Vec<i64>) -> Self {
        let mem = Memory::new(intcode);
        Self {
            mem,
            ptr: 0,
            relative_base: 0,
            input: VecDeque::new(),
        }
    }

    fn feed(&mut self, iter: impl IntoIterator<Item = i64>) {
        self.input.extend(iter)
    }

    fn mem_get(&self, addr: usize) -> i64 {
        self.mem.get(addr).unwrap_or(0)
    }

    fn mem_get_mut(&mut self, addr: usize) -> &mut i64 {
        let new_len = max(self.mem.len(), addr + 1);
        self.mem.resize(new_len);
        self.mem.get_mut(addr).unwrap()
    }

    fn param_ptr(&mut self, i: usize) -> Result<usize> {
        let opcode = self.mem_get(self.ptr);
        let ptr = self.ptr + i;
        match opcode / (10i64.pow((1 + i) as u32)) % 10 {
            0 => {
                self.mem.mark_param(ptr, Mode::Positional);
                Ok(cast(self.mem_get(ptr)))
            }
            1 => {
                self.mem.mark_param(ptr, Mode::Immediate);
                Ok(ptr)
            }
            2 => {
                self.mem.mark_param(ptr, Mode::Relative);
                Ok(cast(self.relative_base + self.mem_get(ptr)))
            }
            mode => Err(Error::UnknownMode { mode }),
        }
    }

    fn param(&mut self, i: usize) -> Result<i64> {
        self.param_ptr(i).map(move |ptr| self.mem_get(ptr))
    }

    fn param_mut(&mut self, i: usize) -> Result<&mut i64> {
        self.param_ptr(i).map(move |ptr| self.mem_get_mut(ptr))
    }

    fn next(&mut self) -> Result<State> {
        loop {
            match self.mem_get(self.ptr) % 100 {
                1 => {
                    self.mem.mark_opcode(self.ptr, Opcode::Add);
                    *self.param_mut(3)? = self.param(1)? + self.param(2)?;
                    self.ptr += 4;
                }
                2 => {
                    self.mem.mark_opcode(self.ptr, Opcode::Multiply);
                    *self.param_mut(3)? = self.param(1)? * self.param(2)?;
                    self.ptr += 4;
                }
                3 => {
                    self.mem.mark_opcode(self.ptr, Opcode::Input);
                    if let Some(input) = self.input.pop_front() {
                        *self.param_mut(1)? = input;
                        self.ptr += 2;
                    } else {
                        break Ok(State::Waiting);
                    }
                }
                4 => {
                    self.mem.mark_opcode(self.ptr, Opcode::Output);
                    let output = self.param(1)?;
                    self.ptr += 2;
                    break Ok(State::Yielded(output));
                }
                5 => {
                    self.mem.mark_opcode(self.ptr, Opcode::JumpNonZero);
                    // Make sure to read this parameter so it gets marked.
                    let addr = self.param(2)?;
                    if self.param(1)? != 0 {
                        self.ptr = cast(addr);
                    } else {
                        self.ptr += 3;
                    }
                }
                6 => {
                    self.mem.mark_opcode(self.ptr, Opcode::JumpZero);
                    // Make sure to read this parameter so it gets marked.
                    let addr = self.param(2)?;
                    if self.param(1)? == 0 {
                        self.ptr = cast(addr);
                    } else {
                        self.ptr += 3;
                    }
                }
                7 => {
                    self.mem.mark_opcode(self.ptr, Opcode::LessThan);
                    *self.param_mut(3)? = (self.param(1)? < self.param(2)?) as i64;
                    self.ptr += 4;
                }
                8 => {
                    self.mem.mark_opcode(self.ptr, Opcode::Equal);
                    *self.param_mut(3)? = (self.param(1)? == self.param(2)?) as i64;
                    self.ptr += 4;
                }
                9 => {
                    self.mem.mark_opcode(self.ptr, Opcode::AdjustRelativeBase);
                    self.relative_base += self.param(1)?;
                    self.ptr += 2;
                }
                99 => {
                    self.mem.mark_opcode(self.ptr, Opcode::Halt);
                    break Ok(State::Complete);
                }
                opcode => break Err(Error::UnknownOpcode { opcode }),
            }
        }
    }

    fn reset(&mut self) {
        self.mem.reset();
        self.ptr = 0;
        self.relative_base = 0;
        self.input = VecDeque::new();
    }

    fn into_memory(self) -> Memory {
        self.mem
    }
}

#[derive(Debug, Default)]
pub struct Run {
    input: Vec<i64>,
}

impl Run {
    /// Run the program once with a single zero as input.
    pub fn once() -> impl IntoIterator<Item = Run> {
        iter::once(Self { input: vec![0] })
    }

    /// Run the program twice with two single numbers as inputs.
    pub fn twice(a: i64, b: i64) -> impl IntoIterator<Item = Run> {
        [Self { input: vec![a] }, Self { input: vec![b] }]
    }

    fn input(self) -> impl IntoIterator<Item = i64> {
        self.input
    }
}

/// Disassemble the intcode program into an AST that can be displayed.
pub fn to_ast(intcode: Vec<i64>, runs: impl IntoIterator<Item = Run>) -> Result<Program> {
    let mut c = Computer::new(intcode);

    // Run the program and mark the memory appropriately.
    for (i, run) in runs.into_iter().enumerate() {
        c.feed(run.input());
        loop {
            match c.next()? {
                State::Yielded(_) => {
                    continue;
                }
                State::Waiting => {
                    panic!("require more input on run {}", i);
                }
                State::Complete => {
                    break;
                }
            }
        }
        c.reset();
    }

    Ok(c.into_memory().into_ast())
}
