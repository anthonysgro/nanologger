//! Tests that module filtering works through the `log` facade.
//!
//! This test binary initializes nanologger with a module deny list and verifies
//! that messages from denied modules are filtered out when using `log::info!`.
#![cfg(feature = "log")]

use nanologger::{LogLevel, LogOutput, LoggerBuilder};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// A shared buffer we can inspect after logging.
fn shared_buffer() -> (LogOutput, Arc<Mutex<Cursor<Vec<u8>>>>) {
    let buf = Arc::new(Mutex::new(Cursor::new(Vec::new())));
    let writer = {
        let buf = Arc::clone(&buf);
        // We need a Write impl that writes into our shared buffer.
        // Cursor<Vec<u8>> implements Write, but we need to share it.
        // Use a simple wrapper.
        SharedWriter(buf)
    };
    let output = LogOutput::writer(LogLevel::Trace, writer);
    (output, buf)
}

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
fn test_log_facade_module_filtering() {
    let (output, buf) = shared_buffer();

    // Deny this test module's path so log facade messages from here are filtered.
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .module_deny(vec!["log_facade_module_filter".to_string()])
        .add_output(output)
        .init()
        .expect("init should succeed");

    // This message originates from this module, which is denied.
    log::info!("this should be filtered");

    let data = buf.lock().unwrap();
    let output_str = String::from_utf8_lossy(data.get_ref());
    assert!(
        !output_str.contains("this should be filtered"),
        "denied module's message should not appear in output, got: {output_str}"
    );
}
