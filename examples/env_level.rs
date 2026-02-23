//! Demonstrates NANOLOG_LEVEL env var and runtime level changes.
//!
//! Try running with different env var values:
//!   NANOLOG_LEVEL=trace cargo run --example env_level
//!   NANOLOG_LEVEL=error cargo run --example env_level
//!   cargo run --example env_level

use nanolog::{LoggerBuilder, LogLevel, LogOutput};

fn main() {
    // The builder reads NANOLOG_LEVEL from the environment automatically.
    // If unset or invalid, it defaults to Info.
    LoggerBuilder::new()
        .add_output(LogOutput::term(LogLevel::Trace))
        .init()
        .unwrap();

    nanolog::info!("logger initialized (level came from NANOLOG_LEVEL or default Info)");
    nanolog::debug!("this only shows if NANOLOG_LEVEL=debug or trace");
    nanolog::trace!("this only shows if NANOLOG_LEVEL=trace");

    // Change the level at runtime — useful after parsing CLI flags like --verbose
    nanolog::set_level(LogLevel::Trace);
    nanolog::info!("switched to Trace at runtime");
    nanolog::debug!("now debug messages are visible");
    nanolog::trace!("and trace too");

    // Tighten it back up
    nanolog::set_level(LogLevel::Error);
    nanolog::warn!("this warn is hidden after set_level(Error)");
    nanolog::error!("only errors come through now");
}
