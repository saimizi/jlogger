//! A simple log utility.

use log::{self, LevelFilter, Log, Metadata, Record};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::sync::RwLock;

pub enum LogTimeFormat {
    TimeStamp,
    TimeLocal,
    TimeNone,
}

pub struct Jlogger {
    log_console: bool,
    log_file: Option<RwLock<File>>,
    log_runtime: bool,
    time_format: LogTimeFormat,
    system_start: i64,
    max_level: LevelFilter,
}

impl Jlogger {
    fn runtime() -> String {
        std::thread::current()
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let exe_cmd = std::env::current_exe().unwrap();
                exe_cmd
                    .as_path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
    }
}

impl Log for Jlogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let level = if let Ok(l) = std::env::var("JLOGGER_LEVEL") {
            match l.as_str() {
                "off" => LevelFilter::Off,
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Off,
            }
        } else {
            self.max_level
        };

        metadata.level() <= level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut log_message = String::new();

            let now = chrono::Local::now();
            match self.time_format {
                LogTimeFormat::TimeStamp => log_message.push_str({
                    format!(
                        "{}.{:<09} ",
                        now.timestamp() - self.system_start,
                        now.timestamp_nanos() % 1000000000
                    )
                    .as_str()
                }),
                LogTimeFormat::TimeLocal => {
                    log_message.push_str(format!("{} ", now.format("%Y-%m-%d %H:%M:%S")).as_str())
                }

                LogTimeFormat::TimeNone => {}
            }

            log_message.push_str(format!("{:5} ", record.level()).as_str());

            if self.log_runtime {
                log_message.push_str(format!("{} ", Jlogger::runtime()).as_str());
            }

            log_message.push_str(format!(": {}", record.args()).as_str());

            if self.log_console {
                eprintln!("{}", log_message);
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
    ///        .log_file("/tmp/my_log.log")
    ///        .build();
    ///
    /// ```
    pub fn new() -> Self {
        JloggerBuilder {
            max_level: LevelFilter::Info,
            log_console: true,
            log_file: None,
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
    pub fn build(mut self) {
        let now = chrono::Local::now().timestamp();
        let system_start = {
            if let Ok(f) = fs::OpenOptions::new()
                .create(false)
                .write(false)
                .read(true)
                .open("/proc/stat")
            {
                let mut br = BufReader::new(f);
                loop {
                    let mut buf = String::new();
                    if let Ok(n) = br.read_line(&mut buf) {
                        if n == 0 {
                            break now;
                        }

                        if buf.starts_with("btime") {
                            let v: Vec<&str> = buf.split_whitespace().into_iter().collect();
                            break v[1].parse::<i64>().unwrap();
                        }
                    }
                }
            } else {
                now
            }
        };

        let logger = Box::new(Jlogger {
            log_console: self.log_console,
            log_file: self.log_file.take(),
            log_runtime: self.log_runtime,
            time_format: self.time_format,
            system_start,
            max_level: self.max_level,
        });

        log::set_max_level(LevelFilter::Trace);
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

#[macro_export]
macro_rules! jtrace {
    () => {
        log::trace!(
            "{}-{} : arrived.",
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::trace!(
            "{}-{} : {}",
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::trace!(
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
        .log_runtime(true)
        .log_time(LogTimeFormat::TimeLocal)
        .log_file("/tmp/abc")
        .build();

    jdebug!("test: {}", String::from("hello"));
    jdebug!("this is debug");

    std::thread::Builder::new()
        .name("thread1".to_string())
        .spawn(|| {
            log::debug!(
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
        log::debug!(
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
