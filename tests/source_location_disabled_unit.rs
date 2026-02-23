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

/// Test source_location(false) omits [file:line] from output.
/// Requirements: 12.2
#[test]
fn test_source_location_disabled() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .source_location(false)
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    nanolog::info!("no location");

    let output = buf_reader.contents();

    assert!(
        output.contains("no location"),
        "Output should contain the message, got: {output:?}"
    );

    // Should NOT contain a source location pattern after the level tag.
    let tag_end = output.find("[INFO]").expect("should contain [INFO]") + "[INFO]".len();
    let after_tag = &output[tag_end..];
    assert!(
        !after_tag.contains("source_location_disabled_unit.rs:"),
        "Output should not contain file:line when source_location is disabled, got: {output:?}"
    );
}
