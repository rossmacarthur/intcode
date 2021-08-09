pub mod ast;
mod error;
mod lex;
mod parse;
mod span;

use std::iter;

use either::Either;
use indexmap::IndexMap;

use crate::ast::{Instr, Param, Program, RawParam, Stmt};

pub use crate::error::Error;

#[derive(Debug, Default)]
struct State {
    label: Option<usize>,
    refs: Vec<usize>,
}

fn assemble(ast: Program) -> Vec<i64> {
    let mut output = Vec::new();
    let mut labels = IndexMap::<_, State>::new();

    for Stmt { label, instr } in ast.stmts {
        if let Some(label) = label {
            let v = labels.entry(label).or_default();
            match v.label {
                Some(_) => panic!("label `{}` used multiple times", label),
                None => v.label = Some(output.len()),
            }
        }

        let mut param = |output: &mut Vec<_>, p| -> i64 {
            let (mode, value) = match p {
                Param::Number(m, value) => (m.into(), value),
                Param::Label(m, label, offset) => {
                    labels.entry(label).or_default().refs.push(output.len());
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
            Instr::Data(data) => {
                let i = output.len();
                output.extend(data.into_iter().flat_map(|d| match d {
                    RawParam::Label(label, offset) => {
                        labels.entry(label).or_default().refs.push(i);
                        Either::Left(iter::once(offset))
                    }
                    RawParam::Number(value) => Either::Left(iter::once(value)),
                    RawParam::String(string) => {
                        Either::Right(string.into_owned().into_bytes().into_iter().map(i64::from))
                    }
                }));
            }
            Instr::Halt => output.push(instr.opcode()),
        }
    }

    for (_, State { label, refs }) in labels {
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

    output
}

/// Assemble the program as intcode.
pub fn program(input: &str) -> Result<String, Vec<Error>> {
    parse::program(input).map(|prog| {
        assemble(prog)
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(",")
    })
}
