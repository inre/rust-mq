use log::{Log, LogRecord, LogLevel, LogMetadata, SetLoggerError, set_logger, LogLevelFilter};

struct ClientLogger;

impl Log for ClientLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{}", record.args());
        }
    }
}

pub fn set_stdout_logger() -> Result<(), SetLoggerError> {
    set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Debug);
        Box::new(ClientLogger)
    })
}
