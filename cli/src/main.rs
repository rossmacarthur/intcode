use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::{fmt::Display, path::Path};

use anyhow::Result;
use clap::{AppSettings, Clap};
use yansi::Paint;

use core::{Error, Pretty, Warning};

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

        #[clap(long)]
        utf8: bool,
    },
}

fn eprint(header: &str, message: impl Display) {
    if atty::is(atty::Stream::Stdout) {
        eprintln!("{:>12} {}", Paint::green(header).bold(), message);
    } else {
        eprintln!("{:>12} {}", header, message);
    }
}

fn assemble(input: &Path) -> Result<String> {
    let asm = fs::read_to_string(input)?;
    let fmt = Pretty::new(&asm).filename(input);
    eprint("Assembling", input.display());
    core::assemble::to_intcode(&asm)
        .map(|(output, warnings)| {
            for Warning { msg, span } in warnings {
                eprintln!("{}", fmt.warn(msg, span));
            }
            output
        })
        .map_err(|(errors, warnings)| {
            for Warning { msg, span } in warnings {
                eprintln!("{}", fmt.warn(msg, span));
            }
            for Error { msg, span } in errors {
                eprintln!("{}", fmt.error(msg, span));
            }
            eprintln!(
                "{}{} could not assemble `{}`",
                Paint::red("error").bold(),
                Paint::default(":").bold(),
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

fn run(input: PathBuf, utf8: bool) -> Result<()> {
    let intcode = match input.extension().and_then(OsStr::to_str) {
        Some("s") => assemble(&input)?,
        Some("intcode") | None => fs::read_to_string(&input)?,
        Some(ext) => {
            eprintln!(
                "{}{} unrecognized file extension `{}`",
                Paint::red("error").bold(),
                Paint::default(":").bold(),
                ext
            );
            process::exit(1);
        }
    };
    eprint("Running", input.display());
    if utf8 {
        core::run::intcode_utf8(&intcode)?;
    } else {
        core::run::intcode(&intcode)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    match Opt::parse() {
        Opt::Build { input, output } => build(input, output),
        Opt::Run { input, utf8 } => run(input, utf8),
    }
}
