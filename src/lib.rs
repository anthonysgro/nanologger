//! # nanologger
//!
//! A minimal logger for Rust.
//!
//! nanologger provides five leveled log macros ([`error!`], [`warn!`], [`info!`],
//! [`debug!`], [`trace!`]) that write colored, formatted output to stderr. It
//! supports optional timestamps, source locations, thread info, module
//! filtering, file logging, and combined multi-output configurations.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use nanologger::{LoggerBuilder, LogLevel};
//!
//! LoggerBuilder::new()
//!     .level(LogLevel::Trace)
//!     .timestamps(true)
//!     .init()
//!     .unwrap();
//!
//! nanologger::error!("something went wrong: {}", "disk full");
//! nanologger::warn!("retries remaining: {}", 3);
//! nanologger::info!("server started on port {}", 8080);
//! nanologger::debug!("request payload: {:?}", vec![1, 2, 3]);
//! nanologger::trace!("entering function");
//! ```
//!
//! Or with defaults (Info level, stderr, no timestamps):
//!
//! ```rust,no_run
//! nanologger::init().unwrap();
//! nanologger::info!("hello");
//! ```
//!
//! ## Log levels
//!
//! [`LogLevel`] variants ordered by decreasing severity:
//!
//! | Level | Color | Tag |
//! |-------|-------|-----|
//! | `Error` | Red bold | `[ERROR]` |
//! | `Warn` | Yellow bold | `[WARN]` |
//! | `Info` | Green bold | `[INFO]` |
//! | `Debug` | Blue bold | `[DEBUG]` |
//! | `Trace` | Magenta bold | `[TRACE]` |
//!
//! Messages below the configured level are silently discarded. Colors are
//! automatically disabled when stderr is not a TTY.
//!
//! ## Output destinations
//!
//! [`LogOutput`] controls where log messages go:
//!
//! - [`LogOutput::term`] — stderr with color support
//! - [`LogOutput::writer`] — any `impl Write + Send` (files, buffers, etc.), plain text
//! - [`LogOutput::test`] — via `print!()`, captured by Rust's test harness
//!
//! Multiple outputs can be added to a single logger, each with its own level
//! filter:
//!
//! ```rust,no_run
//! use nanologger::{LogLevel, LogOutput, LoggerBuilder};
//! use std::fs::File;
//!
//! let file = File::create("app.log").unwrap();
//!
//! LoggerBuilder::new()
//!     .level(LogLevel::Trace)
//!     .add_output(LogOutput::term(LogLevel::Warn))          // terminal: Warn+
//!     .add_output(LogOutput::writer(LogLevel::Trace, file))  // file: everything
//!     .init()
//!     .unwrap();
//! ```
//!
//! ## Optional features
//!
//! - **Timestamps** — `.timestamps(true)` prepends `HH:MM:SS.mmm` via [nanotime](https://crates.io/crates/nanotime)
//! - **Source location** — `.source_location(true)` appends `[file:line]` after the level tag
//! - **Thread info** — `.thread_info(true)` shows `(thread-name)` or `(ThreadId(N))`
//! - **Module filtering** — `.module_allow()` / `.module_deny()` for prefix-based filtering
//! - **Runtime level changes** — [`set_level`] adjusts the global level after init
//! - **Env var** — `NANOLOG_LEVEL` sets the default level (case-insensitive)
//!
//! ## `log` facade integration
//!
//! Enable the `log` feature to use nanologger as a backend for the [`log`](https://crates.io/crates/log) crate:
//!
//! ```toml
//! [dependencies]
//! nanologger = { version = "0.1.0", features = ["log"] }
//! ```
//!
//! When initialized, nanologger registers itself via `log::set_logger`, so
//! libraries using `log::info!()` etc. route through nanologger automatically.
//!
//! ## Colored message content
//!
//! nanologger re-exports [`Colorize`], [`style`], and [`StyledString`] from
//! [nanocolor](https://crates.io/crates/nanocolor), so you can style log
//! message content without an extra dependency:
//!
//! ```rust,no_run
//! use nanologger::{info, Colorize, style};
//!
//! info!("listening on {}", "127.0.0.1:3000".cyan());
//!
//! let v = style(format!("v{}.{}.{}", 0, 1, 0)).cyan().bold();
//! info!("running nanologger {}", v);
//! ```

