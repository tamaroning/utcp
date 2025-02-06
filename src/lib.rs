pub mod driver;
pub mod error;
pub mod net;
pub mod platform;

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
    let loglevel = std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string());
    let loglevel = match loglevel.as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };

    env_logger::Builder::new()
        .format(custom_format)
        .filter(Some("utcp"), loglevel)
        .init();
}
