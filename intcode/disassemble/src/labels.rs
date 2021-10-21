use std::collections::BTreeMap;
use std::iter;
use std::rc::Rc;

use crate::ast::{Label, Mode, Param};
use crate::program::{LabelType, Mark, Opcode, Program};

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

    fn set_label_type(&mut self, addr: usize, label_type: LabelType) {
        self.slots[addr].label_type = Some(label_type);
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

/// A sequence of unique labels to use when labeling instructions.
pub fn unique() -> impl Iterator<Item = Label> {
    let singles = ('a'..'z')
        .filter(|&c| c != 'l')
        .map(|c| Label::Fixed(Rc::new(String::from(c))));
    let doubles = ('a'..='z')
        .filter(|&c| c != 'l')
        .flat_map(|e| iter::repeat(e).zip('0'..='9'))
        .map(|(a, b)| Label::Fixed(Rc::new(format!("{}{}", a, b))));
    singles.chain(doubles)
}

fn get_labeled_addr(p: &Program, i: usize) -> Option<(LabelType, usize)> {
    let slot = &p.slots[i];

    // Exclude 0 because its often used as a placeholder value.
    // Exclude addresses greater than the length of the program. These are valid
    // but we can't label them.
    let addr = || {
        let addr = slot.raw as usize;
        (0 < addr && addr < p.len()).then(|| slot.raw as usize)
    };

    match slot.mark {
        // Find positional parameters, these refer to addresses in the
        // program.
        Some(Mark::Param(Param::Number(Mode::Positional, _))) => {
            addr().map(|a| (LabelType::Data, a))
        }

        // Find immediate parameters, if the instruction is a JNZ or JZ
        // the second parameter will likely refer to an address.
        Some(Mark::Param(Param::Number(Mode::Immediate, _)))
            if {
                let (op_i, op) = p.instr(i)?;
                matches!(op, Opcode::JumpNonZero | Opcode::JumpZero) && (i - op_i) == 2
            } =>
        {
            addr().map(|a| (LabelType::Instr, a))
        }

        _ => None,
    }
}

pub fn assign(p: &mut Program, mut labels: impl Iterator<Item = Label>) {
    let refs = (0..p.len())
        .filter_map(|i| get_labeled_addr(p, i).map(|addr| (i, addr)))
        .fold(BTreeMap::new(), |mut map, (i, (ty, addr))| {
            map.entry(addr).or_insert_with(Vec::new).push((ty, i));
            map
        });

    for (addr, indexes) in refs {
        let labelfn = || labels.next().unwrap();

        let set_label_type = |p: &mut Program| {
            // If all the referrer types are the same then we can add the
            // label type to the address.
            if indexes.windows(2).all(|w| w[0].0 == w[1].0) {
                let label_type = indexes[0].0;
                match &p.slots[addr].label_type {
                    &Some(lt) if lt == label_type => {}
                    Some(lt) => log::warn!("address {} already has label type {:?}", addr, lt),
                    None => p.set_label_type(addr, label_type),
                }
            } else {
                log::warn!("address {} has different referrer types", addr);
            }
        };

        match &p.slots[addr].mark {
            Some(Mark::Opcode(_)) => {
                // Assign a new label to the referring parameters.
                let label = p.get_or_set_label(addr, labelfn);
                set_label_type(p);
                for (_, i) in indexes {
                    p.label_param(i, label.clone(), 0);
                }
            }
            Some(Mark::Param(_)) => {
                let this_op = p.instr_addr(addr).unwrap();
                let prev_op = p.instr_addr(this_op);

                // If all the referrers are in the previous instruction then
                // we can use the `ip` label.
                let label = if indexes.iter().all(|(_, i)| p.instr_addr(*i) == prev_op) {
                    Label::InstructionPointer
                } else {
                    p.get_or_set_label(this_op, labelfn)
                };

                set_label_type(p);

                // Assign label to the referring parameters with an offset.
                let offset = addr - this_op;
                for (_, i) in indexes {
                    p.label_param(i, label.clone(), offset as i64);
                }
            }
            Some(Mark::Data) | None => {
                // The label is referring to some data or unknown thing.
                // For now just assume unknown is data, it could potentially
                // be part of an unmarked instruction though.
                let label = p.get_or_set_label(addr, labelfn);
                set_label_type(p);
                for (_, i) in indexes {
                    p.label_param(i, label.clone(), 0);
                }
            }
        }
    }
}
