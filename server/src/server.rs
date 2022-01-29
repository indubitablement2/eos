use std::time::Instant;

use crate::{
    metascape::Metascape,
    terminal::{performance::PerformanceMetrics, Terminal},
};

pub struct Server {
    metascape: Metascape,
    terminal: Terminal,
    performance: PerformanceMetrics,
}
impl Server {
    pub fn new() -> Self {
        Self {
            metascape: Metascape::new(),
            terminal: Terminal::new().expect("Could not create Terminal."),
            performance: Default::default(),
        }
    }

    /// Return if we should quit.
    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        self.metascape.update();
        let elapsed = now.elapsed().as_secs_f32();

        self.performance.update(elapsed);

        self.terminal.update(&self.performance)
    }
}
