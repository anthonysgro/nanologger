use proptest::prelude::*;
use nanolog::{LogLevel, LogOutput, LoggerBuilder};
use std::io::Write;
use std::sync::{Arc, Mutex};

/// A shared buffer that implements Write, allowing us to inspect output
/// after the logger has taken ownership.
#[derive(Clone)]
struct SharedBuf(Arc<Mutex<Vec<u8>>>);

impl SharedBuf {
    fn new() -> Self {
        SharedBuf(Arc::new(Mutex::new(Vec::new())))
    }
    fn contents(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).to_string()
    }
}

impl Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// WriteLogger output is always plain text (no ANSI escapes).
///
/// NOTE: Because the global logger can only be initialized once per process,
/// this test initializes a single logger with Trace level and a Writer output,
/// then tests multiple generated inputs against it. The proptest runner
/// executes within a single process, so we initialize once via std::sync::Once.
#[test]
fn test_write_logger_plain_text() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    // Use proptest's TestRunner manually since we need the global logger
    // initialized before running generated cases.
    let mut runner = proptest::test_runner::TestRunner::default();

    let strategy = (
        prop_oneof![
            Just(LogLevel::Error),
            Just(LogLevel::Warn),
            Just(LogLevel::Info),
            Just(LogLevel::Debug),
            Just(LogLevel::Trace),
        ],
        "[a-zA-Z0-9 _]{1,80}",
    );

    runner
        .run(&strategy, |(level, msg)| {
            // Clear buffer before each test case
            buf_reader.0.lock().unwrap().clear();

            nanolog::__log_with_context(level, &msg, "test_mod", "test.rs", 1);

            let output = buf_reader.contents();

            // Output must contain the message text
            prop_assert!(
                output.contains(&msg),
                "Output should contain message {:?}, got {:?}",
                msg,
                output
            );

            // Output must not contain ANSI escape sequences
            prop_assert!(
                !output.contains("\x1b["),
                "WriteLogger output must not contain ANSI escapes: {:?}",
                output
            );

            // Output must end with newline
            prop_assert!(output.ends_with('\n'), "Output should end with newline");

            Ok(())
        })
        .unwrap();
}
