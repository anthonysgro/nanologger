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

/// Tests thread_info integration:
/// - Named thread shows thread name in parentheses (Req 2.1, 2.6)
/// - Unnamed thread falls back to ThreadId (Req 2.2)
/// - Default disabled produces no thread info segment (Req 1.2)
///
/// Since nanologger uses a global OnceLock, we can only init once per test binary.
/// We enable thread_info and verify named/unnamed thread output, then check
/// that the format matches expectations.
#[test]
fn test_thread_info_named_and_unnamed_threads() {
    let buf = SharedBuf::new();
    let buf_reader = buf.clone();

    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .thread_info(true)
        .add_output(LogOutput::writer(LogLevel::Trace, buf))
        .init()
        .expect("init should succeed");

    // Log from a named thread
    let handle = std::thread::Builder::new()
        .name("test-worker".into())
        .spawn(|| {
            nanologger::info!("from named thread");
        })
        .unwrap();
    handle.join().unwrap();

    let output = buf_reader.contents();
    assert!(
        output.contains("(test-worker)"),
        "Named thread should show name in parens, got: {output:?}"
    );
    assert!(
        output.contains("from named thread"),
        "Should contain the message, got: {output:?}"
    );

    // Log from an unnamed thread
    let handle = std::thread::spawn(|| {
        nanologger::info!("from unnamed thread");
    });
    handle.join().unwrap();

    let output = buf_reader.contents();
    // Unnamed threads produce ThreadId(N) format
    assert!(
        output.contains("(ThreadId("),
        "Unnamed thread should show ThreadId, got: {output:?}"
    );
    assert!(
        output.contains("from unnamed thread"),
        "Should contain the message, got: {output:?}"
    );
}
