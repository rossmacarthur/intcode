mod ast;
mod dynamically;
mod fmt;
mod labels;
mod program;
mod statically;

use crate::ast::Ast;
pub use crate::dynamically::{Input, Result, Run};
use crate::program::Program;

/// Disassemble the intcode program into an AST that can be displayed.
pub fn to_ast(intcode: Vec<i64>, runs: impl IntoIterator<Item = Run>) -> Result<Ast> {
    let mut p = Program::new(intcode);
    dynamically::mark(&mut p, runs)?;
    statically::mark(&mut p);
    labels::assign(&mut p);
    Ok(p.into_ast())
}
