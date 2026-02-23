//! Tests that source location is captured from `log::Record` through the facade.
#![cfg(feature = "log")]

use nanologger::{LogLevel, LogOutput, LoggerBuilder};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct SharedWriter(Arc<Mutex<Cursor<Vec<u8>>>>);

impl std::io::Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

#[test]
fn test_log_facade_source_location() {
    let buf = Arc::new(Mutex::new(Cursor::new(Vec::new())));
    let writer = SharedWriter(Arc::clone(&buf));
    let output = LogOutput::writer(LogLevel::Trace, writer);

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .source_location(true)
        .add_output(output)
        .init()
        .expect("init should succeed");

    log::info!("location test");

    let data = buf.lock().unwrap();
    let output_str = String::from_utf8_lossy(data.get_ref());

    // The output should contain a source location tag like [tests/log_facade_source_location.rs:35]
    assert!(
        output_str.contains("log_facade_source_location.rs:"),
        "output should contain source file name, got: {output_str}"
    );
    assert!(
        output_str.contains("location test"),
        "output should contain the message text, got: {output_str}"
    );
}
