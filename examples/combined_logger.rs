//! Example: combined logger — stderr for warnings+, file for everything.
//!
//! Run with: cargo run --example combined_logger

use nanologger::{LogLevel, LogOutput, LoggerBuilder};
use std::fs::File;

fn main() {
    let file = File::create("verbose.log").expect("failed to create log file");

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .add_output(LogOutput::term(LogLevel::Warn)) // terminal: Warn and above
        .add_output(LogOutput::writer(LogLevel::Trace, file)) // file: everything
        .init()
        .unwrap();

    nanologger::error!("critical failure");
    nanologger::warn!("something looks off");
    nanologger::info!("this only goes to the file");
    nanologger::debug!("verbose detail — file only");
    nanologger::trace!("very noisy — file only");

    println!("\nTerminal got Error + Warn. File got all five. Check verbose.log.");
}
