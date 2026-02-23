//! Example: using TestLogger for captured test output.
//!
//! TestLogger writes via print!() so Rust's test harness captures the output.
//! Logs only appear when a test fails (or with `cargo test -- --nocapture`).
//!
//! Run with: cargo test --example test_logger -- --nocapture

use nanologger::{LogLevel, LogOutput, LoggerBuilder};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Debug)
        .thread_info(true)
        .add_output(LogOutput::test(LogLevel::Debug))
        .init()
        .unwrap();

    nanologger::error!("something went wrong");
    nanologger::warn!("retries remaining: {}", 3);
    nanologger::info!("server started on port {}", 8080);
    nanologger::debug!("request payload: {:?}", vec![1, 2, 3]);
    nanologger::trace!("this won't appear — below Debug level");

    println!("\nTestLogger output is plain text (no ANSI codes).");
    println!("In a #[test] function, this output is captured by the test harness.");
}
