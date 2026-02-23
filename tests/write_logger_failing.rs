use nanologger::{LogLevel, LogOutput, LoggerBuilder};
use std::io::Write;

/// A writer that always fails on write.
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "simulated failure",
        ))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "simulated failure",
        ))
    }
}

/// Test WriteLogger with a failing writer does not panic.
/// Requirements: 9.5
#[test]
fn test_write_logger_failing_writer_no_panic() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .add_output(LogOutput::writer(LogLevel::Trace, FailingWriter))
        .init()
        .expect("init should succeed");

    // These should silently discard without panicking.
    nanologger::__log_with_context(
        LogLevel::Error,
        "should not panic",
        "test_mod",
        "test.rs",
        1,
    );
    nanologger::__log_with_context(LogLevel::Info, "also fine", "test_mod", "test.rs", 2);
    nanologger::__log_with_context(LogLevel::Trace, "still fine", "test_mod", "test.rs", 3);
}
