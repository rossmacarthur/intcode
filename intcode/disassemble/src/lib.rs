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

    let mut labels = labels::unique();

    dynamically::mark(&mut p, runs)?;
    labels::assign(&mut p, &mut labels);
    log::info!("{:.1}% marked after dynamic marking", p.percent_marked());

    statically::mark(&mut p);
    labels::assign(&mut p, &mut labels);
    log::info!("{:.1}% marked after static marking", p.percent_marked());

    Ok(p.into_ast())
}
