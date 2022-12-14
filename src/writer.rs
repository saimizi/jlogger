use super::level::LevelFilter;
use std::fs::File;
use std::sync::RwLock;
use tracing_subscriber::filter::LevelFilter as TraceLevelFilter;
use tracing_subscriber::fmt::MakeWriter;

pub struct JloggerWriter<'a> {
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

pub struct JloggerMakeWriter {
    log_file: Option<RwLock<File>>,
    log_console: bool,
    max_level: TraceLevelFilter,
}

impl JloggerMakeWriter {
    pub fn new(
        log_file: Option<RwLock<File>>,
        log_console: bool,
        max_level: TraceLevelFilter,
    ) -> Self {
        JloggerMakeWriter {
            log_file,
            log_console,
            max_level,
        }
    }
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

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        let level = if let Ok(l) = std::env::var("JLOGGER_LEVEL") {
            LevelFilter::from(l).into()
        } else {
            self.max_level
        };

        if meta.level() <= &level {
            self.make_writer()
        } else {
            JloggerWriter {
                log_file: None,
                log_console: false,
            }
        }
    }
}
