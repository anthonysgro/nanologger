//! Tests for the optional `log` facade integration (feature = "log").
//!
//! These tests only compile when the `log` feature is enabled.
//! Both tests live in a single function because the global logger (OnceLock)
//! and log facade can only be initialized once per process.
#![cfg(feature = "log")]

use nanolog::{LogLevel, LoggerBuilder};

#[test]
fn test_log_facade_integration() {
    // Initialize nanolog with Trace level so all messages pass through.
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .init()
        .expect("init should succeed");

    // 1. Verify log facade macros route through nanolog without panicking.
    log::error!("facade error");
    log::warn!("facade warn");
    log::info!("facade info");
    log::debug!("facade debug");
    log::trace!("facade trace");

    // 2. Verify level conversion: each log::Level should be enabled at Trace filter.
    let levels = [
        log::Level::Error,
        log::Level::Warn,
        log::Level::Info,
        log::Level::Debug,
        log::Level::Trace,
    ];
    for level in levels {
        assert!(
            log::log_enabled!(level),
            "log facade should be enabled for {level:?} when filter is Trace"
        );
    }
}
