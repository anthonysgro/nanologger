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

/// Test that timestamps are included in writer output when enabled.
#[test]
fn test_write_logger_with_timestamps() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .timestamps(true)
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    nanologger::__log_with_context(LogLevel::Info, "timestamped msg", "test_mod", "test.rs", 1);

    let output = buf_reader.contents();
    assert!(
        output.contains("timestamped msg"),
        "Writer should capture the message, got: {output:?}"
    );
    // Timestamp format is HH:MM:SS.mmm
    assert!(
        output.contains(':'),
        "Output should contain a timestamp with colons, got: {output:?}"
    );
}
