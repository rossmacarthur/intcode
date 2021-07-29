use indexmap::IndexMap;

use crate::ast::{Instr, Param, Program, Stmt};
use crate::error::Result;
use crate::parse;

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

        let mut param = |output: &mut Vec<_>, p| {
            output.push(match p {
                Param::Exact(v) => v,
                Param::Ident(i) => {
                    idents.entry(i).or_default().refs.push(output.len());
                    0
                }
            });
        };

        match instr {
            Instr::Add(x, y, z)
            | Instr::Multiply(x, y, z)
            | Instr::LessThan(x, y, z)
            | Instr::Equal(x, y, z) => {
                output.push(instr.opcode());
                param(&mut output, x);
                param(&mut output, y);
                param(&mut output, z);
            }
            Instr::JumpNonZero(x, y) | Instr::JumpZero(x, y) => {
                output.push(instr.opcode());
                param(&mut output, x);
                param(&mut output, y);
            }
            Instr::Input(p) | Instr::Output(p) | Instr::AdjustRelativeBase(p) => {
                output.push(instr.opcode());
                param(&mut output, p);
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
            output[r] = ptr as i64;
        }
    }

    Ok(output)
}

/// Compile intcode assembly.
pub fn program<'i>(input: &'i str) -> Result<String> {
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
}
