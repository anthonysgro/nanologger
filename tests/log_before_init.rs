use nanolog::LogLevel;

#[test]
fn test_logging_before_init_does_not_panic() {
    // Call __log_with_context before any init — should silently return without panic.
    nanolog::__log_with_context(LogLevel::Info, "this should not panic", "test_mod", "test.rs", 1);
    nanolog::__log_with_context(LogLevel::Error, "neither should this", "test_mod", "test.rs", 2);
}
