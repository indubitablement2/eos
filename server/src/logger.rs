use super::*;

pub struct Logger;
impl Logger {
    pub fn init() {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
    }
}
impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        println!(
            "{} {}:{} {}",
            record.level(),
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
            record.args()
        );
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;
