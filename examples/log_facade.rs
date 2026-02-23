//! Example: using nanolog as a backend for the `log` crate facade.
//!
//! Run with: cargo run --example log_facade --features log
//!
//! This lets libraries that depend on `log` (like reqwest, sqlx, etc.)
//! have their log output routed through nanolog automatically.

fn main() {
    #[cfg(not(feature = "log"))]
    {
        eprintln!("This example requires the `log` feature.");
        eprintln!("Run with: cargo run --example log_facade --features log");
        std::process::exit(1);
    }

    #[cfg(feature = "log")]
    {
        use nanolog::{LogLevel, LoggerBuilder};

        // Initialize nanolog — this also registers it as the global `log` backend
        LoggerBuilder::new()
            .level(LogLevel::Trace)
            .timestamps(true)
            .init()
            .unwrap();

        // These use the `log` crate's macros, not nanolog's
        log::error!("from log facade: critical error");
        log::warn!("from log facade: warning");
        log::info!("from log facade: info");
        log::debug!("from log facade: debug detail");
        log::trace!("from log facade: trace");

        // nanolog's own macros still work alongside
        nanolog::info!("from nanolog directly");
    }
}
