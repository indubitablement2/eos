use common::net::online::OnlineConnectionsManager;
use crate::{
    metascape::Metascape,
    terminal::{performance::PerformanceMetrics, Terminal},
};
use std::{time::Instant, sync::Arc};

pub struct Server {
    metascape: Metascape<OnlineConnectionsManager>,
    terminal: Terminal,
    metascape_performance: PerformanceMetrics,
}
impl Server {
    pub fn new() -> Self {
        // Load systems.
        let mut file = File::open("systems.bin").expect("could not open systems.bin");
        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buffer).unwrap();
        let systems =
            bincode::deserialize::<Systems>(&buffer).expect("could not deserialize systems.bin");

        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap());

        Self {
            metascape: Metascape::load(),
            terminal: Terminal::new().expect("Could not create Terminal."),
            metascape_performance: Default::default(),
        }
    }

    /// Return if we should quit.
    pub fn update(&mut self) -> bool {
        let now = Instant::now();

        self.metascape.connections_manager.update();

        self.metascape.update();

        let elapsed = now.elapsed().as_secs_f32();
        self.metascape_performance.update(elapsed);

        self.terminal.update(&self.metascape_performance)
    }

    pub fn clear_terminal(&mut self) {
        self.terminal.clear();
    }
}
