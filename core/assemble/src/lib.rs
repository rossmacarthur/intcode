pub mod ast;
mod error;
mod lex;
mod parse;
mod span;

use indexmap::IndexMap;

use crate::ast::{Instr, Label, Param, Program, RawParam, Stmt};

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
                Some(_) => unreachable!(),
                None => v.label = Some(output.len()),
            }
        }

        let mut param = |output: &mut Vec<_>, p, ip| -> i64 {
            let (mode, value) = match p {
                Param::Number(m, value) => (m.into(), value),
                Param::Label(m, Label::InstructionPointer, offset) => (m.into(), ip + offset),
                Param::Label(m, Label::Fixed(label), offset) => {
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
                let ip = (i + 4) as i64;
                output.push(instr.opcode());
                let x_mode = param(&mut output, x, ip);
                let y_mode = param(&mut output, y, ip);
                let z_mode = param(&mut output, z, ip);
                output[i] += x_mode * 100 + y_mode * 1_000 + z_mode * 10_000;
            }
            Instr::JumpNonZero(x, y) | Instr::JumpZero(x, y) => {
                let i = output.len();
                let ip = (i + 3) as i64;
                output.push(instr.opcode());
                let x_mode = param(&mut output, x, ip);
                let y_mode = param(&mut output, y, ip);
                output[i] += x_mode * 100 + y_mode * 1_000;
            }
            Instr::Input(p) | Instr::Output(p) | Instr::AdjustRelativeBase(p) => {
                let i = output.len();
                let ip = (i + 2) as i64;
                output.push(instr.opcode());
                let mode = param(&mut output, p, ip);
                output[i] += mode * 100;
            }
            Instr::Data(data) => {
                let ip = (output.len() + data.len()) as i64;
                for d in data {
                    match d {
                        RawParam::Label(Label::InstructionPointer, offset) => {
                            output.push(ip + offset);
                        }
                        RawParam::Label(Label::Fixed(label), offset) => {
                            labels.entry(label).or_default().refs.push(output.len());
                            output.push(offset);
                        }
                        RawParam::Number(value) => {
                            output.push(value);
                        }
                        RawParam::String(string) => {
                            output.extend(
                                string.into_owned().into_bytes().into_iter().map(i64::from),
                            );
                        }
                    }
                }
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

/// Assemble the program as a valid AST.
pub fn to_ast(input: &str) -> Result<Program, Vec<Error>> {
    parse::program(input)
}

/// Assemble the program as intcode.
pub fn to_intcode(input: &str) -> Result<String, Vec<Error>> {
    parse::program(input).map(|prog| {
        assemble(prog)
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(",")
    })
}