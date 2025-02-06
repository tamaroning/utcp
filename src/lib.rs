pub mod driver;
pub mod error;
pub mod net;
pub mod platform;

use env_logger::{Builder, Env, fmt::style};
use log::Level;
use std::io::Write;

pub fn log_init() {
    let env = Env::default()
        .filter("RUST_LOG")
        .write_style("RUST_LOG_STYLE");

    Builder::from_env(env)
        .format(|buf, record| {
            let ts = ""; // buf.timestamp();
            let level = record.level();
            let target = record.target();
            let args = record.args();
            let file = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);

            let gray_style = style::AnsiColor::White.on_default().effects(style::Effects::DIMMED);
            let level_style = match record.level() {
                Level::Trace => style::AnsiColor::Cyan.on_default(),
                Level::Debug => style::AnsiColor::Blue.on_default(),
                Level::Info => style::AnsiColor::Green.on_default(),
                Level::Warn => style::AnsiColor::Yellow.on_default(),
                Level::Error => style::AnsiColor::Red
                    .on_default()
                    .effects(style::Effects::BOLD),
            };
            // debug以下はgra_style
            let args_style = match record.level() {
                Level::Trace | Level::Debug => gray_style,
                _ => style::Style::new(),
            };

            write!(
                buf,
                "{gray_style}{ts}{gray_style:#} {level_style}[{level}]{level_style:#} {target}: {args_style}{args}{args_style:#} ({file}:{line})\n",
            )
        })
        .init();
}
