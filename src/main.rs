mod ast;
mod lex;
mod parse;

use std::io;
use std::io::prelude::*;

use anyhow::Result;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let prog = parse::program(&input);
    println!("{:#?}", prog);
    Ok(())
}
