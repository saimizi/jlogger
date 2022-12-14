use tracing_subscriber::filter::LevelFilter as TraceLevelFilter;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
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

impl From<String> for LevelFilter {
    fn from(s: String) -> Self {
        match s.as_str() {
            "off" => LevelFilter::OFF,
            "error" => LevelFilter::ERROR,
            "warn" => LevelFilter::WARN,
            "info" => LevelFilter::INFO,
            "debug" => LevelFilter::DEBUG,
            "trace" => LevelFilter::TRACE,
            _ => LevelFilter::OFF,
        }
    }
}
