use gdnative::prelude::*;
use log::{Level, Metadata, Record};

pub struct GodotLogger;

impl log::Log for GodotLogger {
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
                Level::Error => godot_error!("{}", message),
                Level::Warn => godot_warn!("{}", message),
                _ => godot_print!("{}", message),
            }
        }
    }

    fn flush(&self) {}
}
