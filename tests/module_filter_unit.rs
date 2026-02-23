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

/// Test that macros pass module_path!() and that the allow list filters correctly.
/// Requirements: 11.1, 11.6
#[test]
fn test_module_allow_list_filters_end_to_end() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    // Only allow messages from "module_filter_unit" (this test's module path).
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .module_allow(vec!["module_filter_unit".to_string()])
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    // This uses the macro which captures module_path!() automatically.
    // The module path for this integration test file is "module_filter_unit".
    nanolog::info!("allowed message");

    let output = buf_reader.contents();
    assert!(
        output.contains("allowed message"),
        "Message from allowed module should appear, got: {output:?}"
    );

    // Simulate a message from a different module (not in allow list).
    buf_reader.0.lock().unwrap().clear();
    nanolog::__log_with_context(
        LogLevel::Info,
        "denied message",
        "some_other_module",
        "other.rs",
        1,
    );

    let output = buf_reader.contents();
    assert!(
        !output.contains("denied message"),
        "Message from non-allowed module should be filtered out, got: {output:?}"
    );
}
