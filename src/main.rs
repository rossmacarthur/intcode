mod ast;
mod compile;
mod error;
mod lex;
mod parse;

use std::fs;
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{AppSettings, Clap};

#[derive(Debug, Clone, Clap)]
#[clap(
    global_setting = AppSettings::DeriveDisplayOrder,
    global_setting = AppSettings::DisableHelpSubcommand,
    global_setting = AppSettings::GlobalVersion,
    global_setting = AppSettings::VersionlessSubcommands,
)]
struct Opt {
    /// The input file.
    #[clap()]
    input: PathBuf,

    /// The output file.
    #[clap(long, short)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let input = fs::read_to_string(&opt.input)?;

    match compile::program(&input) {
        Ok(prog) => {
            let p = match opt.output {
                Some(p) => p,
                None => opt.input.with_extension("code"),
            };
            fs::write(&p, prog)?;
            println!("Compiled `{}` to `{}`", &opt.input.display(), p.display());
            process::exit(0);
        }
        Err(err) => {
            println!("{}", err.pretty(&input, &opt.input));
            process::exit(1);
        }
    }
}
