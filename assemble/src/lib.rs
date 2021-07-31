pub mod ast;
mod error;
mod lex;
mod parse;

use indexmap::IndexMap;

use crate::ast::{Instr, Param, Program, Stmt};
use crate::error::Result;

pub use crate::error::Error;

#[derive(Debug, Default)]
struct State {
    label: Option<usize>,
    refs: Vec<usize>,
}

fn assemble(ast: Program) -> Result<Vec<i64>> {
    let mut output = Vec::new();
    let mut idents = IndexMap::<_, State>::new();

    for Stmt { label, instr } in ast.stmts {
        if let Some(label) = label {
            let v = idents.entry(label).or_default();
            match v.label {
                Some(_) => panic!("label `{}` used multiple times", label),
                None => v.label = Some(output.len()),
            }
        }

        let mut param = |output: &mut Vec<_>, p| -> i64 {
            let (mode, value) = match p {
                Param::Number(m, value) => (m.into(), value),
                Param::Ident(m, ident, offset) => {
                    idents.entry(ident).or_default().refs.push(output.len());
                    (m.into(), offset)
                }
            };
            output.push(value);
            mode
        };

        match instr {
            Instr::Add(x, y, z)
            | Instr::Multiply(x, y, z)
            | Instr::LessThan(x, y, z)
            | Instr::Equal(x, y, z) => {
                let i = output.len();
                output.push(instr.opcode());
                let x_mode = param(&mut output, x);
                let y_mode = param(&mut output, y);
                let z_mode = param(&mut output, z);
                output[i] += x_mode * 100 + y_mode * 1_000 + z_mode * 10_000;
            }
            Instr::JumpNonZero(x, y) | Instr::JumpZero(x, y) => {
                let i = output.len();
                output.push(instr.opcode());
                let x_mode = param(&mut output, x);
                let y_mode = param(&mut output, y);
                output[i] += x_mode * 100 + y_mode * 1_000;
            }
            Instr::Input(p) | Instr::Output(p) | Instr::AdjustRelativeBase(p) => {
                let i = output.len();
                output.push(instr.opcode());
                let mode = param(&mut output, p);
                output[i] += mode * 100;
            }
            Instr::DataByte(p) => {
                param(&mut output, p);
            }
            Instr::Halt => output.push(instr.opcode()),
        }
    }

    for (_, State { label, refs }) in idents {
        let ptr = match label {
            Some(ptr) => ptr,
            None => {
                output.push(0);
                output.len() - 1
            }
        };
        for r in refs {
            output[r] += ptr as i64;
        }
    }

    Ok(output)
}

/// Assemble the program as intcode.
pub fn program(input: &str) -> Result<String> {
    Ok(assemble(parse::program(input)?)?
        .into_iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn advent_day2_example1() {
        let asm = "\
ADD a, b, 3
MUL 3, c, 0
HLT
a: DB 30
b: DB 40
c: DB 50
";
        let expected = "1,9,10,3,2,3,11,0,99,30,40,50";
        assert_eq!(program(asm).unwrap(), expected);
    }

    #[test]
    fn advent_day5_example_immediate() {
        let asm = "\
MUL a, #3, 4
a: DB 33
";
        let expected = "1002,4,3,4,33";
        assert_eq!(program(asm).unwrap(), expected);
    }

    #[test]
    fn advent_day5_example_cmp_positional() {
        let asm = "\
IN a
EQ a, b, a
OUT a
HLT
a: DB -1
b: DB 8
";
        let expected = "3,9,8,9,10,9,4,9,99,-1,8";
        assert_eq!(program(asm).unwrap(), expected);
    }

    #[test]
    fn advent_day5_example_cmp_immediate() {
        let asm = "\
IN 3
EQ #-1, #8, 3
OUT 3
HLT
";
        let expected = "3,3,1108,-1,8,3,4,3,99";
        assert_eq!(program(asm).unwrap(), expected);
    }

    #[test]
    fn advent_day5_example_jump_positional() {
        let asm = "\
   IN a
   JZ a, d
   ADD b, c, b
o: OUT b
   HLT
a: DB -1
b: DB 0
c: DB 1
d: DB o
";
        let expected = "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9";
        assert_eq!(program(asm).unwrap(), expected);
    }

    #[test]
    fn advent_day5_example_jump_immediate() {
        let asm = "\
   IN  j+1
j: JNZ #-1, #o
   ADD #0, #0, a
o: OUT a
   HLT
a: DB  1
";
        let expected = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";
        assert_eq!(program(asm).unwrap(), expected);
    }

    #[test]
    fn advent_day9_example_quine() {
        let asm = "\
ARB #1
OUT ~-1
ADD 100, #1, 100
EQ  100, #16, 101
JZ  101, #0
HLT
";
        let expected = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
        assert_eq!(program(asm).unwrap(), expected);
    }

    #[test]
    fn advent_day9_example_16_digit_number() {
        let asm = "\
MUL #34915192, #34915192, x
OUT x
HLT
";
        let expected = "1102,34915192,34915192,7,4,7,99,0";
        assert_eq!(program(asm).unwrap(), expected);
    }
}