use std::fmt;
use std::io::{IsTerminal, Write};
use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// Log severity levels, ordered from highest to lowest severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };
        f.write_str(s)
    }
}

impl FromStr for LogLevel {
    type Err = ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(ParseLevelError(s.to_string())),
        }
    }
}

/// Error returned when parsing an invalid log level string.
#[derive(Debug, Clone)]
pub struct ParseLevelError(String);

impl fmt::Display for ParseLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid log level: '{}'", self.0)
    }
}

impl std::error::Error for ParseLevelError {}

impl LogLevel {
    /// Returns the bracketed, uppercase tag for log output, e.g. `[ERROR]`.
    /// Converts a LogLevel to its u8 representation (matches the enum discriminant).
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Converts a u8 to a LogLevel. Returns None for values > 4.
    pub fn from_u8(val: u8) -> Option<LogLevel> {
        match val {
            0 => Some(LogLevel::Error),
            1 => Some(LogLevel::Warn),
            2 => Some(LogLevel::Info),
            3 => Some(LogLevel::Debug),
            4 => Some(LogLevel::Trace),
            _ => None,
        }
    }

    /// Returns the bracketed, uppercase tag for log output, e.g. `[ERROR]`.
    /// Padded to 7 chars so all levels align.
    pub fn tag(&self) -> String {
        match self {
            LogLevel::Error => "[ERROR]".to_string(),
            LogLevel::Warn => "[WARN] ".to_string(),
            LogLevel::Info => "[INFO] ".to_string(),
            LogLevel::Debug => "[DEBUG]".to_string(),
            LogLevel::Trace => "[TRACE]".to_string(),
        }
    }
}

// Re-export nanocolor's Colorize trait so users can style log message content
// without adding nanocolor as a separate dependency.
pub use nanocolor::Colorize;
pub use nanocolor::{style, StyledString};

/// Formats a log message with an optional colored, bold level prefix.
///
/// When `use_color` is true, the prefix is bold and colored per level.
/// When `use_color` is false, plain text with no ANSI codes is produced.
#[cfg(test)]
pub(crate) fn format_message(level: LogLevel, message: &str, use_color: bool) -> String {
    format_message_full(level, message, use_color, None, None, None)
}

/// Formats a log message with an optional timestamp and optional colored, bold level prefix.
#[cfg(test)]
pub(crate) fn format_message_with_timestamp(
    level: LogLevel,
    message: &str,
    use_color: bool,
    timestamp: Option<&str>,
) -> String {
    format_message_full(level, message, use_color, timestamp, None, None)
}

/// Core formatting function. Produces the full log line with optional timestamp
/// and optional source location.
///
/// Format: `{timestamp} {bold_colored_prefix} [{file}:{line}] {message_text}\n`
/// Segments are omitted when `None`.
pub(crate) fn format_message_full(
    level: LogLevel,
    message: &str,
    use_color: bool,
    timestamp: Option<&str>,
    source_loc: Option<(&str, u32)>,
    thread_info: Option<&str>,
) -> String {
    let tag = level.tag();
    let ts_part = match timestamp {
        Some(ts) => format!("{ts} "),
        None => String::new(),
    };
    let thread_part = match thread_info {
        Some(info) => format!("({info}) "),
        None => String::new(),
    };
    let loc_part = match source_loc {
        Some((file, line)) => format!("[{file}:{line}] "),
        None => String::new(),
    };
    if use_color {
        let styled = match level {
            LogLevel::Error => tag.red().bold().to_string(),
            LogLevel::Warn => tag.yellow().bold().to_string(),
            LogLevel::Info => tag.green().bold().to_string(),
            LogLevel::Debug => tag.blue().bold().to_string(),
            LogLevel::Trace => tag.magenta().bold().to_string(),
        };
        format!("{ts_part}{thread_part}{styled} {loc_part}{message}\n")
    } else {
        format!("{ts_part}{thread_part}{tag} {loc_part}{message}\n")
    }
}

