//! A simple log utility.

use std::fs::{self, File};
use std::sync::RwLock;
use tracing_subscriber::filter::LevelFilter as TraceLevelFilter;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub enum LevelFilter {
    OFF,
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

impl From<LevelFilter> for TraceLevelFilter {
    fn from(level: LevelFilter) -> Self {
        match level {
            LevelFilter::OFF => TraceLevelFilter::OFF,
            LevelFilter::ERROR => TraceLevelFilter::ERROR,
            LevelFilter::WARN => TraceLevelFilter::WARN,
            LevelFilter::INFO => TraceLevelFilter::INFO,
            LevelFilter::DEBUG => TraceLevelFilter::DEBUG,
            LevelFilter::TRACE => TraceLevelFilter::TRACE,
        }
    }
}

struct JloggerWriter<'a> {
    log_file: Option<&'a RwLock<File>>,
    log_console: bool,
}

impl<'a> std::io::Write for JloggerWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let write_file = self
            .log_file
            .map_or(Ok(0), |fw| fw.write().unwrap().write(buf))?;

        let write_console = if self.log_console {
            std::io::stderr().write(buf)?
        } else {
            0_usize
        };

        if write_file > 0 && write_console > 0 {
            Ok(usize::min(write_file, write_console))
        } else if write_file > 0 {
            Ok(write_file)
        } else if write_console > 0 {
            Ok(write_console)
        } else {
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(lock_writer) = &self.log_file {
            lock_writer.write().unwrap().flush()?;
        }

        if self.log_console {
            std::io::stderr().flush()
        } else {
            Ok(())
        }
    }
}

struct JloggerMakeWriter {
    log_file: Option<RwLock<File>>,
    log_console: bool,
}

impl<'a> MakeWriter<'a> for JloggerMakeWriter {
    type Writer = JloggerWriter<'a>;
    fn make_writer(&'a self) -> Self::Writer {
        if let Some(rw) = &self.log_file {
            JloggerWriter {
                log_file: Some(rw),
                log_console: self.log_console,
            }
        } else {
            JloggerWriter {
                log_file: None,
                log_console: self.log_console,
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd)]
pub enum LogTimeFormat {
    TimeStamp,
    TimeLocal,
    TimeNone,
}

pub struct JloggerBuilder {
    max_level: LevelFilter,
    log_console: bool,
    log_file: Option<String>,
    log_file_append: bool,
    log_runtime: bool,
    time_format: LogTimeFormat,
}

impl Default for JloggerBuilder {
    fn default() -> Self {
        JloggerBuilder::new()
    }
}

impl JloggerBuilder {
    /// Create a new JloggerBuilder which is used to build a Jlogger.
    ///
    /// # Examples
    /// ```
    ///     use log::LevelFilter;
    ///     use jlogger::{JloggerBuilder, LogTimeFormat};
    ///
    ///     JloggerBuilder::new()
    ///        .max_level(LevelFilter::Debug)
    ///        .log_console(true)
    ///        .log_time(LogTimeFormat::TimeStamp)
    ///        .log_file(Some("/tmp/my_log.log"), false)
    ///        .build();
    ///
    /// ```
    pub fn new() -> Self {
        JloggerBuilder {
            max_level: LevelFilter::INFO,
            log_console: true,
            log_file: None,
            log_file_append: true,
            log_runtime: false,
            time_format: LogTimeFormat::TimeNone,
        }
    }

    /// Set the max level to be outputted.
    /// Log messages with a level below it will not be outputted.
    /// At runtime, the log level can be filtered though "JLOGGER_LEVEL" environment variable.
    pub fn max_level(mut self, max_level: LevelFilter) -> Self {
        self.max_level = max_level;
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
    pub fn log_file(mut self, log_file: Option<(&str, bool)>) -> Self {
        if let Some((log_file, append)) = log_file {
            self.log_file = Some(log_file.to_string());
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

        let make_writer = JloggerMakeWriter {
            log_file,
            log_console: self.log_console,
        };

        if self.time_format == LogTimeFormat::TimeNone {
            tracing_subscriber::fmt()
                .with_writer(make_writer)
                .with_target(self.log_runtime)
                .with_max_level(TraceLevelFilter::from(self.max_level))
                .without_time()
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_writer(make_writer)
                .with_target(self.log_runtime)
                .with_max_level(TraceLevelFilter::from(self.max_level))
                .init();
        }
    }
}

#[macro_export]
macro_rules! jerror{
    () => {
        tracing::error!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        tracing::error!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        tracing::error!(
            "{}-{} : {}",
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[macro_export]
macro_rules! jwarn{
    () => {
        tracing::warn!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        tracing::warn!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        tracing::warn!(
            "{}-{} : {}",
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[macro_export]
macro_rules! jinfo{
    () => {
        tracing::info!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        tracing::info!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        tracing::info!(
            "{}-{} : {}",
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[macro_export]
macro_rules! jdebug {
    () => {
        tracing::debug!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        tracing::debug!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        tracing::debug!(
            "{}-{} : {}",
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[macro_export]
macro_rules! jtrace {
    () => {
        tracing::trace!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        tracing::trace!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        tracing::trace!(
            "{}-{} : {}",
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[test]
fn test_debug_macro() {
    use tracing::{debug, info};

    JloggerBuilder::new()
        .max_level(LevelFilter::DEBUG)
        .log_console(true)
        .log_runtime(true)
        .log_time(LogTimeFormat::TimeLocal)
        .log_file(Some(("/tmp/abc", false)))
        .build();

    jdebug!("test: {}", String::from("hello"));
    jdebug!("this is debug");

    std::thread::Builder::new()
        .name("thread1".to_string())
        .spawn(|| {
            debug!(
                "this is debug in the thread {}.",
                std::thread::current().name().unwrap()
            );
            jinfo!("this is info in the thread.");
        })
        .unwrap()
        .join()
        .unwrap();

    jerror!("this is error");
    jinfo!("this is info");
    std::thread::spawn(|| {
        debug!(
            "this is debug in the thread {}.",
            std::thread::current()
                .name()
                .unwrap_or("No thread name set"),
        );
        jinfo!("this is info in the thread.");
    })
    .join()
    .unwrap();
    info!("this is info");
    jdebug!();
    debug!("default");
}
