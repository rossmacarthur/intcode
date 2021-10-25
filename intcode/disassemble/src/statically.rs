use crate::ast::Mode;
use crate::program::{Mark, Opcode, Program, Slot};

fn try_mark_instr(p: &mut Program, addr: usize) -> Option<usize> {
    let slot = &p.slots[addr];

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
        let addr = addr + i + 1;
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
            addr
        );
    }

    // Everything looks good, mark the instruction and parameters!
    p.mark_opcode(addr, opcode);
    for (addr, mode) in modes {
        p.mark_param(addr, mode);
    }

    Some(ps + 1)
}

fn try_mark_string(p: &mut Program, addr: usize) -> Option<usize> {
    let is_char = |p: &Program, a| {
        let slot: &Slot = p.slots.get(a)?;
        let is = slot.is_unmarked()
            && slot.is_unlabelled()
            && matches!(slot.raw, 9 | 10 | 13 | 32..=126);
        Some(is)
    };

    let mut a = addr;
    let mut addresses = Vec::new();
    while let Some(true) = is_char(p, a) {
        addresses.push(a);
        a += 1;
    }

    if addresses.len() < 2 {
        return None;
    }
    for a in addresses {
        p.mark(a, Mark::String);
    }
    Some(a - addr)
}

/// Statically mark code in the program if it looks like an instruction and
/// parameters. This has a lot of false positives, so it best to mark using the
/// dynamic marker first.
pub fn mark(p: &mut Program) {
    // First mark string data, otherwise default to ordinary data
    let indexes: Vec<_> = p
        .slots
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| {
            let is = slot.is_unmarked() && slot.has_rw_purpose();
            is.then(|| i)
        })
        .collect();
    for i in indexes {
        if try_mark_string(p, i).is_none() && p.slots[i].is_unmarked() {
            p.mark(i, Mark::Data)
        }
    }

    // Then mark instructions at addresses mentioned at immediate jumps
    let indexes: Vec<_> = p
        .slots
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| {
            let is = slot.is_unmarked() && slot.has_jump_purpose();
            is.then(|| i)
        })
        .collect();
    for i in indexes {
        try_mark_instr(p, i);
    }
}
