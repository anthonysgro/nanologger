//! Example: including source file and line number in log output.
//!
//! Run with: cargo run --example source_location

use nanologger::{LogLevel, LoggerBuilder};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .source_location(true)
        .timestamps(true)
        .init()
        .unwrap();

    nanologger::error!("something went wrong");
    nanologger::warn!("heads up: {}", "low memory");
    nanologger::info!("started on port {}", 8080);
    nanologger::debug!("payload: {:?}", vec![1, 2, 3]);
    nanologger::trace!("entering main");
}
