use clap::Parser;
use function_name::named;
#[allow(unused)]
use jlogger_tracing::{
    jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder, LevelFilter, LogTimeFormat,
};

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
    let log_console = cli.log_file.is_none();

    JloggerBuilder::new()
        .max_level(LevelFilter::TRACE)
        .log_file(cli.log_file.as_ref().map(|a| (a.as_str(), false)))
        .log_console(log_console)
        .log_runtime(true)
        .log_time(LogTimeFormat::TimeLocal)
        .build();

    log::info!("{}", function_name!());
    level3();
}
