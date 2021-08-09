//! Abstract representation of assembly code.

use dairy::String;

/// A parameter mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Positional,
    Immediate,
    Relative,
}

/// A parameter in an instruction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Param<'i> {
    /// A label, optionally with an offset.
    Label(Mode, &'i str, i64),
    /// An exact number.
    Number(Mode, i64),
}

/// A raw parameter in an instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum RawParam<'i> {
    /// A label, optionally with an offset.
    Label(&'i str, i64),
    /// An exact number.
    Number(i64),
    /// A string literal.
    String(String<'i>),
}

/// An instruction.
///
/// These generally map to an Intcode instruction., however there is also a
/// pseudo instruction `Data` for placing raw data into the program.
#[derive(Debug, Clone, PartialEq)]
pub enum Instr<'i> {
    /// Adds two parameters together.
    Add(Param<'i>, Param<'i>, Param<'i>),
    /// Multiplies two parameters together.
    Multiply(Param<'i>, Param<'i>, Param<'i>),
    /// Compares two parameters.
    LessThan(Param<'i>, Param<'i>, Param<'i>),
    /// Checks if two parameters are equal.
    Equal(Param<'i>, Param<'i>, Param<'i>),

    /// Moves the instruction pointer if the result is non-zero.
    JumpNonZero(Param<'i>, Param<'i>),
    /// Moves the instruction pointer if the result is zero.
    JumpZero(Param<'i>, Param<'i>),

    /// Fetches external data.
    Input(Param<'i>),
    /// Outputs external data.
    Output(Param<'i>),
    /// Adjusts the relative base by the given amount.
    AdjustRelativeBase(Param<'i>),

    /// Halts the program.
    Halt,

    /// (Pseudo) Places raw data in the program.
    Data(Vec<RawParam<'i>>),
}

/// A single line in a program.
///
/// This is simply just an instruction together with an optional label.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt<'i> {
    pub label: Option<&'i str>,
    pub instr: Instr<'i>,
}

/// An entire program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program<'i> {
    pub stmts: Vec<Stmt<'i>>,
}

impl From<Mode> for i64 {
    fn from(mode: Mode) -> i64 {
        match mode {
            Mode::Positional => 0,
            Mode::Immediate => 1,
            Mode::Relative => 2,
        }
    }
}

impl Instr<'_> {
    pub(crate) fn opcode(&self) -> i64 {
        match self {
            Self::Add(..) => 1,
            Self::Multiply(..) => 2,
            Self::Input(..) => 3,
            Self::Output(..) => 4,
            Self::JumpNonZero(..) => 5,
            Self::JumpZero(..) => 6,
            Self::LessThan(..) => 7,
            Self::Equal(..) => 8,
            Self::AdjustRelativeBase(..) => 9,
            Self::Halt => 99,
            i => panic!("no opcode for `{:?}`", i),
        }
    }
}
