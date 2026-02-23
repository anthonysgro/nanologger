//! Example: using TestLogger for captured test output.
//!
//! TestLogger writes via print!() so Rust's test harness captures the output.
//! Logs only appear when a test fails (or with `cargo test -- --nocapture`).
//!
//! Run with: cargo test --example test_logger -- --nocapture

use nanolog::{LogLevel, LogOutput, LoggerBuilder};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Debug)
        .thread_info(true)
        .add_output(LogOutput::test(LogLevel::Debug))
        .init()
        .unwrap();

    nanolog::error!("something went wrong");
    nanolog::warn!("retries remaining: {}", 3);
    nanolog::info!("server started on port {}", 8080);
    nanolog::debug!("request payload: {:?}", vec![1, 2, 3]);
    nanolog::trace!("this won't appear — below Debug level");

    println!("\nTestLogger output is plain text (no ANSI codes).");
    println!("In a #[test] function, this output is captured by the test harness.");
}
