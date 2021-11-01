use crate::*;
use std::time::Duration;

pub struct Server {
    /// When paused, call to tick will only try to receive the Strategyscape.
    pub paused: bool,
    /// How long since the last strategyscape update.
    pub last_update_delta: Duration,
    pub strategyscape: Option<Strategyscape>,
    strategyscape_runner_handle: StrategyscapeRunnerHandle,
}
impl Server {
    /// Initialize a server with default parameters.
    pub fn new() -> Self {
        Self {
            paused: true,
            last_update_delta: Duration::ZERO,
            strategyscape: Some(Strategyscape::new()),
            strategyscape_runner_handle: StrategyscapeRunnerHandle::new(),
        }
    }

    /// This can be called as often as needed.
    /// Will only request a Strategyscape update once per seconds or a little more.
    pub fn tick(&mut self, delta: Duration) {
        if self.paused && self.strategyscape.is_some() {
            return;
        }

        if let Ok(strategyscape) = self.strategyscape_runner_handle.result_receiver.try_recv() {
            self.strategyscape.replace(strategyscape);
        }

        self.last_update_delta += delta;

        if self.last_update_delta >= Duration::from_secs(1) {
            if let Some(strategyscape) = self.strategyscape.take() {
                self.last_update_delta = Duration::ZERO;

                self.strategyscape_runner_handle
                    .request_sender
                    .send(strategyscape)
                    .expect("Should be hable to send Strategyscape.");
            }
        }
    }
}
