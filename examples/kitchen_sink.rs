//! Example: fully specced-out nanologger — every feature in one place.
//!
//! This example demonstrates all nanologger capabilities together:
//! - All five log levels
//! - Timestamps
//! - Source location (file:line)
//! - Thread info (thread name / ID)
//! - Module filtering (allow/deny lists)
//! - Combined logger (terminal + file outputs with independent levels)
//! - Colored message content via nanocolor re-exports
//! - Runtime level changes via set_level
//! - Log facade integration (when `--features log` is enabled)
//!
//! Run with:
//!   cargo run --example kitchen_sink
//!   cargo run --example kitchen_sink --features log

use nanologger::{
    debug, error, info, style, trace, warn, Colorize, LogLevel, LogOutput, LoggerBuilder,
};
use std::fs::File;

fn main() {
    // --- Setup: combined logger with all the bells and whistles ---
    let file = File::create("kitchen_sink.log").expect("failed to create log file");

    LoggerBuilder::new()
        .level(LogLevel::Trace) // global: accept everything
        .timestamps(true) // prepend HH:MM:SS.mmm
        .source_location(true) // append [file:line]
        .thread_info(true) // show (thread-name) or (ThreadId)
        .module_allow(vec![
            // only allow our own modules
            "kitchen_sink".into(),
        ])
        .module_deny(vec![
            // but silence a noisy submodule
            "kitchen_sink::noisy".into(),
        ])
        .add_output(LogOutput::term(LogLevel::Info)) // terminal: Info and above
        .add_output(LogOutput::writer(LogLevel::Trace, file)) // file: everything
        .init()
        .expect("logger already initialized");

    // --- All five log levels ---
    eprintln!();
    error!("disk usage at {}%", 98.red().bold());
    warn!("connection pool running low: {} available", 2.yellow());
    info!("server listening on {}", "0.0.0.0:8080".cyan().bold());
    debug!(
        "loaded {} routes in {:?}",
        42,
        std::time::Duration::from_millis(3)
    );
    trace!("entering main()");

    // --- Styled message content ---
    let version = style(format!("v{}.{}.{}", 0, 1, 0)).cyan().bold();
    info!("running nanologger {version}");
    info!("status: {}", "DEPLOYED".green().bold().underline());
    error!("{}", " CRITICAL ".white().bold().on_red());

    // --- Multi-threaded logging ---
    let workers: Vec<_> = (0..3)
        .map(|i| {
            std::thread::Builder::new()
                .name(format!("worker-{i}"))
                .spawn(move || {
                    info!("worker {i} started");
                    debug!("worker {i} processing batch");
                    info!("worker {i} done");
                })
                .unwrap()
        })
        .collect();

    for w in workers {
        w.join().unwrap();
    }

    // --- Runtime level change ---
    info!("switching to Error-only mode");
    nanologger::set_level(LogLevel::Error);
    warn!("this warn is now hidden");
    debug!("this debug is now hidden");
    error!("only errors get through after set_level");

    // Restore for the rest of the demo
    nanologger::set_level(LogLevel::Trace);
    info!("back to Trace level");

    // --- Log facade integration (only with --features log) ---
    #[cfg(feature = "log")]
    {
        log::info!("hello from the log facade");
        log::debug!("log facade debug — goes to file only (below terminal's Info)");
        log::trace!("log facade trace — file only");
    }

    #[cfg(not(feature = "log"))]
    {
        info!(
            "(re-run with {} to see log facade integration)",
            "--features log".dim()
        );
    }

    // --- Module filter demo ---
    // These use matches_module_filter directly to show what the filter does.
    // In real code, the macros handle this automatically via module_path!().
    let cases = [
        "kitchen_sink",
        "kitchen_sink::routes",
        "kitchen_sink::noisy",
        "kitchen_sink::noisy::spam",
        "other_crate",
    ];
    info!("module filter results:");
    for module in cases {
        let allowed = nanologger::matches_module_filter(
            module,
            &["kitchen_sink".into()],
            &["kitchen_sink::noisy".into()],
        );
        info!(
            "  {module:35} => {}",
            if allowed {
                "allowed".green()
            } else {
                "denied".red()
            }
        );
    }

    info!(
        "done — check {} for the full trace-level output",
        "kitchen_sink.log".underline()
    );
    eprintln!();
}
