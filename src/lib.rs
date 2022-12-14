//! A simple log utility.

use std::fs;
use std::sync::RwLock;
use tracing_subscriber::filter::LevelFilter as TraceLevelFilter;

pub use tracing::debug as jdebug;
pub use tracing::error as jerror;
pub use tracing::info as jinfo;
pub use tracing::trace as jtrace;
pub use tracing::warn as jwarn;
pub use tracing_subscriber::fmt::format::FmtSpan;

mod timer;
use timer::JloggerTimer;
pub use timer::LogTimeFormat;

mod level;
pub use level::LevelFilter;

mod writer;
use writer::JloggerMakeWriter;

pub struct JloggerBuilder<'a> {
    max_level: TraceLevelFilter,
    log_console: bool,
    log_file: Option<&'a str>,
    log_file_append: bool,
    log_runtime: bool,
    time_format: LogTimeFormat,
    span_event: FmtSpan,
}

impl<'a> Default for JloggerBuilder<'a> {
    fn default() -> Self {
        JloggerBuilder::new()
    }
}

impl<'a> JloggerBuilder<'a> {
    /// Create a new JloggerBuilder which is used to build a Jlogger.
    ///
    /// # Examples
    /// ```
    ///     use jlogger_tracing::{JloggerBuilder, LogTimeFormat, LevelFilter};
    ///
    ///     JloggerBuilder::new()
    ///        .max_level(LevelFilter::DEBUG)
    ///        .log_console(true)
    ///        .log_time(LogTimeFormat::TimeStamp)
    ///        .log_file(Some(("/tmp/my_log.log", false)))
    ///        .build();
    ///
    /// ```
    pub fn new() -> Self {
        JloggerBuilder {
            max_level: TraceLevelFilter::INFO,
            log_console: true,
            log_file: None,
            log_file_append: true,
            log_runtime: false,
            time_format: LogTimeFormat::TimeNone,
            span_event: FmtSpan::NONE,
        }
    }

    /// Set the max level to be outputted.
    /// Log messages with a level below it will not be outputted.
    /// At runtime, the log level can be filtered though "JLOGGER_LEVEL" environment variable.
    pub fn max_level(mut self, max_level: LevelFilter) -> Self {
        self.max_level = max_level.into();
        self
    }

    /// If enabled, log message will be printed to the console.
    /// Default is true.
    pub fn log_console(mut self, log_console: bool) -> Self {
        self.log_console = log_console;
        self
    }

    /// Log file name.
    /// If specified, log message will be outputted to it.
    /// A tuple (log_file: &str, append: bool) is used to specify the log file.
    /// if "append" is true and the log file already exists, the log message will be appended to
    /// the log file. Otherwise a new log file will be created.
    pub fn log_file(mut self, log_file: Option<(&'a str, bool)>) -> Self {
        if let Some((log_file, append)) = log_file {
            self.log_file = Some(log_file);
            self.log_file_append = append;
        }

        self
    }

    /// Add runtime information to log message.
    /// If the current thread name is set, it will be used as runtime information, otherwise
    /// process name is used
    ///
    /// >DEBUG thread1 : logging from thread thread1.  
    /// >DEBUG jlogger-cac0970c6f073082 : logging from a thread whose name is not set.
    ///
    ///
    pub fn log_runtime(mut self, log_runtime: bool) -> Self {
        self.log_runtime = log_runtime;
        self
    }

    /// Time stamp string format, only take effect when time stamp is enable in the log.
    /// * TimeStamp  
    /// Timestamp (from system boot) will be outputted in the log message.
    /// > 9080.163365118 DEBUG test_debug_macro : src/lib.rs-364 : this is debug  
    /// > 9083.164066687 INFO  test_debug_macro : this is info
    /// * TimeLocal  
    /// Date and time are printed in the log message.  
    /// > 2022-05-17 13:00:03 DEBUG : src/lib.rs-363 : this is debug  
    /// > 2022-05-17 13:00:06 INFO  : this is info
    /// * TimeNone
    /// No timestamp included in the log message.
    pub fn log_time(mut self, time_format: LogTimeFormat) -> Self {
        self.time_format = time_format;
        self
    }

    /// Enable tracing span event.
    /// See tracing_subscriber::fmt::format::FmtSpan for detail
    /// This only take affect when tracing::Span is used.
    pub fn tracing_span_event(mut self, span: FmtSpan) -> Self {
        self.span_event = span;
        self
    }

    /// Build a Jlogger.
    pub fn build(self) {
        let log_file = if let Some(log) = &self.log_file {
            if !self.log_file_append {
                let _ = fs::remove_file(log);
            }

            Some(RwLock::new(
                fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .read(true)
                    .open(log)
                    .unwrap(),
            ))
        } else {
            None
        };

        let make_writer = JloggerMakeWriter::new(log_file, self.log_console, self.max_level);
        let timer = JloggerTimer::new(self.time_format);

        tracing_subscriber::fmt()
            .with_writer(make_writer)
            .with_timer(timer)
            .with_target(self.log_runtime)
            .with_span_events(self.span_event)
            .with_max_level(TraceLevelFilter::TRACE)
            .init();
    }
}

#[test]
fn test_debug_macro() {
    use tracing::{debug, error, info};

    JloggerBuilder::new()
        .max_level(LevelFilter::DEBUG)
        .log_console(true)
        .log_runtime(true)
        .log_time(LogTimeFormat::TimeLocal)
        .log_file(Some(("/tmp/abc", false)))
        .build();

    jdebug!("{} - {} test: {}", file!(), line!(), String::from("hello"));
    debug!("this is debug");

    std::thread::Builder::new()
        .name("thread1".to_string())
        .spawn(|| {
            debug!(
                "this is debug in the thread {}.",
                std::thread::current().name().unwrap()
            );
            info!("this is info in the thread.");
        })
        .unwrap()
        .join()
        .unwrap();

    jerror!(file = file!(), l = line!(), msg = "this is error");
    error!(file = file!(), l = line!(), msg = "this is another error");
    jinfo!("this is info");
    std::thread::spawn(|| {
        debug!(
            "this is debug in the thread {}.",
            std::thread::current()
                .name()
                .unwrap_or("No thread name set"),
        );
        info!("this is info in the thread.");
    })
    .join()
    .unwrap();
    info!("this is info");
    debug!("default");
}
