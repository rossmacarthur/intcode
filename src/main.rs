mod assemble;
mod ast;
mod error;
mod lex;
mod parse;

use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{AppSettings, Clap};
use peter::Stylize;

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

fn print(header: &str, message: impl Display) {
    if atty::is(atty::Stream::Stdout) {
        println!("{:>12} {}", header.bold().green(), message);
    } else {
        println!("{:>12} {}", header, message);
    }
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let input = fs::read_to_string(&opt.input)?;

    print("Assembling", opt.input.display());
    match assemble::program(&input) {
        Ok(prog) => {
            let p = match opt.output {
                Some(p) => p,
                None => opt.input.with_extension("intcode"),
            };
            fs::write(&p, prog)?;
            print("Finished", p.display());
            process::exit(0);
        }
        Err(err) => {
            println!("{}", err.pretty(&input, &opt.input));
            println!(
                "{}{} could not assemble `{}`",
                "error".bold().red(),
                ":".bold(),
                opt.input.display()
            );
            process::exit(1);
        }
    }
}
