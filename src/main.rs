use std::fs;
use std::path::PathBuf;
use std::process;
use std::{fmt::Display, path::Path};

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
enum Opt {
    Build {
        /// The input file.
        #[clap()]
        input: PathBuf,

        /// The output file.
        #[clap(long, short)]
        output: Option<PathBuf>,
    },
    Run {
        #[clap()]
        input: PathBuf,
    },
}

fn eprint(header: &str, message: impl Display) {
    if atty::is(atty::Stream::Stdout) {
        eprintln!("{:>12} {}", header.bold().green(), message);
    } else {
        eprintln!("{:>12} {}", header, message);
    }
}

fn assemble(input: &Path) -> Result<String> {
    let asm = fs::read_to_string(input)?;
    eprint("Assembling", input.display());
    assemble::program(&asm).map_err(|err| {
        eprintln!("{}", err.pretty(&asm, input));
        eprintln!(
            "{}{} could not assemble `{}`",
            "error".bold().red(),
            ":".bold(),
            input.display()
        );
        process::exit(1);
    })
}

fn build(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    let output = output.unwrap_or_else(|| input.with_extension("intcode"));
    let intcode = assemble(&input)?;
    fs::write(&output, intcode)?;
    eprint("Finished", output.display());
    Ok(())
}

fn run(input: PathBuf) -> Result<()> {
    let intcode = assemble(&input)?;
    eprint("Running", input.display());
    run::program(&intcode)?;
    Ok(())
}

fn main() -> Result<()> {
    match Opt::parse() {
        Opt::Build { input, output } => build(input, output),
        Opt::Run { input } => run(input),
    }
}
