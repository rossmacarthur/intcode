use std::collections::BTreeMap;
use std::iter;
use std::rc::Rc;

use crate::ast::{Label, Param};
use crate::program::{Mark, Mention, Opcode, Program};

impl Mark {
    fn label_param(&mut self, label: Label, offset: i64) {
        let mode = match self {
            // This parameter is already marked with the same label and offset 👍
            Self::Param(Param::Label(_, l, o)) if *l == label && *o == offset => {
                return;
            }
            // This parameter is unmarked
            Self::Param(Param::Number(mode, _)) => *mode,
            p => panic!("unexpected parameter `{:?}`", p),
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

    /// Finds the start of this string.
    fn string_addr(&self, mut addr: usize) -> usize {
        while let Some(Mark::String) = self.slots.get(addr).and_then(|s| s.mark.as_ref()) {
            addr -= 1
        }
        addr
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

pub fn assign(p: &mut Program, mut labels: impl Iterator<Item = Label>) {
    let refs: BTreeMap<_, _> = (0..p.len())
        .filter_map(|i| {
            let mentions: Vec<_> = p.slots[i]
                .mentions
                .iter()
                .filter(|m| p.slots[m.referrer].raw == (i as i64))
                .copied()
                .collect();
            (!mentions.is_empty()).then(|| (i, mentions))
        })
        .collect();

    for (addr, mentions) in refs {
        let labelfn = || labels.next().unwrap();
        match &p.slots[addr].mark {
            Some(Mark::Opcode(_) | Mark::Data) | None => {
                // Assign a new label to the referring parameters.
                let label = p.get_or_set_label(addr, labelfn);
                for Mention { referrer, .. } in mentions {
                    p.label_param(referrer, label.clone(), 0);
                }
            }

            Some(Mark::Param(_)) => {
                let this_op = p.instr_addr(addr).unwrap();
                let prev_op = p.instr_addr(this_op);

                // If all the referrers are in the previous instruction then
                // we can use the `ip` label.
                let label = if mentions.iter().all(|m| p.instr_addr(m.referrer) == prev_op) {
                    Label::InstructionPointer
                } else {
                    p.get_or_set_label(this_op, labelfn)
                };

                // Assign label to the referring parameters with an offset.
                let offset = addr - this_op;
                for Mention { referrer, .. } in mentions {
                    p.label_param(referrer, label.clone(), offset as i64);
                }
            }

            Some(Mark::String) => {
                // Find the start of the string and label from there.
                let start = p.string_addr(addr);
                let label = p.get_or_set_label(addr, labelfn);
                // Assign label to the referring parameters with an offset.
                let offset = addr - start;
                for Mention { referrer, .. } in mentions {
                    p.label_param(referrer, label.clone(), offset as i64);
                }
            }
        }
    }
}
