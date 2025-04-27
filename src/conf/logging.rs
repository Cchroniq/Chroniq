use crate::conf::config::Config;
use env_logger::Builder;
use log::LevelFilter;
use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use std::io::Write;

pub fn init_logger(config: &Config) {
    let log_file = BasicRollingFileAppender::new(
        &format!("logs/{}.log", chrono::Local::now().format("%Y-%m-%d")),
        RollingConditionBasic::new().daily(),
        30,
    )
    .unwrap();

    let log_level = match config.log_level.to_uppercase().as_str() {
        "TRACE" => LevelFilter::Trace,
        "DEBUG" => LevelFilter::Debug,
        "INFO" => LevelFilter::Info,
        "WARN" => LevelFilter::Warn,
        "ERROR" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    Builder::new()
        .filter(None, log_level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} - {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .write_style(env_logger::WriteStyle::Always)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();
}
