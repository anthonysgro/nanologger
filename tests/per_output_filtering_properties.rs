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

const ALL_LEVELS: [LogLevel; 5] = [
    LogLevel::Error,
    LogLevel::Warn,
    LogLevel::Info,
    LogLevel::Debug,
    LogLevel::Trace,
];

fn arb_log_level() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::Error),
        Just(LogLevel::Warn),
        Just(LogLevel::Info),
        Just(LogLevel::Debug),
        Just(LogLevel::Trace),
    ]
}

/// Each output independently filters by its own level threshold.
///

#[test]
fn test_per_output_level_filtering() {
    // Create one Writer per LogLevel filter, so we have 5 outputs.
    let bufs: Vec<(LogLevel, SharedBuf)> = ALL_LEVELS
        .iter()
        .map(|&lvl| (lvl, SharedBuf::new()))
        .collect();

    let mut builder = LoggerBuilder::new().level(LogLevel::Trace);
    for (lvl, buf) in &bufs {
        builder = builder.add_output(LogOutput::writer(*lvl, buf.clone()));
    }
    builder.init().expect("init should succeed");

    let mut runner = proptest::test_runner::TestRunner::default();

    runner
        .run(&(arb_log_level(), "[a-zA-Z0-9]{1,40}"), |(msg_level, msg)| {
            // Clear all buffers
            for (_, buf) in &bufs {
                buf.clear();
            }

            nanolog::__log_with_context(msg_level, &msg, "test_mod", "test.rs", 1);

            for (out_level, buf) in &bufs {
                let output = buf.contents();
                let should_emit = msg_level <= *out_level;

                if should_emit {
                    prop_assert!(
                        output.contains(&msg),
                        "Output with filter {:?} should contain message at {:?}: {:?}",
                        out_level, msg_level, output
                    );
                } else {
                    prop_assert!(
                        output.is_empty(),
                        "Output with filter {:?} should be empty for message at {:?}: {:?}",
                        out_level, msg_level, output
                    );
                }
            }

            Ok(())
        })
        .unwrap();
}
