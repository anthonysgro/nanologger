use nanolog::{LoggerBuilder, LogLevel};

fn main() {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .timestamps(true)
        .init()
        .unwrap();

    nanolog::error!("something went wrong: {}", "disk full");
    nanolog::warn!("retries remaining: {}", 3);
    nanolog::info!("server started on port {}", 8080);
    nanolog::debug!("request payload: {:?}", vec![1, 2, 3]);
    nanolog::trace!("entering function");
}
