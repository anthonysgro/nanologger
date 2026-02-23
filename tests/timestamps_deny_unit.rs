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

/// Exercises timestamps(true) and module_deny() builder paths, plus the
/// timestamp formatting and writer output branches in __log_with_context.
#[test]
fn test_timestamps_and_deny_list_via_writer() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .timestamps(true)
        .module_deny(vec!["blocked_mod".to_string()])
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    // Log a message — exercises timestamp formatting + writer output
    nanologger::__log_with_context(
        LogLevel::Info,
        "timestamped msg",
        "coverage_gaps",
        "coverage_gaps.rs",
        1,
    );

    let output = buf_reader.contents();
    assert!(
        output.contains("timestamped msg"),
        "Writer should capture the message, got: {output:?}"
    );
    // Timestamp format is HH:MM:SS.mmm — look for the colon pattern
    assert!(
        output.contains(':'),
        "Output should contain a timestamp with colons, got: {output:?}"
    );

    // Denied module should be filtered out
    buf_reader.0.lock().unwrap().clear();
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
}
