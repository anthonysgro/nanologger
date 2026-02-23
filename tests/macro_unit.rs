use std::process::Command;

/// Verify macros compile with positional, named, and debug format arguments.
#[test]
fn test_macros_accept_format_args() {
    nanologger::init().ok();

    let value = 42;
    let name = "world";
    let items = vec![1, 2, 3];

    // Positional arguments
    nanologger::error!("code {}", value);
    nanologger::warn!("code {}", value);
    nanologger::info!("hello {}", name);

    // Named arguments
    nanologger::debug!("value is {val}", val = value);

    // Debug format
    nanologger::trace!("items: {:?}", items);
}

/// Verify that macro output goes to stderr, not stdout.
///
/// When invoked with __NANOLOG_STDERR_CHECK=1, this test acts as the
/// subprocess: it initializes the logger and emits messages, then exits.
/// Otherwise it spawns itself as a child process and inspects the streams.
#[test]
fn test_macros_write_to_stderr_not_stdout() {
    if std::env::var("__NANOLOG_STDERR_CHECK").is_ok() {
        // Subprocess mode: just log and exit.
        nanologger::init().unwrap();
        nanologger::info!("hello from info");
        nanologger::error!("hello from error");
        return;
    }

    // Parent mode: re-run this exact test as a child process.
    let exe = std::env::current_exe().expect("current_exe");
    let output = Command::new(exe)
        .arg("test_macros_write_to_stderr_not_stdout")
        .arg("--exact")
        .arg("--nocapture")
        .env("__NANOLOG_STDERR_CHECK", "1")
        .output()
        .expect("failed to spawn subprocess");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // stdout must not contain log messages
    assert!(
        !stdout.contains("[INFO]") && !stdout.contains("[ERROR]"),
        "Expected no log output on stdout, got: {:?}",
        stdout
    );

    // stderr must contain our log messages
    assert!(
        stderr.contains("hello from info"),
        "Expected stderr to contain 'hello from info', got: {:?}",
        stderr
    );
    assert!(
        stderr.contains("hello from error"),
        "Expected stderr to contain 'hello from error', got: {:?}",
        stderr
    );
}
