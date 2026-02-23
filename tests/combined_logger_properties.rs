use proptest::prelude::*;
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
    fn clear(&self) {
        self.0.lock().unwrap().clear();
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

fn arb_log_level() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::Error),
        Just(LogLevel::Warn),
        Just(LogLevel::Info),
        Just(LogLevel::Debug),
        Just(LogLevel::Trace),
    ]
}

/// All child loggers receive every dispatched message.
///

#[test]
fn test_combined_dispatches_all() {
    // Create 3 Writer outputs, all at Trace level so everything passes through.
    let bufs: Vec<SharedBuf> = (0..3).map(|_| SharedBuf::new()).collect();

    let mut builder = LoggerBuilder::new().level(LogLevel::Trace);
    for buf in &bufs {
        builder = builder.add_output(LogOutput::writer(LogLevel::Trace, buf.clone()));
    }
    builder.init().expect("init should succeed");

    let mut runner = proptest::test_runner::TestRunner::default();

    runner
        .run(&(arb_log_level(), "[a-zA-Z0-9]{1,40}"), |(level, msg)| {
            for buf in &bufs {
                buf.clear();
            }

            nanolog::__log_with_context(level, &msg, "test_mod", "test.rs", 1);

            for (i, buf) in bufs.iter().enumerate() {
                let output = buf.contents();
                prop_assert!(
                    output.contains(&msg),
                    "Child {} should contain message {:?}, got {:?}",
                    i, msg, output
                );
            }

            Ok(())
        })
        .unwrap();
}
