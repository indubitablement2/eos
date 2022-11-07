use gdnative::prelude::*;
use log::{Level, Metadata, Record};

static LOGGER: GodotLogger = GodotLogger;

pub struct GodotLogger;
impl GodotLogger {
    pub fn init() {
        log::set_logger(&LOGGER).expect("can not start logger");
        log::set_max_level(log::LevelFilter::Trace);

        std::panic::set_hook(Box::new(|panic_info| {
            log::error!("{}", panic_info);
            log::logger().flush();
        }));
    }
}

impl log::Log for GodotLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!(
                "{} - {}, {}:{}",
                record.level(),
                record.args(),
                record.file().unwrap_or("*Unknow file*"),
                record.line().unwrap_or_default(),
            );

            match record.level() {
                Level::Error => {
                    godot_error!("{}", message);
                    crate::client::FATAL_ERROR.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                Level::Warn => godot_warn!("{}", message),
                _ => godot_print!("{}", message),
            }
        }
    }

    fn flush(&self) {
        // TODO: Save to file.
        log::warn!("flush")
    }
}
