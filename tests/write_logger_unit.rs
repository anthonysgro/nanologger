use nanolog::{LogLevel, LogOutput, LoggerBuilder};
use std::io::Write;
use std::sync::{Arc, Mutex};

/// A shared buffer that implements Write, allowing inspection after the logger owns it.
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

/// Test WriteLogger backed by Vec<u8> receives formatted output.
/// Requirements: 9.1
#[test]
fn test_write_logger_receives_output() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    nanolog::__log_with_context(LogLevel::Info, "hello write logger", "test_mod", "test.rs", 1);

    let output = buf_reader.contents();
    assert!(output.contains("hello write logger"), "WriteLogger should capture the message, got: {output:?}");
    assert!(output.contains("[INFO]"), "Output should contain level tag");
    assert!(output.ends_with('\n'), "Output should end with newline");
}
