pub use log::{error, info, warn};
use log::{Level, LevelFilter, Log, Metadata, Record};

static LOGGER: Logger = Logger;

struct Logger;

impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let header = match record.level() {
            Level::Trace => yansi::Paint::fixed(244, "trace"),
            Level::Debug => yansi::Paint::default("debug"),
            Level::Info => yansi::Paint::green("info"),
            Level::Warn => yansi::Paint::yellow("warn"),
            Level::Error => yansi::Paint::red("error"),
        }
        .bold();
        let colon = yansi::Paint::default(":").bold();
        eprintln!("{}{} {}", header, colon, record.args());
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap()
}
