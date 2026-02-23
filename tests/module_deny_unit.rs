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

/// Test that the deny list filters out matching modules.
#[test]
fn test_module_deny_list_filters_end_to_end() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .module_deny(vec!["blocked_mod".to_string()])
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    // Message from a denied module should be filtered out.
    nanologger::__log_with_context(
        LogLevel::Info,
        "should not appear",
        "blocked_mod::inner",
        "blocked.rs",
        1,
    );
    let output = buf_reader.contents();
    assert!(
        !output.contains("should not appear"),
        "Denied module messages should be filtered, got: {output:?}"
    );

    // Message from a non-denied module should pass through.
    nanologger::__log_with_context(LogLevel::Info, "allowed msg", "other_mod", "other.rs", 1);
    let output = buf_reader.contents();
    assert!(
        output.contains("allowed msg"),
        "Non-denied module messages should appear, got: {output:?}"
    );
}
