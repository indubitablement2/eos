use log::{Level, Metadata, Record};

pub static LOGGER: Logger = Logger;

pub struct Logger;

impl Logger {
    pub fn init() {
        log::set_logger(&LOGGER).expect("can not start logger");
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!(
                "{} - {} - Line: {} - {}",
                record.level(),
                record.file().unwrap_or("*Unknow file*"),
                record.line().unwrap_or_default(),
                record.args()
            );

            match record.level() {
                Level::Error => macroquad::telemetry::log_string(&message),
                Level::Warn => macroquad::telemetry::log_string(&message),
                _ => {}
            }
            println!("{}", message);
        }
    }

    fn flush(&self) {}
}

pub fn write_logs_to_file() {
    // TODO: Write string to file.
}