/// Returns `true` if a message from `module_path` should be emitted given the
/// allow and deny lists.
///
/// - If `allow` is empty, all modules pass the allow check.
/// - Otherwise, `module_path` must start with at least one entry in `allow`.
/// - After the allow check, the module is rejected if `module_path` starts with
///   any entry in `deny`.
pub fn matches_module_filter(module_path: &str, allow: &[String], deny: &[String]) -> bool {
    let allowed = allow.is_empty() || allow.iter().any(|a| module_path.starts_with(a.as_str()));
    if !allowed {
        return false;
    }
    !deny.iter().any(|d| module_path.starts_with(d.as_str()))
}

/// Represents a log output destination.
///
/// Each variant carries its own level filter. `Term` writes colored output to
/// stderr while `Writer` writes plain text to an arbitrary `Write` destination.
pub enum LogOutput {
    /// Logs to stderr with optional color support.
    Term { level: LogLevel },
    /// Logs to an arbitrary `Write` destination in plain text.
    Writer {
        level: LogLevel,
        writer: std::sync::Mutex<Box<dyn Write + Send>>,
    },
    /// Logs plain text via `print!()`, captured by Rust's test harness.
    Test { level: LogLevel },
}

impl LogOutput {
    /// Creates a `Term` output that writes to stderr at the given level.
    pub fn term(level: LogLevel) -> Self {
        LogOutput::Term { level }
    }

    /// Creates a `Writer` output that writes plain text to the given destination.
    pub fn writer(level: LogLevel, w: impl Write + Send + 'static) -> Self {
        LogOutput::Writer {
            level,
            writer: std::sync::Mutex::new(Box::new(w)),
        }
    }

    /// Creates a `Test` output that writes plain text via `print!()`.
    /// Output is captured by Rust's test harness.
    pub fn test(level: LogLevel) -> Self {
        LogOutput::Test { level }
    }
}

// ---------------------------------------------------------------------------
// Global Logger
// ---------------------------------------------------------------------------

