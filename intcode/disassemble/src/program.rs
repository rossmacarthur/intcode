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
}

#[derive(Debug, Clone, Default)]
pub struct Slot {
    /// The original value before the program has run.
    pub original: Option<i64>,
    /// The current value during a program run.
    pub current: i64,
    /// An optional mark if we figure out what the memory looks like is.
    pub mark: Option<Mark>,
    /// An optional label if we add one.
    pub label: Option<Label>,
}

/// Represents an intcode program during our analysis.
#[derive(Debug, Clone)]
pub struct Program {
    pub slots: Vec<Slot>,
}

impl Opcode {
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

impl Slot {
    fn reset(&mut self) {
        self.current = self.original.unwrap_or(0);
    }
}

impl Program {
    pub fn new(intcode: Vec<i64>) -> Self {
        let slots = intcode
            .into_iter()
            .map(|n| Slot {
                original: Some(n),
                current: n,
                mark: None,
                label: None,
            })
            .collect();
        Self { slots }
    }

    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            slot.reset();
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn resize(&mut self, new_len: usize) {
        self.slots.resize_with(new_len, Slot::default)
    }

    pub fn get(&self, addr: usize) -> Option<i64> {
        self.slots.get(addr).map(|slot| slot.current)
    }

    pub fn get_mut(&mut self, addr: usize) -> Option<&mut i64> {
        self.slots.get_mut(addr).map(|slot| &mut slot.current)
    }

    pub fn get_original(&self, addr: usize) -> Option<i64> {
        self.slots.get(addr).and_then(|slot| slot.original)
    }

    pub fn mark_opcode(&mut self, addr: usize, opcode: Opcode) {
        let slot = &mut self.slots[addr];
        match slot.original {
            Some(orig) => {
                match &mut slot.mark {
                    // This address is already marked with the same opcode 👍.
                    Some(Mark::Opcode(o)) if *o == opcode => {}
                    // If this address is already marked with a different opcode
                    // then mark it as a "mutable" opcode.
                    Some(Mark::Opcode(o)) => *o = Opcode::Mutable,
                    // We are marking this address with an opcode that does not
                    // match the original value, so this is also a "mutable"
                    // opcode.
                    m @ None if orig % 100 != opcode.value() => *m = Some(Mark::Opcode(Opcode::Mutable)),
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
            },
            None => panic!(
                "tried to mark address `{}` with opcode `{:?}` but it doesn't exist in the original",
                addr, opcode
            ),
        }
    }

    pub fn mark_param(&mut self, addr: usize, mode: Mode) {
        let slot = &mut self.slots[addr];
        match slot.original {
            Some(value) => {
                let mark = Mark::Param(Param::Number(mode, value));
                match &mut slot.mark {
                    // This address is already marked with the same param 👍.
                    Some(ref m) if *m == mark => {}
                    // This address is unmarked, mark it with the given param.
                    m @ None => *m = Some(mark),
                    // Otherwise, this is address is already marked as something
                    // else, so we panic.
                    Some(m) => {
                        panic!(
                            "tried to mark address `{}` with `{:?}`, but it is already marked with `{:?}`",
                            addr, mark, m
                        );
                    }
                }
            },
            None => panic!(
                "tried to mark address `{}` with parameter `{:?}` but it doesn't exist in the original",
                addr, mode
            ),
        }
    }

    pub fn get_param(&self, addr: usize) -> Option<Param> {
        self.slots.get(addr).and_then(|slot| match &slot.mark {
            Some(Mark::Param(param)) => Some(param.clone()),
            _ => None,
        })
    }

    pub fn into_ast(self) -> Ast {
        let mut ptr = 0;
        let mut stmts = Vec::new();

        while let Some(
            slot @ Slot {
                original: Some(_), ..
            },
        ) = self.slots.get(ptr)
        {
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
                                self.get_param(ptr).and_then(|_| self.slots[ptr].original)
                            })
                            .collect();
                            Instr::Mutable(slot.original.unwrap(), params)
                        }
                    };
                    stmts.push(Stmt {
                        label: slot.label.clone(),
                        instr,
                    });
                }
                None => {
                    // For now assume unmarked data is just raw bytes 🤷‍♂️
                    let label = slot.label.clone();
                    let mut raw_params = vec![RawParam::Number(slot.original.unwrap())];
                    raw_params.extend(iter::from_fn(|| {
                        ptr += 1;
                        match &self.slots.get(ptr) {
                            Some(Slot {
                                original: Some(orig),
                                mark: None,
                                label: None,
                                ..
                            }) => Some(RawParam::Number(*orig)),
                            _ => None,
                        }
                    }));
                    let instr = Instr::Data(raw_params);
                    stmts.push(Stmt { label, instr })
                }
                Some(mark) => {
                    panic!("unexpected marked address {:?}", mark);
                }
            }
        }

        Ast { stmts }
    }
}
