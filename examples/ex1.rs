use std::fmt::Display;

use clap::Parser;
use function_name::named;
#[allow(unused)]
use {
    jlogger_tracing::{
        jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder, LevelFilter, LogTimeFormat,
    },
    tracing::{debug, error, info, span, trace, warn},
};

#[derive(Debug, Clone, Copy)]
enum TimeFormat {
    None,
    Local,
    TimeStamp,
}

impl Display for TimeFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TimeFormat::None => "none",
            TimeFormat::Local => "local",
            TimeFormat::TimeStamp => "timestamp",
        };

        write!(f, "{}", s)
    }
}

impl From<&str> for TimeFormat {
    fn from(s: &str) -> Self {
        match s {
            "local" => TimeFormat::Local,
            "timestamp" => TimeFormat::TimeStamp,
            _ => TimeFormat::None,
        }
    }
}

impl From<TimeFormat> for LogTimeFormat {
    fn from(t: TimeFormat) -> Self {
        match t {
            TimeFormat::None => LogTimeFormat::TimeNone,
            TimeFormat::Local => LogTimeFormat::TimeLocal,
            TimeFormat::TimeStamp => LogTimeFormat::TimeStamp,
        }
    }
}

#[derive(Parser, Debug)]
/// Jlogger example program.
struct Cli {
    #[clap(short, long)]
    log_file: Option<String>,

    #[clap(short, long, default_value_t = TimeFormat::None)]
    time_format: TimeFormat,
}

#[named]
pub fn level1() {
    let _level1_span = span!(tracing::Level::INFO, "level1_span").entered();
    jtrace!("{}", function_name!());
}

#[named]
pub fn level2() {
    jtrace!("{}", function_name!());
    level1();
}

#[named]
pub fn level3() {
    let _level3_span = span!(tracing::Level::INFO, "level3_span").entered();

    debug!(f_name = function_name!(), line = line!(), msg = "hello");
    level2();
}

#[named]
pub fn main() {
    let cli = Cli::parse();
    let log_console = cli.log_file.is_none();

    // By default, max log level is info.
    // use "JLOGGER_LEVEL=trace" to control the log output at runtime.
    JloggerBuilder::new()
        .max_level(LevelFilter::INFO)
        .log_file(cli.log_file.as_ref().map(|a| (a.as_str(), false)))
        .log_console(log_console)
        .log_runtime(true)
        .log_time(LogTimeFormat::from(cli.time_format))
        .build();

    info!(f_name = function_name!(), line = line!(), msg = "hello");
    level3();
}
