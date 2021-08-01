//! Abstract representation of assembly code.

/// The parameter mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Positional,
    Immediate,
    Relative,
}

/// A parameter in an instruction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Param<'i> {
    /// A parameter that refers to a variable or label, optionally with an
    /// offset.
    ///
    /// For example the 'x+3' in the following code:
    /// ```asm
    /// ADD 0, 1, x+3
    /// ```
    Ident(Mode, &'i str, i64),
    /// A parameter that is an exact number.
    ///
    /// For example the '7' in the following code:
    /// ```asm
    /// ADD x, y, 7
    /// ```
    Number(Mode, i64),
}

/// Raw data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Data<'i> {
    /// Data that refers to a variable or label, optionally with an offset.
    Ident(&'i str, i64),
    /// Data that is an exact number.
    Number(i64),
    /// Data that is a string literal.
    String(&'i str),
}

/// An instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Instr<'i> {
    /// Adds two parameters together.
    Add(Param<'i>, Param<'i>, Param<'i>),
    /// Multiplies two parameters together.
    Multiply(Param<'i>, Param<'i>, Param<'i>),
    /// Move the instruction pointer if the result is non-zero.
    JumpNonZero(Param<'i>, Param<'i>),
    /// Move the instruction pointer if the result is zero.
    JumpZero(Param<'i>, Param<'i>),
    /// Compare two parameters.
    LessThan(Param<'i>, Param<'i>, Param<'i>),
    /// Check if two parameters are equal.
    Equal(Param<'i>, Param<'i>, Param<'i>),

    /// Fetch external data.
    Input(Param<'i>),
    /// Output external data.
    Output(Param<'i>),
    /// Adjust the relative base by the given amount.
    AdjustRelativeBase(Param<'i>),

    /// Halts the program.
    Halt,

    /// (Pseudo) Places raw data in the program.
    Data(Vec<Data<'i>>),
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
    pub fn opcode(&self) -> i64 {
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
