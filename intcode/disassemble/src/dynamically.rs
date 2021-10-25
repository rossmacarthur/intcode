use std::cmp::max;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::iter;
use std::result;

use thiserror::Error;

use crate::ast::Mode;
use crate::program::{Mention, Opcode, Program, Purpose};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown mode `{}`", .mode)]
    UnknownMode { mode: i64 },
    #[error("unknown opcode `{}`", .opcode)]
    UnknownOpcode { opcode: i64 },
    #[error("failed to cast `{}` as `usize`", .num)]
    BadConversion { num: i64 },
    #[error("run requires more input")]
    WantInput,
}

#[derive(Debug)]
enum State {
    Yielded(i64),
    Waiting,
    Complete,
}

#[derive(Debug)]
struct Computer<'a> {
    prog: &'a mut Program,
    mem: Vec<i64>,
    ptr: usize,
    relative_base: i64,
    input: VecDeque<i64>,
}

#[derive(Debug)]
pub enum Input {
    /// A series of exact inputs to provide the computer.
    Static(Vec<i64>),
    /// Always provide the value as input if the machine asks for input.
    Forever(i64),
}

#[derive(Debug, Default)]
pub struct Run {
    input: Option<Input>,
}

fn cast(num: i64) -> Result<usize> {
    usize::try_from(num).map_err(|_| Error::BadConversion { num })
}

impl<'a> Computer<'a> {
    fn new(prog: &'a mut Program) -> Self {
        let mem = prog.original();
        Self {
            prog,
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
        self.mem.get(addr).copied().unwrap_or(0)
    }

    fn mem_get_mut(&mut self, addr: usize) -> &mut i64 {
        let new_len = max(self.mem.len(), addr + 1);
        self.mem.resize(new_len, 0);
        self.mem.get_mut(addr).unwrap()
    }

    fn param_ptr(&mut self, i: usize, purpose: Purpose) -> Result<usize> {
        let opcode = self.mem_get(self.ptr);
        let ptr = self.ptr + i;
        let divs = [10, 100, 1_000, 10_000];
        let mode = opcode / divs[i] % 10;
        let mode = Mode::from_value(mode).ok_or(Error::UnknownMode { mode })?;
        self.prog.mark_param(ptr, mode);
        match mode {
            Mode::Positional => {
                let addr = cast(self.mem_get(ptr))?;
                let mention = Mention::new(purpose, ptr);
                self.prog.mention(addr, mention);
                Ok(addr)
            }
            Mode::Immediate => {
                // We don't need to mention the referrer here because immediate
                // values refer to themselves.
                Ok(ptr)
            }
            Mode::Relative => {
                let addr = cast(self.relative_base + self.mem_get(ptr))?;
                let mention = Mention::new(purpose, ptr);
                self.prog.mention(addr, mention);
                Ok(addr)
            }
        }
    }

    fn param(&mut self, i: usize) -> Result<i64> {
        self.param_ptr(i, Purpose::Read)
            .map(move |addr| self.mem_get(addr))
    }

    fn param_mut(&mut self, i: usize) -> Result<&mut i64> {
        self.param_ptr(i, Purpose::Write)
            .map(move |addr| self.mem_get_mut(addr))
    }

    fn jump_param(&mut self, i: usize) -> Result<usize> {
        let addr = cast(self.param(i)?)?;
        let mention = Mention::new(Purpose::Jump, self.ptr + i);
        self.prog.mention(addr, mention);
        Ok(addr)
    }

    fn next(&mut self) -> Result<State> {
        loop {
            let opcode = self.mem_get(self.ptr) % 100;
            let opcode = Opcode::from_value(opcode).ok_or(Error::UnknownOpcode { opcode })?;
            self.prog.mark_opcode(self.ptr, opcode);
            match opcode {
                Opcode::Add => {
                    *self.param_mut(3)? = self.param(1)? + self.param(2)?;
                    self.ptr += 4;
                }
                Opcode::Multiply => {
                    *self.param_mut(3)? = self.param(1)? * self.param(2)?;
                    self.ptr += 4;
                }
                Opcode::Input => {
                    if let Some(input) = self.input.pop_front() {
                        *self.param_mut(1)? = input;
                        self.ptr += 2;
                    } else {
                        break Ok(State::Waiting);
                    }
                }
                Opcode::Output => {
                    let output = self.param(1)?;
                    self.ptr += 2;
                    break Ok(State::Yielded(output));
                }
                Opcode::JumpNonZero => {
                    // Make sure to read this parameter so it gets marked.
                    let addr = self.jump_param(2)?;
                    if self.param(1)? != 0 {
                        self.ptr = addr;
                    } else {
                        self.ptr += 3;
                    }
                }
                Opcode::JumpZero => {
                    // Make sure to read this parameter so it gets marked.
                    let addr = self.jump_param(2)?;
                    if self.param(1)? == 0 {
                        self.ptr = addr;
                    } else {
                        self.ptr += 3;
                    }
                }
                Opcode::LessThan => {
                    *self.param_mut(3)? = (self.param(1)? < self.param(2)?) as i64;
                    self.ptr += 4;
                }
                Opcode::Equal => {
                    *self.param_mut(3)? = (self.param(1)? == self.param(2)?) as i64;
                    self.ptr += 4;
                }
                Opcode::AdjustRelativeBase => {
                    self.relative_base += self.param(1)?;
                    self.ptr += 2;
                }
                Opcode::Halt => {
                    break Ok(State::Complete);
                }
                _ => {
                    // Opcode::from_value(..) doesn't yield any other variants
                    unreachable!()
                }
            }
        }
    }

    fn reset(&mut self) {
        self.mem = self.prog.original();
        self.ptr = 0;
        self.relative_base = 0;
        self.input = VecDeque::new();
    }
}

impl Run {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn input(mut self, i: Input) -> Self {
        self.input = Some(i);
        self
    }

    /// Run the program once with the provided input.
    pub fn once(i: Input) -> impl IntoIterator<Item = Run> {
        [Self::new().input(i)]
    }

    /// Run the program twice with two single numbers as inputs.
    pub fn twice(a: Input, b: Input) -> impl IntoIterator<Item = Run> {
        [Self::new().input(a), Self::new().input(b)]
    }
}

/// Dynamically mark the program by actually running it and seeing what each
/// memory location is for.
pub fn mark(p: &mut Program, runs: impl IntoIterator<Item = Run>) -> Result<()> {
    let mut c = Computer::new(p);

    // Run the program and mark the memory appropriately.
    for Run { mut input } in runs.into_iter() {
        loop {
            match c.next()? {
                State::Yielded(_) => {
                    continue;
                }
                State::Waiting => match &mut input {
                    Some(Input::Forever(v)) => c.feed(iter::once(*v)),
                    Some(Input::Static(v)) => c.feed(v.drain(..)),
                    None => return Err(Error::WantInput),
                },
                State::Complete => {
                    break;
                }
            }
        }
        c.reset();
    }

    Ok(())
}
