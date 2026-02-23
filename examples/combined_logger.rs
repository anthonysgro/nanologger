//! Example: combined logger — stderr for warnings+, file for everything.
//!
//! Run with: cargo run --example combined_logger

use nanolog::{LogLevel, LogOutput, LoggerBuilder};
use std::fs::File;

fn main() {
    let file = File::create("verbose.log").expect("failed to create log file");

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .add_output(LogOutput::term(LogLevel::Warn))          // terminal: Warn and above
        .add_output(LogOutput::writer(LogLevel::Trace, file))  // file: everything
        .init()
        .unwrap();

    nanolog::error!("critical failure");
    nanolog::warn!("something looks off");
    nanolog::info!("this only goes to the file");
    nanolog::debug!("verbose detail — file only");
    nanolog::trace!("very noisy — file only");

    println!("\nTerminal got Error + Warn. File got all five. Check verbose.log.");
}
