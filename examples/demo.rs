use nanologger::{LoggerBuilder, LogLevel};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .timestamps(true)
        .init()
        .unwrap();

    nanologger::error!("something went wrong: {}", "disk full");
    nanologger::warn!("retries remaining: {}", 3);
    nanologger::info!("server started on port {}", 8080);
    nanologger::debug!("request payload: {:?}", vec![1, 2, 3]);
    nanologger::trace!("entering function");
}
