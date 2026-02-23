//! Example: filtering logs by module path.
//!
//! Run with: cargo run --example module_filter

use nanolog::{LogLevel, LogOutput, LoggerBuilder};

fn main() {
    // Only allow logs from "myapp::db", deny "myapp::db::pool"
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .module_allow(vec!["myapp::db".into()])
        .module_deny(vec!["myapp::db::pool".into()])
        .add_output(LogOutput::writer(LogLevel::Trace, std::io::stderr()))
        .init()
        .unwrap();

    // Simulate what the filter does with different module paths
    let cases = [
        ("myapp::db", true),
        ("myapp::db::query", true),
        ("myapp::db::pool", false),
        ("myapp::db::pool::conn", false),
        ("myapp::web", false),
        ("other_crate", false),
    ];

    for (module, expected) in cases {
        let result = nanolog::matches_module_filter(
            module,
            &["myapp::db".into()],
            &["myapp::db::pool".into()],
        );
        println!("{module:30} => allowed: {result:5} (expected: {expected})");
    }
}
