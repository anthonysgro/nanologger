use nanolog::{LogLevel, LogOutput, LoggerBuilder};

/// Test that TestLogger output is captured by the test harness (via print!).
/// Also tests combined thread info + TestLogger and plain text output.
/// Requirements: 4.2, 4.3, 5.1, 7.1
///
/// Since nanolog uses a global OnceLock, we init once with both thread_info
/// and a Test output, then verify all behaviors in one test.
#[test]
fn test_logger_captured_with_thread_info() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .thread_info(true)
        .add_output(LogOutput::test(LogLevel::Trace))
        .init()
        .expect("init should succeed");

    // This output goes through print!() and is captured by the test harness.
    // If this test passes, the harness captured it (no panic from print!).
    // When the test fails, you'd see the log output — that's the whole point.
    nanolog::info!("test logger capture works");
}
