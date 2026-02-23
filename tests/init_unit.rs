use nanolog::{init, LogLevel, LoggerBuilder, InitError};

/// Tests run in a single function to guarantee ordering, since OnceLock is per-process.
#[test]
fn test_init_default_and_double_init() {
    // 1. First init() should succeed with default level Info.
    let result = init();
    assert!(result.is_ok(), "First init() should succeed");

    // 2. Second init() should return InitError.
    let result2 = LoggerBuilder::new().level(LogLevel::Debug).init();
    assert!(result2.is_err(), "Second init() should return InitError");

    // 3. Verify the error message.
    let err = result2.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("already initialized"),
        "InitError message should mention 'already initialized', got: {msg}"
    );
}

#[test]
fn test_init_error_is_std_error() {
    let err = InitError;
    let _: &dyn std::error::Error = &err;
}
