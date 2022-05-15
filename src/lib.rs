//! A simple log utility.

use log::{self, LevelFilter, Log, Metadata, Record};
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::thread;
use std::sync::RwLock;
use std::time::SystemTime;

pub enum LogTimeFormat {
    TimeStamp,
}

pub struct Jlogger {
    log_console: bool,
    log_file: Option<RwLock<File>>,
    log_mark: bool,
    mark: String,
    log_time: bool,
    time_format: LogTimeFormat,
    start: SystemTime,
}

impl Log for Jlogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if let Ok(level) = std::env::var("JLOGGER_LEVEL") {
            let level = match level.as_str() {
                "off" => LevelFilter::Off,
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Off,
            };

            return metadata.level() <= level;
        }

        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let nanos = self.start.elapsed().unwrap().as_nanos();
            let mark = std::env::var("JLOGGER_MARK").unwrap_or(self.mark.clone());

            let log_mark = self.log_mark && !mark.trim().is_empty();

            let mut log_message = String::new();

            if self.log_time {
                match self.time_format {
                    LogTimeFormat::TimeStamp => log_message.push_str(
                        format!("{:<05}.{:<9} ", nanos / 1000000000, nanos % 1000000000).as_str(),
                    ),
                }
            }

            log_message.push_str(format!("{:5} ", record.level()).as_str());

            if log_mark {
                log_message.push_str(format!("{} ", mark).as_str());
            }

            log_message.push_str(format!(": {}", record.args()).as_str());

            if self.log_console {
                println!("{}", log_message);
            }

            if let Some(f) = &self.log_file {
                let mut fw = f.write().unwrap();
                writeln!(fw, "{}", log_message).unwrap();
            }
        }
    }

    fn flush(&self) {}
}

pub struct JloggerBuilder {
    max_level: LevelFilter,
    log_console: bool,
    log_file: Option<RwLock<File>>,
    log_mark: bool,
    mark: String,
    log_time: bool,
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
    ///     use jlogger::JloggerBuilder;
    ///
    ///     JloggerBuilder::new()
    ///        .max_level(LevelFilter::Debug)
    ///        .log_console(true)
    ///        .log_mark(true, Some("Mark"))
    ///        .log_file("/tmp/mylog.log")
    ///        .build();
    ///
    /// ```
    pub fn new() -> Self {
        let exe_cmd = std::env::current_exe().unwrap();
        let mark = exe_cmd
            .as_path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        JloggerBuilder {
            max_level: LevelFilter::Info,
            log_console: true,
            log_file: None,
            log_mark: false,
            mark,
            log_time: true,
            time_format: LogTimeFormat::TimeStamp,
        }
    }

    /// Set the max level to be outputed.
    /// Log messages with a level below it will not be outputed.
    /// At runtime, the log level can be filterred though "JLOGGER_LEVEL" environment variable.
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
    /// If specifed, log message will be outputed to it.
    pub fn log_file(mut self, log_file: &str) -> Self {
        self.log_file = Some(RwLock::new(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .write(true)
                .open(log_file)
                .unwrap(),
        ));

        self
    }

    /// If enabled, a mark string will be printed together with the log message.
    /// By default, the mark string is set to the process name, it can be specifed though
    /// "JLOGGER_MARK" environment variable.
    pub fn log_mark(mut self, log_mark: bool, mark: Option<&str>) -> Self {
        self.log_mark = log_mark;
        if let Some(m) = mark {
            self.mark = m.to_string();
        }
        self
    }

    /// If enabled, a time stamp string will be printed together with the log message.
    /// Default: enabled.
    pub fn log_time(mut self, log_time: bool) -> Self {
        self.log_time = log_time;
        self
    }

    /// Time stamp string format, only take effect when time stamp is enable in the log.
    pub fn log_time_format(mut self, time_format: LogTimeFormat) -> Self {
        self.time_format = time_format;
        self
    }

    /// Build a Jlogger.
    pub fn build(mut self) {
        let logger = Box::new(Jlogger {
            log_console: self.log_console,
            log_file: self.log_file.take(),
            log_mark: self.log_mark,
            mark: self.mark,
            log_time: self.log_time,
            time_format: self.time_format,
            start: SystemTime::now(),
        });

        log::set_max_level(self.max_level);
        log::set_boxed_logger(logger).unwrap();
    }
}

#[macro_export]
macro_rules! jerror{
    () => {
        log::error!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::error!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::error!(
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
        log::warn!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::warn!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::warn!(
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
        log::info!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::info!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::info!(
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
        log::debug!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::debug!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::debug!(
            "{}-{} : {}",
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[test]
fn test_debug_macro() {
    use log::{debug, info};

    JloggerBuilder::new()
        .max_level(LevelFilter::Debug)
        .log_console(true)
        .log_mark(true, Some("test_debug_macro"))
        .log_time(true)
        .log_file("/tmp/abc")
        .build();

    jdebug!("test: {}", String::from("hello"));
    jdebug!("this is debug");
    jwarn!("this is warn");
    jerror!("this is error");
    jinfo!("this is info");
    info!("this is info");
    jdebug!();
    debug!("default");
}