/// The global logger. Immutable after initialization.
/// The global logger. Immutable after initialization.
pub struct Logger {
    level: AtomicU8,
    timestamps: bool,
    source_location: bool,
    thread_info: bool,
    module_allow: Vec<String>,
    module_deny: Vec<String>,
    outputs: Vec<LogOutput>,
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Formats the current local time as `HH:MM:SS.mmm` using nanotime.
fn format_current_timestamp() -> String {
    nanotime::NanoTime::now().to_string()
}

impl Logger {
    /// Returns the configured log level.
    pub fn level(&self) -> LogLevel {
        LogLevel::from_u8(self.level.load(Ordering::Relaxed)).unwrap_or(LogLevel::Info)
    }
}

/// Builder for configuring and initializing the global Logger.
/// Builder for configuring and initializing the global Logger.
pub struct LoggerBuilder {
    level: LogLevel,
    timestamps: bool,
    source_location: bool,
    thread_info: bool,
    module_allow: Vec<String>,
    module_deny: Vec<String>,
    outputs: Vec<LogOutput>,
}

impl LoggerBuilder {
    /// Creates a new builder with the default level (`Info`) and timestamps disabled.
    pub fn new() -> Self {
        let default_level = std::env::var("NANOLOG_LEVEL")
            .ok()
            .and_then(|s| LogLevel::from_str(&s).ok())
            .unwrap_or(LogLevel::Info);

        Self {
            level: default_level,
            timestamps: false,
            source_location: false,
            thread_info: false,
            module_allow: Vec::new(),
            module_deny: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Sets the minimum log level.
    pub fn level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Returns the currently configured log level.
    pub fn get_level(&self) -> LogLevel {
        self.level
    }

    /// Enables or disables timestamp prefixes (`HH:MM:SS`) on log messages.
    pub fn timestamps(mut self, enabled: bool) -> Self {
        self.timestamps = enabled;
        self
    }

    /// Enables or disables source location (`[file:line]`) in log output.
    pub fn source_location(mut self, enabled: bool) -> Self {
        self.source_location = enabled;
        self
    }

    /// Enables or disables thread info (thread name or ID) in log output.
    pub fn thread_info(mut self, enabled: bool) -> Self {
        self.thread_info = enabled;
        self
    }

    /// Sets the module allow list. Only messages from modules whose paths start
    /// with an entry in this list will be emitted.
    pub fn module_allow(mut self, modules: Vec<String>) -> Self {
        self.module_allow = modules;
        self
    }

    /// Sets the module deny list. Messages from modules whose paths start with
    /// an entry in this list will be discarded.
    pub fn module_deny(mut self, modules: Vec<String>) -> Self {
        self.module_deny = modules;
        self
    }

    /// Adds a log output destination. Multiple outputs can be added; each
    /// applies its own level filter independently.
    pub fn add_output(mut self, output: LogOutput) -> Self {
        self.outputs.push(output);
        self
    }

    /// Initializes the global logger. Returns `Err(InitError)` if already initialized.
    ///
    /// When the `log` feature is enabled, this also registers the logger with the
    /// `log` facade via `log::set_logger` and `log::set_max_level`.
    pub fn init(self) -> Result<(), InitError> {
        let outputs = if self.outputs.is_empty() {
            vec![LogOutput::Term { level: self.level }]
        } else {
            self.outputs
        };
        let logger = Logger {
            level: AtomicU8::new(self.level.as_u8()),
            timestamps: self.timestamps,
            source_location: self.source_location,
            thread_info: self.thread_info,
            module_allow: self.module_allow,
            module_deny: self.module_deny,
            outputs,
        };
        LOGGER.set(logger).map_err(|_| InitError)?;

        #[cfg(feature = "log")]
        {
            // SAFETY: LOGGER is a static OnceLock that was just set above and will
            // never be dropped, so the reference is valid for 'static.
            let logger_ref: &'static Logger =
                unsafe { &*(LOGGER.get().expect("just set") as *const Logger) };
            log::set_logger(logger_ref).expect("log facade logger already set");
            log::set_max_level(logger_ref.level().to_log_level_filter());
        }

        Ok(())
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned when attempting to initialize the logger more than once.
#[derive(Debug)]
pub struct InitError;

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nanologger: logger already initialized")
    }
}

impl std::error::Error for InitError {}

/// Convenience function: initialize the global logger with default settings (level = Info).
pub fn init() -> Result<(), InitError> {
    LoggerBuilder::new().init()
}

/// Changes the global log level at runtime.
///
/// Has no effect if the logger has not been initialized.
pub fn set_level(level: LogLevel) {
    if let Some(logger) = LOGGER.get() {
        logger.level.store(level.as_u8(), Ordering::Relaxed);

        #[cfg(feature = "log")]
        log::set_max_level(level.to_log_level_filter());
    }
}

/// Hidden public function used by the log macros. Do not call directly.
#[doc(hidden)]
/// Hidden public function used by the log macros. Do not call directly.
#[doc(hidden)]
pub fn __log_with_context(
    level: LogLevel,
    message: &str,
    module_path: &str,
    file: &str,
    line: u32,
) {
    let Some(logger) = LOGGER.get() else {
        return;
    };

    // Global level gate
    if level > logger.level() {
        return;
    }

    // Apply module filter
    if !matches_module_filter(module_path, &logger.module_allow, &logger.module_deny) {
        return;
    }

    let ts = if logger.timestamps {
        Some(format_current_timestamp())
    } else {
        None
    };

    let source_loc = if logger.source_location {
        Some((file, line))
    } else {
        None
    };

    let thread_info_str = if logger.thread_info {
        let current = std::thread::current();
        let info = match current.name() {
            Some(name) => name.to_string(),
            None => format!("{:?}", current.id()),
        };
        Some(info)
    } else {
        None
    };

    for output in &logger.outputs {
        match output {
            LogOutput::Term { level: out_level } => {
                if level > *out_level {
                    continue;
                }
                let use_color = std::io::stderr().is_terminal();
                let formatted = format_message_full(
                    level,
                    message,
                    use_color,
                    ts.as_deref(),
                    source_loc,
                    thread_info_str.as_deref(),
                );
                let mut stderr = std::io::stderr().lock();
                let _ = stderr.write_all(formatted.as_bytes());
            }
            LogOutput::Writer {
                level: out_level,
                writer,
            } => {
                if level > *out_level {
                    continue;
                }
                let formatted = format_message_full(
                    level,
                    message,
                    false,
                    ts.as_deref(),
                    source_loc,
                    thread_info_str.as_deref(),
                );
                if let Ok(mut w) = writer.lock() {
                    let _ = w.write_all(formatted.as_bytes());
                }
            }
            LogOutput::Test { level: out_level } => {
                if level > *out_level {
                    continue;
                }
                let formatted = format_message_full(
                    level,
                    message,
                    false,
                    ts.as_deref(),
                    source_loc,
                    thread_info_str.as_deref(),
                );
                print!("{formatted}");
            }
        }
    }
}

/// Logs a message at the `Error` level.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::__log_with_context($crate::LogLevel::Error, &format!($($arg)*), module_path!(), file!(), line!())
    };
}

/// Logs a message at the `Warn` level.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::__log_with_context($crate::LogLevel::Warn, &format!($($arg)*), module_path!(), file!(), line!())
    };
}

