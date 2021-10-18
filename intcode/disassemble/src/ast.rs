//! Abstract representation of assembly code.

use std::rc::Rc;

/// A label specified in a parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum Label {
    Underscore,
    InstructionPointer,
    Fixed(Rc<String>),
}

/// A parameter mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Positional,
    Immediate,
    Relative,
}

/// A parameter in an instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    /// A label, optionally with an offset.
    Label(Mode, Label, i64),
    /// An exact number.
    Number(Mode, i64),
}

/// A raw parameter in an instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum RawParam {
    /// A label, optionally with an offset.
    Label(Label, i64),
    /// An exact number.
    Number(i64),
    /// A string literal.
    String(String),
}

/// An instruction.
///
/// These generally map to an Intcode instruction., however there is also a
/// pseudo instruction `Data` for placing raw data into the program.
#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    /// Adds two parameters together.
    Add(Param, Param, Param),
    /// Multiplies two parameters together.
    Multiply(Param, Param, Param),
    /// Compares two parameters.
    LessThan(Param, Param, Param),
    /// Checks if two parameters are equal.
    Equal(Param, Param, Param),

    /// Moves the instruction pointer if the result is non-zero.
    JumpNonZero(Param, Param),
    /// Moves the instruction pointer if the result is zero.
    JumpZero(Param, Param),

    /// Fetches external data.
    Input(Param),
    /// Outputs external data.
    Output(Param),
    /// Adjusts the relative base by the given amount.
    AdjustRelativeBase(Param),

    /// Halts the program.
    Halt,

    /// (Pseudo) Places raw data in the program.
    Data(Vec<RawParam>),
    /// (Pseudo) Represents a mutable instruction.
    Mutable(i64, Vec<i64>),
}

/// A single line in a program.
///
/// This is simply just an instruction together with an optional label.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub label: Option<Label>,
    pub instr: Instr,
}

/// An entire program.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub stmts: Vec<Stmt>,
}
