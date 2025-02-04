mod driver;
mod error;
pub mod net;
mod platform;

use env_logger::fmt::Formatter;
use log::Record;
use std::io::Write;

fn custom_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let file = record.file().unwrap_or("unknown");
    let line = record.line().unwrap_or(0);

    write!(
        buf,
        "{} [{}] {}: {} ({}:{})\n",
        buf.timestamp_seconds(),
        record.level(),
        record.target(),
        record.args(),
        file,
        line
    )
}

pub fn log_init() {
    env_logger::Builder::new()
        .format(custom_format)
        .filter(None, log::LevelFilter::Info)
        .init();
}
