use crate::{
    _metascape2::Metascape,
    terminal::{performance::PerformanceMetrics, Terminal},
};
use std::time::Instant;

pub use crate::connection_manager::*;
pub use ahash::{AHashMap, AHashSet};
pub use common::idx::*;
pub use common::net::packets::*;
pub use common::time::Time;
pub use glam::{vec2, Vec2};

pub struct Server {
    metascape: Metascape,
    terminal: Terminal,
    metascape_performance: PerformanceMetrics,
}
impl Server {
    pub fn new() -> Self {
        Self {
            metascape: Metascape::load(),
            terminal: Terminal::new().expect("Could not create Terminal."),
            metascape_performance: Default::default(),
        }
    }

    /// Return if we should quit.
    pub fn update(&mut self) -> bool {
        let now = Instant::now();

        self.metascape.update();

        let elapsed = now.elapsed().as_secs_f32();
        self.metascape_performance.update(elapsed);

        self.terminal.update(&self.metascape_performance)
    }
}
