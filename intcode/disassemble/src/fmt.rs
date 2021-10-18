//! Format the AST as intcode assembly.

use std::fmt;
use std::fmt::Display;
use std::iter;

use crate::ast::{Ast, Instr, Label, Mode, Param, RawParam, Stmt};

impl Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Label::Underscore => "_",
            Label::InstructionPointer => "ip",
            Label::Fixed(label) => label,
        };
        f.write_str(s)
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Label(mode, label, value) => match mode {
                Mode::Positional => match value {
                    0 => write!(f, "{}", label),
                    v => write!(f, "{}{:+}", label, v),
                },
                Mode::Immediate => match value {
                    0 => write!(f, "#{}", label),
                    v => write!(f, "#{}{:+}", label, v),
                },
                Mode::Relative => match value {
                    0 => write!(f, "rb"),
                    v => write!(f, "rb{:+}", v),
                },
            },
            Self::Number(mode, value) => match mode {
                Mode::Positional => write!(f, "{}", value),
                Mode::Immediate => write!(f, "#{}", value),
                Mode::Relative => match value {
                    0 => write!(f, "rb"),
                    v => write!(f, "rb{:+}", v),
                },
            },
        }
    }
}

impl Display for RawParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Label(label, value) => match value {
                0 => write!(f, "{}", label),
                v => write!(f, "{}{:+}", label, v),
            },
            Self::Number(value) => write!(f, "{}", value),
            RawParam::String(s) => write!(f, "{:?}", s),
        }
    }
}

impl Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instr::Add(a, b, c) => write!(f, "ADD {}, {}, {}", a, b, c),
            Instr::Multiply(a, b, c) => write!(f, "MUL {}, {}, {}", a, b, c),
            Instr::LessThan(a, b, c) => write!(f, "LT {}, {}, {}", a, b, c),
            Instr::Equal(a, b, c) => write!(f, "EQ {}, {}, {}", a, b, c),
            Instr::JumpNonZero(a, b) => write!(f, "JNZ {}, {}", a, b),
            Instr::JumpZero(a, b) => write!(f, "JZ {}, {}", a, b),
            Instr::Input(a) => write!(f, "IN {}", a),
            Instr::Output(a) => write!(f, "OUT {}", a),
            Instr::AdjustRelativeBase(a) => write!(f, "ARB {}", a),
            Instr::Halt => write!(f, "HLT"),
            Instr::Data(params) => {
                write!(f, "DB ")?;
                for (i, p) in params.iter().enumerate() {
                    if i == params.len() - 1 {
                        write!(f, "{}", p)?
                    } else {
                        write!(f, "{}, ", p)?
                    }
                }
                Ok(())
            }
            Instr::Mutable(value, params) => {
                write!(f, "DB ")?;
                for (i, d) in iter::once(value as &dyn Display)
                    .chain(params.iter().map(|p| p as &dyn Display))
                    .enumerate()
                {
                    if i == params.len() {
                        write!(f, "{}", d)?;
                    } else {
                        write!(f, "{}, ", d)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.label {
            Some(label) => write!(f, "{}: {}", label, self.instr),
            None => write!(f, "{}", self.instr),
        }
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.stmts {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}
