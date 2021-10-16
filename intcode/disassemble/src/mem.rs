use std::collections::BTreeMap;
use std::rc::Rc;

use crate::ast::{Instr, Label, Mode, Param, Program, RawParam, Stmt};

/// An instruction type.
#[derive(Debug, Clone, PartialEq)]
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
}

#[derive(Debug, Clone, PartialEq)]
enum Mark {
    Opcode(Opcode),
    Param(Param),
}

#[derive(Debug, Clone, Default)]
struct Slot {
    /// The original value before the program has run.
    original: Option<i64>,
    /// The current value during a program run.
    current: i64,
    /// An optional mark if we figure out what the memory looks like is.
    mark: Option<Mark>,
    /// An optional label if we add one.
    label: Option<Label>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    slots: Vec<Slot>,
}

impl Slot {
    fn reset(&mut self) {
        self.current = self.original.unwrap_or(0);
    }
}

impl Memory {
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

    fn get_original(&self, addr: usize) -> Option<i64> {
        self.slots.get(addr).and_then(|slot| slot.original)
    }

    fn mark(&mut self, addr: usize, mark: Mark) {
        match &mut self.slots[addr].mark {
            Some(ref m) if *m == mark => {}
            m @ None => *m = Some(mark),
            Some(m) => {
                panic!(
                    "tried to mark address `{}` with `{:?}`, but it is already marked with `{:?}`",
                    addr, mark, m
                );
            }
        }
    }

    pub fn mark_opcode(&mut self, addr: usize, opcode: Opcode) {
        match self.slots[addr].original {
            Some(_) => self.mark(addr, Mark::Opcode(opcode)),
            None => panic!(
                "tried to mark address `{}` with opcode `{:?}` but it doesn't exist in the original",
                addr, opcode
            ),
        }
    }

    pub fn mark_param(&mut self, addr: usize, mode: Mode) {
        match self.slots[addr].original {
            Some(value) => self.mark(addr, Mark::Param(Param::Number(mode, value))),
            None => panic!(
                "tried to mark address `{}` with parameter `{:?}` but it doesn't exist in the original",
                addr, mode
            ),
        }
    }

    fn upgrade_param(&mut self, addr: usize, label: Label, offset: i64) {
        let mark = self.slots[addr].mark.as_mut().unwrap();
        let mode = match mark {
            Mark::Param(Param::Number(mode, _)) => *mode,
            _ => todo!(),
        };
        *mark = Mark::Param(Param::Label(mode, label, offset));
    }

    fn get_param(&self, addr: usize) -> Param {
        match &self.slots[addr].mark {
            Some(Mark::Param(param)) => param.clone(),
            m => panic!("no param marked at `{}`, found `{:?}`", addr, m),
        }
    }

    fn get_or_set_label(&mut self, addr: usize, with: impl FnOnce() -> Label) -> Label {
        self.slots[addr].label.get_or_insert_with(with).clone()
    }

    /// Finds the address of the preceding opcode.
    fn opcode_ctx(&self, mut addr: usize) -> Option<usize> {
        while addr > 0 {
            addr -= 1;
            if matches!(self.slots[addr].mark, Some(Mark::Opcode(_))) {
                return Some(addr);
            }
        }
        None
    }

    pub fn assign_positional_labels(&mut self) {
        let mut iter = ('a'..='z').map(|c| Label::Fixed(Rc::new(String::from(c))));

        let refs = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                // Find positional parameters, these refer to addresses in
                // the program.
                let is_pos_param = matches!(
                    slot.mark,
                    Some(Mark::Param(Param::Number(Mode::Positional, _)))
                );
                // We only care about addresses that exist in the original
                // program. Exclude 0 because its often used as a placeholder
                // value.
                let is_address = matches!(
                    slot.original,
                    Some(addr) if addr > 0 && self.get_original(addr as usize).is_some()
                );
                (is_pos_param && is_address).then(|| (i, slot.original.unwrap() as usize))
            })
            .fold(BTreeMap::new(), |mut map, (i, addr)| {
                map.entry(addr).or_insert_with(Vec::new).push(i);
                map
            });

        for (addr, indexes) in refs {
            let labelfn = || iter.next().unwrap();
            match &self.slots[addr].mark {
                Some(Mark::Opcode(_)) => {
                    // Assign a new label to the referring parameters.
                    let label = self.get_or_set_label(addr, labelfn);
                    for i in indexes {
                        self.upgrade_param(i, label.clone(), 0);
                    }
                }
                Some(Mark::Param(_)) => {
                    let this_opcode = self.opcode_ctx(addr).unwrap();
                    let prev_opcode = self.opcode_ctx(this_opcode);

                    // If all the referrers are in the previous instruction then
                    // we can use the `ip` label.
                    let label = if indexes.iter().all(|i| self.opcode_ctx(*i) == prev_opcode) {
                        Label::InstructionPointer
                    } else {
                        self.get_or_set_label(this_opcode, labelfn)
                    };

                    // Assign label to the referring parameters with an offset.
                    let offset = addr - this_opcode;
                    for i in indexes {
                        self.upgrade_param(i, label.clone(), offset as i64);
                    }
                }
                None => {
                    // The label is referring to some unknown thing 🤔
                    // For now just assume that it is data, it could potentially
                    // be part of an unmarked instruction though.
                    let label = self.get_or_set_label(addr, labelfn);
                    for i in indexes {
                        self.upgrade_param(i, label.clone(), 0);
                    }
                }
            }
        }
    }

    pub fn into_ast(mut self) -> Program {
        self.assign_positional_labels();

        let mut ptr = 0;
        let mut stmts = Vec::new();

        while let Some(slot) = self.slots.get(ptr) {
            match &slot.mark {
                Some(Mark::Opcode(opcode)) => {
                    let param = |i: usize| self.get_param(ptr + i);
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
                    };
                    stmts.push(Stmt {
                        label: slot.label.clone(),
                        instr,
                    });
                }
                None => {
                    // For now assume unmarked data is just raw bytes 🤷‍♂️
                    ptr += 1;
                    if let Some(orig) = slot.original {
                        let instr = Instr::Data(vec![RawParam::Number(orig)]);
                        stmts.push(Stmt {
                            label: slot.label.clone(),
                            instr,
                        })
                    }
                }
                Some(mark) => {
                    panic!("unexpected marked address {:?}", mark);
                }
            }
        }

        Program { stmts }
    }
}
