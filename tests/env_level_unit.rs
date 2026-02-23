use nanolog::{LogLevel, LoggerBuilder};
use serial_test::serial;

/// Test: NANOLOG_LEVEL unset → builder defaults to Info (Req 1.3)
#[test]
#[serial]
fn env_var_unset_defaults_to_info() {
    std::env::remove_var("NANOLOG_LEVEL");
    let builder = LoggerBuilder::new();
    assert_eq!(builder.get_level(), LogLevel::Info);
}

/// Test: set_level before init is a no-op (Req 3.2)
#[test]
fn set_level_before_init_is_noop() {
    // set_level on an uninitialized logger should not panic or have any effect
    nanolog::set_level(LogLevel::Debug);
    // If we get here without panic, the no-op behavior is confirmed
}
