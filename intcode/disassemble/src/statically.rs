use crate::ast::Mode;
use crate::program::{LabelType, Opcode, Program};

fn try_mark_instr(p: &mut Program, ptr: usize) -> Option<usize> {
    let slot = &p.slots[ptr];

    if slot.mark.is_some() {
        return None;
    }

    let instr = slot.raw;
    let opcode = Opcode::from_value(instr % 100)?;
    let ps = opcode.params();

    // Check if the instruction has digits greater than allowed for this
    // parameter count.
    let divs = [100, 1_000, 10_000, 100_000];
    if instr / divs[ps] > 0 {
        return None;
    }
    // Check that there are available parameter slots and they are unmarked.
    let mut modes = Vec::new();
    for (i, div) in divs.iter().enumerate().take(ps) {
        let addr = ptr + i + 1;
        if p.slots.get(addr)?.mark.is_some() {
            return None;
        }
        let mode = Mode::from_value(instr / div % 10)?;
        modes.push((addr, mode));
    }

    if !modes.is_empty() && modes.iter().all(|(_, m)| matches!(m, Mode::Positional)) {
        log::warn!(
            "all the modes are positional for {:?} instruction at address {}",
            opcode,
            ptr
        );
    }

    // Everything looks good, mark the instruction and parameters!
    p.mark_opcode(ptr, opcode);
    for (addr, mode) in modes {
        p.mark_param(addr, mode);
    }

    Some(ps + 1)
}

/// Statically mark code in the program if it looks like an instruction and
/// parameters. This has a lot of false positives, so it best to mark using the
/// dynamic marker first.
pub fn mark(p: &mut Program) {
    // First mark instructions at addresses marked with LabelType::Instr
    let a = p.slots.iter().enumerate().filter_map(|(i, slot)| {
        (slot.mark.is_none() && matches!(slot.label_type, Some(LabelType::Instr))).then(|| i)
    });
    // Then mark instructions at addresses *not* marked with LabelType::Data
    let b = p.slots.iter().enumerate().filter_map(|(i, slot)| {
        (slot.mark.is_none() && !matches!(slot.label_type, Some(LabelType::Data))).then(|| i)
    });
    let indexes: Vec<_> = a.chain(b).collect();
    for i in indexes {
        try_mark_instr(p, i);
    }

    // Finally we mark any remaining addresses as data
    let i = p.slots.iter().enumerate().filter_map(|(i, slot)| {
        (slot.mark.is_none() && matches!(slot.label_type, Some(LabelType::Data))).then(|| i)
    });
    let indexes: Vec<_> = i.collect();
    for i in indexes {
        p.mark_data(i);
    }
}
