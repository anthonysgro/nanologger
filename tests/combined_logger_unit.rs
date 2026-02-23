use nanologger::{LogLevel, LogOutput, LoggerBuilder};
use std::io::Write;
use std::sync::{Arc, Mutex};

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

/// Test builder API accepts multiple outputs via add_output.
/// Test mixed Term + Writer configuration.
/// Requirements: 10.3, 10.4
#[test]
fn test_combined_logger_multiple_outputs() {
    let buf1 = SharedBuf::new();
    let buf2 = SharedBuf::new();
    let buf1_reader = buf1.clone();
    let buf2_reader = buf2.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .add_output(LogOutput::term(LogLevel::Error)) // Term output, only errors
        .add_output(LogOutput::writer(LogLevel::Trace, buf1)) // Writer 1, all levels
        .add_output(LogOutput::writer(LogLevel::Warn, buf2))  // Writer 2, warn and above
        .init()
        .expect("init should succeed");

    // Log at Info level — should reach buf1 (Trace filter) but not buf2 (Warn filter)
    nanologger::__log_with_context(LogLevel::Info, "info message", "test_mod", "test.rs", 1);

    assert!(
        buf1_reader.contents().contains("info message"),
        "Writer with Trace filter should receive Info message"
    );
    assert!(
        !buf2_reader.contents().contains("info message"),
        "Writer with Warn filter should not receive Info message"
    );

    // Log at Error level — should reach both writers
    nanologger::__log_with_context(LogLevel::Error, "error message", "test_mod", "test.rs", 2);

    assert!(
        buf1_reader.contents().contains("error message"),
        "Writer with Trace filter should receive Error message"
    );
    assert!(
        buf2_reader.contents().contains("error message"),
        "Writer with Warn filter should receive Error message"
    );
}
