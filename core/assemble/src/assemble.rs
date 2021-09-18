pub mod ast;
mod lex;
mod parse;

use indexmap::IndexMap;

use self::ast::{Instr, Label, Param, Program, RawParam, Stmt};
use self::parse::Parser;
use crate::error::{Error, ErrorSet, ResultSet, Warning};
use crate::span::{Span, S};

#[derive(Debug, Clone)]
pub struct Intcode {
    pub output: Vec<i64>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Default)]
struct State {
    defs: Vec<(usize, Span)>,
    refs: Vec<(usize, Span)>,
}

fn insert_label<'a>(
    labels: &mut IndexMap<&'a str, State>,
    label: Option<S<Label<'a>>>,
    address: usize,
) -> Result<(), Error> {
    match label {
        Some(S(Label::Underscore, span)) => {
            return Err(Error::new(
                "label is reserved to indicate a runtime value",
                span,
            ));
        }
        Some(S(Label::InstructionPointer, span)) => {
            return Err(Error::new(
                "label is reserved to refer to the instruction pointer",
                span,
            ));
        }
        Some(S(Label::Fixed("rb"), span)) => {
            return Err(Error::new(
                "label is reserved to refer to the relative base",
                span,
            ));
        }
        Some(S(Label::Fixed(label), span)) => {
            labels.entry(label).or_default().defs.push((address, span));
        }
        None => {}
    }
    Ok(())
}

fn assemble(ast: Program<'_>) -> ResultSet<Intcode> {
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut labels = IndexMap::<&str, State>::new();

    for Stmt { label, instr } in ast.stmts {
        insert_label(&mut labels, label, output.len())
            .map_err(|err| errors.push(err))
            .ok();

        let mut param = |output: &mut Vec<_>, S(p, _), ip| -> i64 {
            let (mode, value) = match p {
                Param::Number(m, value) => (m.into(), value),
                Param::Label(m, S(Label::Underscore, _), offset) => (m.into(), offset),
                Param::Label(m, S(Label::InstructionPointer, _), offset) => (m.into(), ip + offset),
                Param::Label(m, S(Label::Fixed(label), span), offset) => {
                    labels
                        .entry(label)
                        .or_default()
                        .refs
                        .push((output.len(), span));
                    (m.into(), offset)
                }
            };
            output.push(value);
            mode
        };

        match instr.0 {
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
                    match d.0 {
                        RawParam::Label(S(Label::Underscore, _), offset) => {
                            output.push(offset);
                        }
                        RawParam::Label(S(Label::InstructionPointer, _), offset) => {
                            output.push(ip + offset);
                        }
                        RawParam::Label(S(Label::Fixed(label), span), offset) => {
                            labels
                                .entry(label)
                                .or_default()
                                .refs
                                .push((output.len(), span));
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

    for (label, State { defs, refs }) in labels {
        match defs.as_slice() {
            &[] => {
                for (_, span) in refs {
                    errors.push(Error::new("undefined label", span));
                }
            }
            &[(address, span)] => {
                if refs.is_empty() && !label.starts_with('_') {
                    warnings.push(Warning::new("label is never used", span))
                } else {
                    for (r, _) in refs {
                        output[r] += address as i64;
                    }
                }
            }
            _ => {
                for (i, (_, span)) in defs.into_iter().enumerate() {
                    let msg = if i == 0 {
                        "first definition of label"
                    } else {
                        "label redefined here"
                    };
                    errors.push(Error::new(msg, span))
                }
            }
        }
    }
    match errors.is_empty() {
        true => Ok(Intcode { output, warnings }),
        false => Err(ErrorSet { errors, warnings }),
    }
}

/// Assemble the program as intcode.
pub fn to_intcode(asm: &str) -> ResultSet<Intcode> {
    Parser::new(asm).eat_program().and_then(assemble)
}
