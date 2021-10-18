use std::collections::BTreeMap;
use std::iter;
use std::rc::Rc;

use crate::ast::{Label, Mode, Param};
use crate::program::{Mark, Opcode, Program};

impl Mark {
    fn label_param(&mut self, label: Label, offset: i64) {
        let mode = match self {
            Self::Param(Param::Number(mode, _)) => *mode,
            _ => panic!("expected number parameter"),
        };
        *self = Self::Param(Param::Label(mode, label, offset));
    }
}

impl Program {
    fn label_param(&mut self, addr: usize, label: Label, offset: i64) {
        self.slots[addr]
            .mark
            .as_mut()
            .unwrap()
            .label_param(label, offset);
    }

    fn get_or_set_label(&mut self, addr: usize, with: impl FnOnce() -> Label) -> Label {
        self.slots[addr].label.get_or_insert_with(with).clone()
    }

    /// Finds the preceding opcode.
    fn instr(&self, mut addr: usize) -> Option<(usize, Opcode)> {
        while addr > 0 {
            addr -= 1;
            if let Some(Mark::Opcode(opcode)) = self.slots[addr].mark {
                return Some((addr, opcode));
            }
        }
        None
    }

    /// Finds the preceding opcode's address.
    fn instr_addr(&self, addr: usize) -> Option<usize> {
        self.instr(addr).map(|(a, _)| a)
    }
}

fn iter_labels() -> impl Iterator<Item = Label> {
    let singles = ('a'..'z')
        .filter(|&c| c != 'l')
        .map(|c| Label::Fixed(Rc::new(String::from(c))));
    let doubles = ('a'..='z')
        .filter(|&c| c != 'l')
        .flat_map(|e| iter::repeat(e).zip('0'..='9'))
        .map(|(a, b)| Label::Fixed(Rc::new(format!("{}{}", a, b))));
    singles.chain(doubles)
}

fn get_labeled_addr(p: &Program, i: usize) -> Option<usize> {
    let slot = p.slots.get(i)?;

    // We only care about addresses that exist in the original program.
    // Exclude 0 because its often used as a placeholder value.
    let addr = || match slot.original {
        Some(addr) if addr > 0 && p.get_original(addr as usize).is_some() => Some(addr as usize),
        _ => None,
    };

    match slot.mark {
        // Find positional parameters, these refer to addresses in the
        // program.
        Some(Mark::Param(Param::Number(Mode::Positional, _))) => addr(),

        // Find immediate parameters, if the instruction is a JNZ or JZ
        // the second parameter will likely refer to an address.
        Some(Mark::Param(Param::Number(Mode::Immediate, _)))
            if {
                let (op_i, op) = p.instr(i)?;
                matches!(op, Opcode::JumpNonZero | Opcode::JumpZero) && (i - op_i) == 2
            } =>
        {
            addr()
        }

        _ => None,
    }
}

pub fn assign(p: &mut Program) {
    let mut labels = iter_labels();

    let refs = (0..p.len())
        .filter_map(|i| get_labeled_addr(p, i).map(|addr| (i, addr)))
        .fold(BTreeMap::new(), |mut map, (i, addr)| {
            map.entry(addr).or_insert_with(Vec::new).push(i);
            map
        });

    for (addr, indexes) in refs {
        let labelfn = || labels.next().unwrap();
        match &p.slots[addr].mark {
            Some(Mark::Opcode(_)) => {
                // Assign a new label to the referring parameters.
                let label = p.get_or_set_label(addr, labelfn);
                for i in indexes {
                    p.label_param(i, label.clone(), 0);
                }
            }
            Some(Mark::Param(_)) => {
                let this_op = p.instr_addr(addr).unwrap();
                let prev_op = p.instr_addr(this_op);

                // If all the referrers are in the previous instruction then
                // we can use the `ip` label.
                let label = if indexes.iter().all(|i| p.instr_addr(*i) == prev_op) {
                    Label::InstructionPointer
                } else {
                    p.get_or_set_label(this_op, labelfn)
                };

                // Assign label to the referring parameters with an offset.
                let offset = addr - this_op;
                for i in indexes {
                    p.label_param(i, label.clone(), offset as i64);
                }
            }
            None => {
                // The label is referring to some unknown thing 🤔
                // For now just assume that it is data, it could potentially
                // be part of an unmarked instruction though.
                let label = p.get_or_set_label(addr, labelfn);
                for i in indexes {
                    p.label_param(i, label.clone(), 0);
                }
            }
        }
    }
}
