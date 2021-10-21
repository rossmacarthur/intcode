use crate::ast::Mode;
use crate::program::{Opcode, Program, Slot};

fn try_mark_instr_at(p: &mut Program, ptr: usize) -> Option<usize> {
    let instr = p.slots[ptr].raw;
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
    for i in 0..ps {
        let addr = ptr + i + 1;
        if p.slots.get(addr)?.mark.is_some() {
            return None;
        }
        let mode = Mode::from_value(instr / divs[i] % 10)?;
        modes.push((addr, mode));
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
    let mut ptr = 0;
    while ptr < p.len() {
        if let Slot { mark: None, .. } = p.slots[ptr] {
            ptr += try_mark_instr_at(p, ptr).unwrap_or(1);
        } else {
            ptr += 1;
        }
    }
}
