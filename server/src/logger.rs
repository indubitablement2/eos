/// 1) **error**: Fatal error that can not be recovered from.
/// 2) **warn**: Error that can be recovered from, but should not be present in release build.
/// 3) **info**: Info about the current state of the app.
/// 4) **debug**: Like info, but only of interest to dev.
/// 5) **trace**: unused
pub struct Logger;
impl Logger {
    pub fn init() {
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(log::LevelFilter::Trace);
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let message = format!(
                "{} - {}, {}:{}",
                record.level(),
                record.args(),
                record.file().unwrap_or("*Unknow file*"),
                record.line().unwrap_or_default(),
            );

            println!("{}", message);
        }
    }

    fn flush(&self) {
        // TODO: Save to file.
    }
}

static LOGGER: Logger = Logger;
