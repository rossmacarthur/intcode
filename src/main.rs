mod lex;

use std::io;
use std::io::prelude::*;

use anyhow::Result;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut tokens = lex::Tokens::new(&input);
    while let Some(token) = tokens.next()? {
        println!("{:?}", token);
    }
    Ok(())
}
