mod fmt;
mod log;
mod run;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::num::ParseIntError;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::result;
use std::str::FromStr;

use anyhow::Result;
use clap::{AppSettings, Clap};
use intcode::assemble::Intcode;
use intcode::disassemble;
use intcode::error::ErrorSet;

#[derive(Debug, Clone, Clap)]
#[clap(
    author,
    global_setting = AppSettings::DeriveDisplayOrder,
    global_setting = AppSettings::DisableHelpSubcommand,
    global_setting = AppSettings::DisableVersionForSubcommands,
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
    Unbuild {
        #[clap()]
        input: PathBuf,
        #[clap(long, multiple_occurrences(true))]
        feed: Vec<Feed>,
    },
}

#[derive(Debug, Clone)]
struct Feed(Vec<i64>);

impl FromStr for Feed {
    type Err = ParseIntError;

    fn from_str(s: &str) -> result::Result<Self, ParseIntError> {
        parse_program(s).map(Self)
    }
}

fn parse_program(input: &str) -> result::Result<Vec<i64>, ParseIntError> {
    input.trim().split(',').map(str::parse).collect()
}

fn assemble(path: &Path) -> Result<Vec<i64>> {
    let asm = fs::read_to_string(path)?;
    let fmt = fmt::Ansi::new(&asm, path);
    log::info!("assembling {}", path.display());
    intcode::assemble::to_intcode(&asm)
        .map(|Intcode { output, warnings }| {
            for warning in warnings {
                eprintln!("{}", fmt.warning(&warning));
            }
            output
        })
        .map_err(|ErrorSet { errors, warnings }| {
            for warning in warnings {
                eprintln!("{}", fmt.warning(&warning));
            }
            for error in errors {
                eprintln!("{}", fmt.error(&error));
            }
            log::error!("could not assemble `{}`", path.display());
            process::exit(1);
        })
}

fn build(path: PathBuf, output: Option<PathBuf>) -> Result<()> {
    let output = output.unwrap_or_else(|| path.with_extension("intcode"));
    let intcode = assemble(&path)?;
    fs::write(
        &output,
        intcode
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(","),
    )?;
    log::info!("finished {}", output.display());
    Ok(())
}

fn run(path: PathBuf, basic: bool) -> Result<()> {
    let intcode = match path.extension().and_then(OsStr::to_str) {
        Some("ints") => assemble(&path)?,
        Some("intcode") | None => parse_program(&fs::read_to_string(&path)?)?,
        Some(ext) => {
            log::error!("unrecognized file extension `{}`", ext);
            process::exit(1);
        }
    };
    log::info!("running {}", path.display());
    if basic {
        run::basic(intcode)?;
    } else {
        run::utf8(intcode)?;
    }
    Ok(())
}

fn unbuild(path: PathBuf, feeds: Vec<Feed>) -> Result<()> {
    let intcode = parse_program(&fs::read_to_string(path)?)?;
    let display = disassemble::to_ast(
        intcode,
        feeds
            .into_iter()
            .map(|Feed(i)| disassemble::Run::new().input(disassemble::Input::Static(i))),
    )?
    .to_string();
    io::stdout().lock().write_all(display.as_bytes())?;
    Ok(())
}

fn main() {
    log::init();
    if let Err(err) = match Opt::parse() {
        Opt::Build { input, output } => build(input, output),
        Opt::Run { input, basic } => run(input, basic),
        Opt::Unbuild { input, feed } => unbuild(input, feed),
    } {
        log::error!("{:#}", err);
    }
}