/// Logs a message at the `Info` level.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::__log_with_context($crate::LogLevel::Info, &format!($($arg)*), module_path!(), file!(), line!())
    };
}

/// Logs a message at the `Debug` level.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::__log_with_context($crate::LogLevel::Debug, &format!($($arg)*), module_path!(), file!(), line!())
    };
}

/// Logs a message at the `Trace` level.
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::__log_with_context($crate::LogLevel::Trace, &format!($($arg)*), module_path!(), file!(), line!())
    };
}

// ---------------------------------------------------------------------------
// Log facade integration (feature = "log")
// ---------------------------------------------------------------------------

#[cfg(feature = "log")]
impl LogLevel {
    /// Converts a `log::Level` to a `LogLevel`.
    fn from_log_level(level: log::Level) -> Self {
        match level {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        }
    }

    /// Converts a `LogLevel` to a `log::LevelFilter`.
    fn to_log_level_filter(self) -> log::LevelFilter {
        match self {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

#[cfg(feature = "log")]
impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let level = LogLevel::from_log_level(metadata.level());
        if level > self.level() {
            return false;
        }
        // target() defaults to module_path in the log crate
        matches_module_filter(metadata.target(), &self.module_allow, &self.module_deny)
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = LogLevel::from_log_level(record.level());
        let message = format!("{}", record.args());
        let file = record.file().unwrap_or("");
        let line = record.line().unwrap_or(0);

        let ts = if self.timestamps {
            Some(format_current_timestamp())
        } else {
            None
        };

        let source_loc = if self.source_location {
            Some((file, line))
        } else {
            None
        };

        let thread_info_str = if self.thread_info {
            let current = std::thread::current();
            let info = match current.name() {
                Some(name) => name.to_string(),
                None => format!("{:?}", current.id()),
            };
            Some(info)
        } else {
            None
        };

        for output in &self.outputs {
            match output {
                LogOutput::Term { level: out_level } => {
                    if level > *out_level {
                        continue;
                    }
                    let use_color = std::io::stderr().is_terminal();
                    let formatted = format_message_full(
                        level,
                        &message,
                        use_color,
                        ts.as_deref(),
                        source_loc,
                        thread_info_str.as_deref(),
                    );
                    let mut stderr = std::io::stderr().lock();
                    let _ = stderr.write_all(formatted.as_bytes());
                }
                LogOutput::Writer {
                    level: out_level,
                    writer,
                } => {
                    if level > *out_level {
                        continue;
                    }
                    let formatted = format_message_full(
                        level,
                        &message,
                        false,
                        ts.as_deref(),
                        source_loc,
                        thread_info_str.as_deref(),
                    );
                    if let Ok(mut w) = writer.lock() {
                        let _ = w.write_all(formatted.as_bytes());
                    }
                }
                LogOutput::Test { level: out_level } => {
                    if level > *out_level {
                        continue;
                    }
                    let formatted = format_message_full(
                        level,
                        &message,
                        false,
                        ts.as_deref(),
                        source_loc,
                        thread_info_str.as_deref(),
                    );
                    print!("{formatted}");
                }
            }
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serial_test::serial;

    fn arb_log_level() -> impl Strategy<Value = LogLevel> {
        prop_oneof![
            Just(LogLevel::Error),
            Just(LogLevel::Warn),
            Just(LogLevel::Info),
            Just(LogLevel::Debug),
            Just(LogLevel::Trace),
        ]
    }

    // ── format_message / format_message_with_timestamp unit tests ──

    #[test]
    #[serial]
    fn test_error_color_is_red_bold() {
        nanocolor::set_colors_override(true);
        let output = format_message(LogLevel::Error, "fail", true);
        assert!(
            output.contains("\x1b[1;31m"),
            "Error prefix should be bold red, got: {:?}",
            output
        );
        nanocolor::clear_colors_override();
    }

    #[test]
    #[serial]
    fn test_warn_color_is_yellow_bold() {
        nanocolor::set_colors_override(true);
        let output = format_message(LogLevel::Warn, "caution", true);
        assert!(
            output.contains("\x1b[1;33m"),
            "Warn prefix should be bold yellow, got: {:?}",
            output
        );
        nanocolor::clear_colors_override();
    }

    #[test]
    #[serial]
    fn test_info_color_is_green_bold() {
        nanocolor::set_colors_override(true);
        let output = format_message(LogLevel::Info, "ok", true);
        assert!(
            output.contains("\x1b[1;32m"),
            "Info prefix should be bold green, got: {:?}",
            output
        );
        nanocolor::clear_colors_override();
    }

    #[test]
    #[serial]
    fn test_debug_color_is_blue_bold() {
        nanocolor::set_colors_override(true);
        let output = format_message(LogLevel::Debug, "details", true);
        assert!(
            output.contains("\x1b[1;34m"),
            "Debug prefix should be bold blue, got: {:?}",
            output
        );
        nanocolor::clear_colors_override();
    }

    #[test]
    #[serial]
    fn test_trace_color_is_magenta_bold() {
        nanocolor::set_colors_override(true);
        let output = format_message(LogLevel::Trace, "verbose", true);
        assert!(
            output.contains("\x1b[1;35m"),
            "Trace prefix should be bold magenta, got: {:?}",
            output
        );
        nanocolor::clear_colors_override();
    }

    #[test]
    fn test_plain_mode_no_ansi() {
        let output = format_message(LogLevel::Error, "test", false);
        assert!(
            !output.contains("\x1b["),
            "Plain mode must not contain ANSI sequences"
        );
        assert_eq!(output, "[ERROR] test\n");
    }

    #[test]
    fn test_timestamp_prepended_plain() {
        let output =
            format_message_with_timestamp(LogLevel::Info, "hello", false, Some("14:30:05"));
        assert_eq!(output, "14:30:05 [INFO]  hello\n");
    }

    #[test]
    #[serial]
    fn test_timestamp_prepended_color() {
        nanocolor::set_colors_override(true);
        let output = format_message_with_timestamp(LogLevel::Error, "fail", true, Some("09:00:00"));
        assert!(
            output.starts_with("09:00:00 "),
            "Timestamp should be at the start: {:?}",
            output
        );
        assert!(
            output.contains("\x1b[1;31m"),
            "Should still have bold red: {:?}",
            output
        );
        nanocolor::clear_colors_override();
    }

    #[test]
    fn test_no_timestamp_same_as_format_message() {
        let with = format_message_with_timestamp(LogLevel::Warn, "test", false, None);
        let without = format_message(LogLevel::Warn, "test", false);
        assert_eq!(with, without);
    }

    // ── format_message_full: thread info unit tests ──

    #[test]
    fn test_plain_text_no_ansi_full() {
        let output =
            format_message_full(LogLevel::Info, "plain text check", false, None, None, None);
        assert!(
            !output.contains("\x1b["),
            "Should have no ANSI codes: {output:?}"
        );
        assert!(
            output.contains("[INFO]"),
            "Should contain level tag: {output:?}"
        );
        assert!(
            output.contains("plain text check"),
            "Should contain message: {output:?}"
        );
    }

    #[test]
    fn test_thread_info_format() {
        let output = format_message_full(
            LogLevel::Warn,
            "combined test",
            false,
            None,
            None,
            Some("my-thread"),
        );
        assert!(
            output.contains("(my-thread)"),
            "Should contain thread info: {output:?}"
        );
        assert!(
            output.contains("[WARN]"),
            "Should contain level tag: {output:?}"
        );
        assert!(
            output.contains("combined test"),
            "Should contain message: {output:?}"
        );
        assert!(
            !output.contains("\x1b["),
            "Should have no ANSI codes: {output:?}"
        );
    }

    // ── format_message property tests ──

    proptest! {
        #[test]
        fn prop_message_format_structure(level in arb_log_level(), msg in "[^\x00]{1,100}") {
            let output = format_message(level, &msg, false);
            let tag = level.tag();
            prop_assert!(output.ends_with('\n'));
            let expected = format!("{tag} {msg}\n");
            prop_assert_eq!(output, expected);
        }

        #[test]
        fn prop_plain_text_no_ansi(level in arb_log_level(), msg in "[^\x1b\x00]{0,100}") {
            let output = format_message(level, &msg, false);
            prop_assert!(!output.contains("\x1b["),
                "Plain text output must not contain ANSI escape sequences: {:?}", output);
        }

        #[test]
        fn prop_timestamp_format_structure(
            level in arb_log_level(),
            msg in "[^\x00]{1,100}",
            h in 0u32..24,
            m in 0u32..60,
            s in 0u32..60,
        ) {
            let ts = format!("{h:02}:{m:02}:{s:02}");
            let output = format_message_with_timestamp(level, &msg, false, Some(&ts));
            let tag = level.tag();
            let expected = format!("{ts} {tag} {msg}\n");
            prop_assert_eq!(output, expected);
        }

        #[test]
        fn prop_no_timestamp_matches_format_message(
            level in arb_log_level(),
            msg in "[^\x00]{1,100}",
        ) {
            let with_ts = format_message_with_timestamp(level, &msg, false, None);
            let without = format_message(level, &msg, false);
            prop_assert_eq!(with_ts, without);
        }
    }

    // ── thread info property tests ──

    proptest! {
        #[test]
        fn prop_thread_info_format_and_placement(
            level in arb_log_level(),
            msg in "[a-zA-Z0-9 ]{1,80}",
            thread_name in "[a-zA-Z0-9_-]{1,30}",
            use_ts in proptest::bool::ANY,
            h in 0u32..24,
            m in 0u32..60,
            s in 0u32..60,
        ) {
            let ts_str = format!("{h:02}:{m:02}:{s:02}");
            let ts = if use_ts { Some(ts_str.as_str()) } else { None };
            let output = format_message_full(level, &msg, false, ts, None, Some(&thread_name));
            let tag = level.tag();
            let wrapped = format!("({thread_name})");

            prop_assert!(output.contains(&wrapped),
                "Output should contain {wrapped}: {output:?}");
            let thread_pos = output.find(&wrapped).unwrap();
            let tag_pos = output.find(&tag).unwrap();
            prop_assert!(thread_pos < tag_pos,
                "Thread info should appear before level tag: {output:?}");
            if let Some(ts) = ts {
                let ts_pos = output.find(ts).unwrap();
                prop_assert!(ts_pos < thread_pos,
                    "Timestamp should appear before thread info: {output:?}");
            }
        }

        #[test]
        fn prop_thread_info_disabled_preserves_format(
            level in arb_log_level(),
            msg in "[a-zA-Z0-9 ]{1,80}",
            use_ts in proptest::bool::ANY,
            h in 0u32..24,
            m in 0u32..60,
            s in 0u32..60,
            use_loc in proptest::bool::ANY,
            file in "[a-zA-Z0-9_/]{1,30}",
            line in 1u32..100_000,
        ) {
            let ts_str = format!("{h:02}:{m:02}:{s:02}");
            let ts = if use_ts { Some(ts_str.as_str()) } else { None };
            let loc = if use_loc { Some((file.as_str(), line)) } else { None };
            let with_none = format_message_full(level, &msg, false, ts, loc, None);
            let without = format_message_full(level, &msg, false, ts, loc, None);
            prop_assert_eq!(&with_none, &without,
                "Output with thread_info=None should be identical");
            let tag = level.tag();
            let tag_pos = with_none.find(&tag).unwrap();
            let before_tag = &with_none[..tag_pos];
            prop_assert!(!before_tag.contains('('),
                "No parenthesized thread info should appear when disabled: {with_none:?}");
        }
    }

    // ── source location property tests ──

    proptest! {
        #[test]
        fn prop_source_location_format(
            level in arb_log_level(),
            file in "[a-zA-Z0-9_/\\.]{1,50}",
            line in 1u32..100_000,
            msg in "[^\x00\x1b]{1,100}",
        ) {
            let output = format_message_full(level, &msg, false, None, Some((&file, line)), None);
            let tag = level.tag();
            let loc_tag = format!("[{file}:{line}]");
            prop_assert!(output.ends_with('\n'));
            prop_assert!(output.contains(&loc_tag), "Output should contain {loc_tag}: {output:?}");
            let tag_pos = output.find(&tag).expect("tag must be present");
            let loc_pos = output.find(&loc_tag).expect("loc must be present");
            let msg_pos = output.rfind(&*msg).expect("msg must be present");
            prop_assert!(tag_pos < loc_pos, "Tag should come before location");
            prop_assert!(loc_pos < msg_pos, "Location should come before message");
        }

        #[test]
        fn prop_source_location_omitted(
            level in arb_log_level(),
            msg in "[a-zA-Z0-9 ]{1,100}",
        ) {
            let output = format_message_full(level, &msg, false, None, None, None);
            let tag = level.tag();
            let after_tag = &output[output.find(&tag).unwrap() + tag.len()..];
            let has_source_loc_after_tag = after_tag.contains('[');
            prop_assert!(!has_source_loc_after_tag,
                "Output should not contain source location brackets after tag when disabled: {output:?}");
        }
    }

    // ── test logger (format_message_full with use_color=false) property tests ──

    proptest! {
        #[test]
        fn prop_test_output_contains_no_ansi(
            level in arb_log_level(),
            msg in "[a-zA-Z0-9 ]{1,80}",
            use_ts in proptest::bool::ANY,
            h in 0u32..24,
            m in 0u32..60,
            s in 0u32..60,
            use_loc in proptest::bool::ANY,
            file in "[a-zA-Z0-9_/]{1,30}",
            line in 1u32..100_000,
            use_thread in proptest::bool::ANY,
            thread_name in "[a-zA-Z0-9_-]{1,30}",
        ) {
            let ts_str = format!("{h:02}:{m:02}:{s:02}");
            let ts = if use_ts { Some(ts_str.as_str()) } else { None };
            let loc = if use_loc { Some((file.as_str(), line)) } else { None };
            let thread = if use_thread { Some(thread_name.as_str()) } else { None };
            let output = format_message_full(level, &msg, false, ts, loc, thread);
            prop_assert!(!output.contains("\x1b["),
                "Test output should contain no ANSI escape sequences: {output:?}");
        }
    }

    // ── module filter property tests ──

    fn arb_module_segment() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_]{0,7}".prop_map(|s| s)
    }

    fn arb_module_path() -> impl Strategy<Value = String> {
        prop::collection::vec(arb_module_segment(), 1..=4).prop_map(|segs| segs.join("::"))
    }

    fn arb_filter_list() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(arb_module_path(), 0..=3)
    }

