use crate::metascape::*;
use std::time::Duration;

/// Get client inputs and send back command ~20hz.
/// Update Metascape ~1hz.
pub struct Server {
    /// When paused, call to tick will only try to receive the Metascape.
    pub paused: bool,
    /// How long since the last Metascape update.
    pub last_update_delta: Duration,
    pub metascape: Option<Metascape>,
    metascape_runner_handle: MetascapeRunnerHandle,
}
impl Server {
    /// Initialize a server with default parameters.
    pub fn new() -> Self {
        Self {
            paused: true,
            last_update_delta: Duration::ZERO,
            metascape: Some(Metascape::new()),
            metascape_runner_handle: MetascapeRunnerHandle::new(),
        }
    }

    /// This can be called as often as needed.
    /// Will only request a Metascape update once per seconds or a little more.
    pub fn tick(&mut self, delta: Duration) {
        if let Ok(metascape) = self.metascape_runner_handle.result_receiver.try_recv() {
            self.metascape.replace(metascape);
        }

        if self.paused && self.metascape.is_some() {
            return;
        }

        self.last_update_delta += delta;

        if self.last_update_delta >= Duration::from_secs(1) {
            if let Some(metascape) = self.metascape.take() {
                self.last_update_delta = Duration::ZERO;

                self.metascape_runner_handle
                    .request_sender
                    .send(metascape)
                    .expect("Should be hable to send Metascape.");
            }
        }
    }

    /// Get client inputs and send back commands.
    pub fn battlescape_tick() {

    }

    /// Update the Metascape which influence the Battlescapes. 
    pub fn metascape_tick() {

    }
}
