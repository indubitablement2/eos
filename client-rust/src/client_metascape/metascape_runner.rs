use common::command::TickCmd;
use crossbeam::channel::{bounded, Receiver, Sender};
use metascape::Metascape;
use std::thread::spawn;

/// Run the metascape on a separate thread and communicate with it through channels.
pub struct MetascapeRunnerHandle {
    runner_sender: Sender<MetascapeRunnerCommand>,
    runner_receiver: Receiver<Box<Metascape>>,
    pub metascape: Option<Box<Metascape>>,
}
impl MetascapeRunnerHandle {
    pub fn new(metascape: Metascape) -> Self {
        let (runner_sender, _runner_receiver) = bounded(1);
        let (_runner_sender, runner_receiver) = bounded(1);

        spawn(move || runner(_runner_receiver, _runner_sender));

        Self {
            runner_sender,
            runner_receiver,
            metascape: Some(Box::new(metascape)),
        }
    }

    /// Handle communication with the runner thread.
    ///
    /// Return the metascape if it not being updated.
    pub fn update(&mut self) -> Option<&mut Metascape> {
        // Try to fetch the metascape.
        match self.runner_receiver.try_recv() {
            Ok(metascape) => {
                self.metascape = Some(metascape);
                self.metascape.as_deref_mut()
            }
            Err(crossbeam::channel::TryRecvError::Empty) => {
                // Still updating we already have it.
                self.metascape.as_deref_mut()
            }
            Err(crossbeam::channel::TryRecvError::Disconnected) => {
                // Runner has crashed.
                log::error!("Runner disconnected.");
                panic!()
            }
        }
    }

    /// Ask to step the metascape on another thread.
    ///
    /// **The metascape should be on this thread.**
    ///
    /// You will be notified when it comes back when calling `update()`.
    pub fn step_metascape(&mut self, tick_cmds: Vec<TickCmd>) {
        if let Some(metascape) = self.metascape.take() {
            self.runner_sender
                .send(MetascapeRunnerCommand {
                    metascape,
                    tick_cmds,
                })
                .unwrap();
        } else {
            log::error!(
                "Asked to step the metascape when it was not present on the main thread. Ignoring..."
            );
        }
    }
}
impl Default for MetascapeRunnerHandle {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

struct MetascapeRunnerCommand {
    metascape: Box<Metascape>,
    /// Commands for the current tick.
    tick_cmds: Vec<TickCmd>,
}

fn runner(
    runner_receiver: Receiver<MetascapeRunnerCommand>,
    runner_sender: Sender<Box<Metascape>>,
) {
    while let Ok(mut runner_command) = runner_receiver.recv() {
        // Step the metascape.
        runner_command.metascape.step(&runner_command.tick_cmds);

        // Send back the updated metascape.
        runner_sender.send(runner_command.metascape).unwrap()
    }
}
