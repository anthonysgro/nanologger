//! Example: logging to a file using WriteLogger.
//!
//! Run with: cargo run --example write_logger

use nanologger::{LogLevel, LogOutput, LoggerBuilder};
use std::fs::File;

fn main() {
    let file = File::create("app.log").expect("failed to create log file");

    LoggerBuilder::new()
        .level(LogLevel::Debug)
        .add_output(LogOutput::writer(LogLevel::Debug, file))
        .init()
        .unwrap();

    nanologger::error!("something went wrong: {}", "disk full");
    nanologger::warn!("retries remaining: {}", 3);
    nanologger::info!("server started on port {}", 8080);
    nanologger::debug!("request payload: {:?}", vec![1, 2, 3]);
    nanologger::trace!("this won't appear — below Debug level");

    println!("Wrote logs to app.log (plain text, no ANSI codes)");
}
