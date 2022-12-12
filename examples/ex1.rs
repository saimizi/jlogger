#[allow(unused)]
use jlogger::{jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder};
use function_name::named;

#[named]
pub fn level1() {
    jinfo!("{}", function_name!());
}

#[named]
pub fn level2() {
    jinfo!("{}", function_name!());
    level1();
}

#[named]
pub fn level3() {
    jinfo!("{}", function_name!());
    level2();
}

#[named]
pub fn main() {
    JloggerBuilder::new()
        .max_level(log::LevelFilter::Trace)
        .log_runtime(true)
        .log_time(jlogger::LogTimeFormat::TimeLocal)
        .build();

    jinfo!("{}", function_name!());
    level3();
}
