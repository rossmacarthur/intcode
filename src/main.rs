mod ast;
mod error;
mod lex;
mod parse;

use std::io;
use std::io::prelude::*;

use anyhow::Result;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    match parse::program(&input) {
        Ok(prog) => {
            println!("{:#?}", prog);
        }
        Err(err) => {
            println!("{}", err.pretty(&input, "<input>"));
        }
    }
    Ok(())
}
