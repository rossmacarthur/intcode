mod fmt;
mod run;

use std::ffi::OsStr;
use std::fmt::Display;
use std::fs;
use std::num::ParseIntError;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::result;

use anyhow::Result;
use clap::{AppSettings, Clap};
use yansi::Paint;

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
        basic: bool,
    },
}

fn parse_program(input: &str) -> result::Result<Vec<i64>, ParseIntError> {
    input.trim().split(',').map(str::parse).collect()
}

fn eprint(header: &str, message: impl Display) {
    if atty::is(atty::Stream::Stdout) {
        eprintln!("{:>12} {}", Paint::green(header).bold(), message);
    } else {
        eprintln!("{:>12} {}", header, message);
    }
}

fn assemble(input: &Path) -> Result<Vec<i64>> {
    let asm = fs::read_to_string(input)?;
    let fmt = fmt::Ansi::new(&asm, input);
    eprint("Assembling", input.display());
    intcode::assemble::to_intcode(&asm)
        .map(|(output, warnings)| {
            for warning in warnings {
                eprintln!("{}", fmt.warning(&warning));
            }
            output
        })
        .map_err(|(errors, warnings)| {
            for warning in warnings {
                eprintln!("{}", fmt.warning(&warning));
            }
            for error in errors {
                eprintln!("{}", fmt.error(&error));
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
    fs::write(
        &output,
        intcode
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(","),
    )?;
    eprint("Finished", output.display());
    Ok(())
}

fn run(input: PathBuf, basic: bool) -> Result<()> {
    let intcode = match input.extension().and_then(OsStr::to_str) {
        Some("s") => assemble(&input)?,
        Some("intcode") | None => parse_program(&fs::read_to_string(&input)?)?,
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
    if basic {
        run::basic(intcode)?;
    } else {
        run::utf8(intcode)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    match Opt::parse() {
        Opt::Build { input, output } => build(input, output),
        Opt::Run { input, basic } => run(input, basic),
    }
}
