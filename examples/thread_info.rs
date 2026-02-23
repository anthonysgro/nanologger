//! Example: displaying thread info (name or ID) in log output.
//!
//! Run with: cargo run --example thread_info

use nanologger::{LogLevel, LoggerBuilder};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .thread_info(true)
        .timestamps(true)
        .init()
        .unwrap();

    // Main thread — shows "(main)"
    nanologger::info!("logging from the main thread");

    // Named thread — shows "(worker-1)"
    let h1 = std::thread::Builder::new()
        .name("worker-1".into())
        .spawn(|| {
            nanologger::info!("logging from a named thread");
            nanologger::debug!("doing some work");
        })
        .unwrap();

    // Unnamed thread — shows "(ThreadId(N))"
    let h2 = std::thread::spawn(|| {
        nanologger::warn!("logging from an unnamed thread");
    });

    h1.join().unwrap();
    h2.join().unwrap();
}
