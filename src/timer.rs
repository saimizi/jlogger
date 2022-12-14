use std::fs;
use std::io::{BufRead, BufReader};

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum LogTimeFormat {
    TimeStamp,
    TimeLocal,
    TimeNone,
}

pub struct JloggerTimer {
    time_format: LogTimeFormat,
    system_start: i64,
}

impl JloggerTimer {
    pub fn new(time_format: LogTimeFormat) -> Self {
        let now = chrono::Local::now().timestamp();

        let system_start = if let Ok(f) = fs::OpenOptions::new()
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
        };

        Self {
            time_format,
            system_start,
        }
    }
}

impl tracing_subscriber::fmt::time::FormatTime for JloggerTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let time_str = match self.time_format {
            LogTimeFormat::TimeNone => "".to_owned(),
            LogTimeFormat::TimeStamp => {
                let now = chrono::Local::now();
                format!(
                    "{}.{:<09}",
                    now.timestamp() - self.system_start,
                    now.timestamp_nanos() % 1000000000
                )
            }
            LogTimeFormat::TimeLocal => {
                let now = chrono::Local::now();
                format!("{}", now.format("%Y-%m-%d %H:%M:%S"))
            }
        };

        w.write_str(time_str.as_str())
    }
}