    proptest! {
        #[test]
        fn prop_module_filter_correctness(
            module_path in arb_module_path(),
            allow in arb_filter_list(),
            deny in arb_filter_list(),
        ) {
            let result = matches_module_filter(&module_path, &allow, &deny);
            let pass_allow = allow.is_empty()
                || allow.iter().any(|a| module_path.starts_with(a.as_str()));
            let pass_deny = !deny.iter().any(|d| module_path.starts_with(d.as_str()));
            let expected = pass_allow && pass_deny;
            prop_assert_eq!(
                result, expected,
                "module_path={:?}, allow={:?}, deny={:?}: got {}, expected {}",
                module_path, allow, deny, result, expected
            );
        }
    }

    // ── Property 5: set_level then level() consistency ──
    // Feature: env-and-runtime-level, Property 5: set_level then level() consistency

    proptest! {
        /// set_level then level() should always agree.
        #[test]
        fn prop_set_level_then_level_consistency(level in arb_log_level()) {
            let logger = Logger {
                level: AtomicU8::new(LogLevel::Info.as_u8()),
                timestamps: false,
                source_location: false,
                thread_info: false,
                module_allow: Vec::new(),
                module_deny: Vec::new(),
                outputs: Vec::new(),
            };
            logger.level.store(level.as_u8(), Ordering::Relaxed);
            prop_assert_eq!(logger.level(), level,
                "After storing {:?}, level() should return it", level);
        }
    }
}
