//! Example: disabling colors programmatically or via NO_COLOR.
//!
//! nanocolor (which nanologger uses for all coloring) respects the NO_COLOR
//! environment variable and also provides a programmatic override.
//!
//! Run with colors:
//!   cargo run --example disable_colors
//!
//! Run without colors (env var):
//!   NO_COLOR=1 cargo run --example disable_colors
//!
//! The example also shows how to force colors off from code using
//! nanocolor::set_colors_override(false).

use nanologger::{LogLevel, LoggerBuilder, Colorize};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .timestamps(true)
        .init()
        .unwrap();

    nanologger::info!("these messages have colors (if your terminal supports them)");
    nanologger::error!("red error");
    nanologger::warn!("yellow warning");
    nanologger::info!("status: {}", "OK".green().bold());

    // Force colors off programmatically
    nanocolor::set_colors_override(false);

    eprintln!();
    nanologger::info!("colors are now disabled via set_colors_override(false)");
    nanologger::error!("plain error");
    nanologger::warn!("plain warning");
    nanologger::info!("status: {}", "OK".green().bold()); // .green().bold() becomes a no-op

    // Restore automatic behavior
    nanocolor::clear_colors_override();

    eprintln!();
    nanologger::info!("colors restored to automatic TTY detection");
}
