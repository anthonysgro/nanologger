//! Example: including source file and line number in log output.
//!
//! Run with: cargo run --example source_location

use nanolog::{LogLevel, LoggerBuilder};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .source_location(true)
        .timestamps(true)
        .init()
        .unwrap();

    nanolog::error!("something went wrong");
    nanolog::warn!("heads up: {}", "low memory");
    nanolog::info!("started on port {}", 8080);
    nanolog::debug!("payload: {:?}", vec![1, 2, 3]);
    nanolog::trace!("entering main");
}
