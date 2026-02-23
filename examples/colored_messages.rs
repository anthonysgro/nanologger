//! Example: Using nanocolor styling through nanolog
//!
//! nanolog re-exports nanocolor's `Colorize` trait, `style()` helper, and
//! `StyledString` so you can add color and style to your log message content
//! without adding nanocolor as a separate dependency.
//!
//! Run with: cargo run --example colored_messages

use nanolog::{
    info, warn, error, debug, trace,
    Colorize, style,
    LogLevel, LogOutput, LoggerBuilder,
};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .add_output(LogOutput::term(LogLevel::Trace))
        .init()
        .expect("logger init");

    // --- Foreground colors ---
    info!("server listening on {}", "127.0.0.1:3000".cyan());
    error!("connection to {} failed", "db-primary".red());
    warn!("disk usage at {}%", 92.yellow());
    debug!("cache {}", "HIT".green());
    trace!("raw bytes: {}", 1024.magenta());

    // --- Text styles ---
    info!("{} text", "bold".bold());
    info!("{} text", "dim".dim());
    info!("{} text", "italic".italic());
    info!("{} text", "underline".underline());
    info!("{} text", "strikethrough".strikethrough());

    // --- Background colors ---
    error!("{}", "CRITICAL ".white().bold().on_red());
    warn!("{}", "DEPRECATION ".black().on_yellow());
    info!("{}", "SUCCESS ".black().on_bright_green());

    // --- Chaining styles ---
    info!("status: {}", "DEPLOYED".green().bold().underline());
    error!("code {}: {}", 500.red().bold(), "Internal Server Error".red().dim());

    // --- style() for dynamic/formatted values ---
    let version = style(format!("v{}.{}.{}", 0, 1, 0)).cyan().bold();
    info!("running nanolog {}", version);

    // --- Conditional styling with .whenever() ---
    let is_tty = true;
    info!("colored only in TTY: {}", "hello".green().bold().whenever(is_tty));
    info!("never colored: {}", "plain".red().whenever(false));

    // --- Decorative masking ---
    // .mask() makes the value disappear entirely when colors are off
    info!("{}deploy complete", "✓ ".green().bold().mask());
}
