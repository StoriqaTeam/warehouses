use chrono::prelude::*;
use env_logger::Builder as LogBuilder;
use log_crate::LevelFilter as LogLevelFilter;
use std::env;
use std::io::Write;

pub fn log_environment() -> LogBuilder {
    let mut builder = LogBuilder::new();
    builder
        .format(|formatter, record| {
            let now = Utc::now();
            writeln!(
                formatter,
                "{} - {} - {}",
                now.to_rfc3339(),
                record.level(),
                record.args()
            )
        })
        .filter(None, LogLevelFilter::Info);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder
}

pub fn acquired_db_connection<T>(_: &T) {
    debug!("Acquired DB connection");
}
