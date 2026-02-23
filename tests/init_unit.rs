use nanologger::{init, InitError, LogLevel, LoggerBuilder};

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

#[test]
fn test_parse_level_error_display() {
    let err: Result<LogLevel, _> = "garbage".parse();
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("garbage"),
        "ParseLevelError should include the invalid input, got: {msg}"
    );
}

/// LoggerBuilder::default() produces the same result as LoggerBuilder::new().
#[test]
fn test_logger_builder_default() {
    let builder: LoggerBuilder = Default::default();
    assert_eq!(builder.get_level(), LogLevel::Info);
}
