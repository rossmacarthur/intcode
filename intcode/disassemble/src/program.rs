use std::collections::HashSet;
use std::iter;

use crate::ast::{Ast, Instr, Label, Mode, Param, RawParam, Stmt};

/// An instruction type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    Add,
    Multiply,
    LessThan,
    Equal,
    JumpNonZero,
    JumpZero,
    Input,
    Output,
    AdjustRelativeBase,
    Halt,
    Mutable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mark {
    Opcode(Opcode),
    Param(Param),
    String,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Purpose {
    Read,
    Write,
    Jump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mention {
    pub purpose: Purpose,
    pub referrer: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Slot {
    /// The raw value in the original program.
    pub raw: i64,
    /// An optional mark when we are confident about this memory location.
    pub mark: Option<Mark>,
    /// How this address is mentioned from other places in the program.
    pub mentions: HashSet<Mention>,
    /// An optional label if we add one.
    pub label: Option<Label>,
}

/// Represents an intcode program during our analysis.
#[derive(Debug, Clone)]
pub struct Program {
    pub slots: Vec<Slot>,
}

impl Mode {
    pub fn from_value(v: i64) -> Option<Mode> {
        Some(match v {
            0 => Self::Positional,
            1 => Self::Immediate,
            2 => Self::Relative,
            _ => return None,
        })
    }
}

impl Opcode {
    pub fn from_value(v: i64) -> Option<Opcode> {
        Some(match v {
            1 => Self::Add,
            2 => Self::Multiply,
            3 => Self::Input,
            4 => Self::Output,
            5 => Self::JumpNonZero,
            6 => Self::JumpZero,
            7 => Self::LessThan,
            8 => Self::Equal,
            9 => Self::AdjustRelativeBase,
            99 => Self::Halt,
            _ => return None,
        })
    }

    pub fn params(&self) -> usize {
        match self {
            Self::Add => 3,
            Self::Multiply => 3,
            Self::Input => 1,
            Self::Output => 1,
            Self::JumpNonZero => 2,
            Self::JumpZero => 2,
            Self::LessThan => 3,
            Self::Equal => 3,
            Self::AdjustRelativeBase => 1,
            Self::Halt => 0,
            i => panic!("no params for `{:?}`", i),
        }
    }

    fn value(&self) -> i64 {
        match self {
            Self::Add => 1,
            Self::Multiply => 2,
            Self::Input => 3,
            Self::Output => 4,
            Self::JumpNonZero => 5,
            Self::JumpZero => 6,
            Self::LessThan => 7,
            Self::Equal => 8,
            Self::AdjustRelativeBase => 9,
            Self::Halt => 99,
            i => panic!("no opcode for `{:?}`", i),
        }
    }
}

impl Mention {
    pub fn new(purpose: Purpose, referrer: usize) -> Self {
        Self { purpose, referrer }
    }
}

impl Slot {
    pub fn is_marked(&self) -> bool {
        self.mark.is_some()
    }

    pub fn is_unmarked(&self) -> bool {
        self.mark.is_none()
    }

    pub fn is_unlabelled(&self) -> bool {
        self.label.is_none()
    }

    pub fn has_rw_purpose(&self) -> bool {
        self.mentions
            .iter()
            .any(|m| matches!(m.purpose, Purpose::Read | Purpose::Write))
    }

    pub fn has_jump_purpose(&self) -> bool {
        self.mentions
            .iter()
            .any(|m| matches!(m.purpose, Purpose::Jump))
    }
}

impl Program {
    pub fn new(intcode: Vec<i64>) -> Self {
        let slots = intcode
            .into_iter()
            .map(|raw| Slot {
                raw,
                ..Default::default()
            })
            .collect();
        Self { slots }
    }

    pub fn original(&self) -> Vec<i64> {
        self.slots.iter().map(|slot| slot.raw).collect()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn percent_marked(&self) -> f64 {
        let count = self.slots.iter().filter(|s| s.is_marked()).count() as f64;
        let total = self.len() as f64;
        100.0 * count / total
    }

    pub fn mention(&mut self, addr: usize, mention: Mention) {
        if addr >= self.len() {
            return;
        }
        self.slots[addr].mentions.insert(mention);
    }

    pub fn mark(&mut self, addr: usize, mark: Mark) {
        if addr >= self.len() {
            panic!(
                "tried to mark address `{}` as `{:?}` but it doesn't exist in the original",
                addr, mark
            )
        }
        let slot = &mut self.slots[addr];
        match &mut slot.mark {
            // This address is already marked with the same param 👍.
            Some(m) if *m == mark => {}
            // This address is unmarked, mark it with the given param.
            m @ None => *m = Some(mark),
            // Otherwise, this is address is already marked as something
            // else, so we panic.
            Some(m) => panic!(
                "tried to mark address `{}` as `{:?}` but it is already marked with `{:?}`",
                addr, mark, m
            ),
        }
    }

    pub fn mark_opcode(&mut self, addr: usize, opcode: Opcode) {
        if addr >= self.len() {
            panic!(
                "tried to mark address `{}` with opcode `{:?}` but it doesn't exist in the original",
                addr, opcode
            )
        }
        let slot = &mut self.slots[addr];
        match &mut slot.mark {
            // This address is already marked with the same opcode 👍.
            Some(Mark::Opcode(o)) if *o == opcode => {}
            // If this address is already marked with a different opcode
            // then mark it as a "mutable" opcode.
            Some(Mark::Opcode(o)) => *o = Opcode::Mutable,
            // We are marking this address with an opcode that does not
            // match the original value, so this is also a "mutable"
            // opcode.
            m @ None if slot.raw % 100 != opcode.value() => {
                *m = Some(Mark::Opcode(Opcode::Mutable))
            }
            // This address is unmarked, mark it with the given opcode.
            m @ None => *m = Some(Mark::Opcode(opcode)),
            // Otherwise, this is address is already marked as something
            // else, so we panic.
            Some(m) => {
                panic!(
                    "tried to mark address `{}` with opcode `{:?}`, but it is already marked with `{:?}`",
                    addr, opcode, m
                );
            }
        }
    }

    pub fn mark_param(&mut self, addr: usize, mode: Mode) {
        if addr >= self.len() {
            panic!(
                "tried to mark address `{}` with parameter `{:?}` but it doesn't exist in the original",
                addr, mode
            )
        }
        let slot = &mut self.slots[addr];
        let mark = Mark::Param(Param::Number(mode, slot.raw));
        self.mark(addr, mark);
    }

    pub fn get_param(&self, addr: usize) -> Option<Param> {
        self.slots.get(addr).and_then(|slot| match &slot.mark {
            Some(Mark::Param(param)) => Some(param.clone()),
            _ => None,
        })
    }

    fn bucket_unlabelled(&self, mut ptr: usize, mark: Mark) -> Vec<i64> {
        let mut v = vec![self.slots[ptr].raw];
        ptr += 1;
        while matches!(self.slots.get(ptr), Some(Slot { mark: Some(m), label: None, .. }) if *m == mark)
        {
            v.push(self.slots[ptr].raw);
            ptr += 1;
        }
        v
    }

    pub fn into_ast(self) -> Ast {
        let mut ptr = 0;
        let mut stmts = Vec::new();

        while let Some(slot) = self.slots.get(ptr) {
            match &slot.mark {
                Some(Mark::Opcode(opcode)) => {
                    let param = |i: usize| self.get_param(ptr + i).unwrap();
                    let instr = match opcode {
                        Opcode::Add => {
                            let a = param(1);
                            let b = param(2);
                            let c = param(3);
                            ptr += 4;
                            Instr::Add(a, b, c)
                        }
                        Opcode::Multiply => {
                            let a = param(1);
                            let b = param(2);
                            let c = param(3);
                            ptr += 4;
                            Instr::Multiply(a, b, c)
                        }
                        Opcode::LessThan => {
                            let a = param(1);
                            let b = param(2);
                            let c = param(3);
                            ptr += 4;
                            Instr::LessThan(a, b, c)
                        }
                        Opcode::Equal => {
                            let a = param(1);
                            let b = param(2);
                            let c = param(3);
                            ptr += 4;
                            Instr::Equal(a, b, c)
                        }
                        Opcode::JumpNonZero => {
                            let a = param(1);
                            let b = param(2);
                            ptr += 3;
                            Instr::JumpNonZero(a, b)
                        }
                        Opcode::JumpZero => {
                            let a = param(1);
                            let b = param(2);
                            ptr += 3;
                            Instr::JumpZero(a, b)
                        }
                        Opcode::Input => {
                            let a = param(1);
                            ptr += 2;
                            Instr::Input(a)
                        }
                        Opcode::Output => {
                            let a = param(1);
                            ptr += 2;
                            Instr::Output(a)
                        }
                        Opcode::AdjustRelativeBase => {
                            let a = param(1);
                            ptr += 2;
                            Instr::AdjustRelativeBase(a)
                        }
                        Opcode::Halt => {
                            ptr += 1;
                            Instr::Halt
                        }
                        Opcode::Mutable => {
                            let params: Vec<_> = iter::from_fn(|| {
                                ptr += 1;
                                self.get_param(ptr).map(|_| self.slots[ptr].raw)
                            })
                            .collect();
                            Instr::Mutable(slot.raw, params)
                        }
                    };
                    stmts.push(Stmt {
                        label: slot.label.clone(),
                        instr,
                    });
                }

                Some(m @ Mark::Param(_)) => {
                    panic!("unexpected marked address {:?}", m);
                }

                Some(Mark::String) => {
                    let bucket = self.bucket_unlabelled(ptr, Mark::String);
                    ptr += bucket.len();
                    let bytes: Vec<_> = bucket.into_iter().map(|raw| raw as u8).collect();
                    let label = slot.label.clone();
                    let param = String::from_utf8(bytes).unwrap();
                    let instr = Instr::Data(vec![RawParam::String(param)]);
                    stmts.push(Stmt { label, instr })
                }

                Some(Mark::Data) => {
                    let bucket = self.bucket_unlabelled(ptr, Mark::Data);
                    ptr += bucket.len();
                    let raw_params: Vec<_> = bucket
                        .into_iter()
                        .map(|raw| RawParam::Number(raw))
                        .collect();
                    let label = slot.label.clone();
                    let instr = Instr::Data(raw_params);
                    stmts.push(Stmt { label, instr })
                }

                None => {
                    panic!("unmarked address `{}`", ptr);
                }
            }
        }

        Ast { stmts }
    }
}
