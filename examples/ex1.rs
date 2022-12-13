use clap::Parser;
use function_name::named;
#[allow(unused)]
use jlogger::{jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder};

#[derive(Parser, Debug)]
/// Jlogger example program.
struct Cli {
    #[clap(short, long)]
    log_file: Option<String>,
}

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
    let cli = Cli::parse();

    JloggerBuilder::new()
        .max_level(log::LevelFilter::Trace)
        .log_file(cli.log_file.as_deref(), false)
        .log_runtime(true)
        .log_time(jlogger::LogTimeFormat::TimeLocal)
        .build();

    jinfo!("{}", function_name!());
    level3();
}
