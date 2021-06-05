/// Abstract representation of assembly code.

/// A parameter in an instruction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Param<'i> {
    /// A parameter that refers to a variable or label.
    ///
    /// For example the 'x' in the following code:
    /// ```asm
    /// ADD 0, 1, x
    /// ```
    Ident(&'i str),

    /// A parameter that refers to an exact location in the program.
    ///
    /// For example the '7' in the following code:
    /// ```asm
    /// ADD x, y, 7
    /// ```
    Exact(i64),
}

/// An instruction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instr<'i> {
    /// Adds the first two parameters together storing the result in the third.
    Add(Param<'i>, Param<'i>, Param<'i>),
    /// Multiplies the first two parameters together storing the result in the third.
    Multiply(Param<'i>, Param<'i>, Param<'i>),
    /// Places raw data in the program.
    DataByte(Param<'i>),
    /// Halts the program.
    Halt,
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

impl Instr<'_> {
    pub fn opcode(&self) -> i64 {
        match *self {
            Self::Add(_, _, _) => 1,
            Self::Multiply(_, _, _) => 2,
            Self::Halt => 99,
            i => panic!("no opcode for `{:?}`", i),
        }
    }
}
