use nanolog::{LogLevel, LogOutput, LoggerBuilder};
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

/// Test source_location(true) includes [file:line] in output.
/// Test source_location(false) omits location from output.
/// Test macros pass file!() and line!() to the logger.
/// Requirements: 12.1, 12.2, 12.4
#[test]
fn test_source_location_integration() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .source_location(true)
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    // Use the macro — it captures file!() and line!() automatically.
    nanolog::info!("location enabled");

    let output = buf_reader.contents();

    // Should contain the source file name of this test file.
    assert!(
        output.contains("source_location_unit.rs:"),
        "Output should contain this test's file name, got: {output:?}"
    );

    // Should contain the message.
    assert!(
        output.contains("location enabled"),
        "Output should contain the message, got: {output:?}"
    );

    // The [file:line] should appear between the level tag and the message.
    let tag_pos = output.find("[INFO]").expect("should contain [INFO]");
    let loc_pos = output.find("source_location_unit.rs:").expect("should contain file");
    let msg_pos = output.find("location enabled").expect("should contain message");
    assert!(tag_pos < loc_pos, "Level tag should come before source location");
    assert!(loc_pos < msg_pos, "Source location should come before message");
}
