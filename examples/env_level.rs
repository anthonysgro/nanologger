//! Demonstrates NANOLOG_LEVEL env var and runtime level changes.
//!
//! Try running with different env var values:
//!   NANOLOG_LEVEL=trace cargo run --example env_level
//!   NANOLOG_LEVEL=error cargo run --example env_level
//!   cargo run --example env_level

use nanologger::{LogLevel, LogOutput, LoggerBuilder};

fn main() {
    // The builder reads NANOLOG_LEVEL from the environment automatically.
    // If unset or invalid, it defaults to Info.
    LoggerBuilder::new()
        .add_output(LogOutput::term(LogLevel::Trace))
        .init()
        .unwrap();

    nanologger::info!("logger initialized (level came from NANOLOG_LEVEL or default Info)");
    nanologger::debug!("this only shows if NANOLOG_LEVEL=debug or trace");
    nanologger::trace!("this only shows if NANOLOG_LEVEL=trace");

    // Change the level at runtime — useful after parsing CLI flags like --verbose
    nanologger::set_level(LogLevel::Trace);
    nanologger::info!("switched to Trace at runtime");
    nanologger::debug!("now debug messages are visible");
    nanologger::trace!("and trace too");

    // Tighten it back up
    nanologger::set_level(LogLevel::Error);
    nanologger::warn!("this warn is hidden after set_level(Error)");
    nanologger::error!("only errors come through now");
}
