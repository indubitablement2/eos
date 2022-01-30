use std::time::Instant;
use crate::{
    metascape::Metascape,
    terminal::{performance::PerformanceMetrics, Terminal},
};

pub struct Server {
    metascape: Metascape,
    terminal: Terminal,
    metascape_performance: PerformanceMetrics,
}
impl Server {
    pub fn new() -> Self {
        Self {
            metascape: Metascape::new(),
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
