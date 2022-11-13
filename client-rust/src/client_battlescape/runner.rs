use battlescape::{bc_event::BattlescapeEvents, commands::BattlescapeCommand, Battlescape};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::thread::spawn;

/// Run the battlescape on a separate thread and communicate with it through channels.
pub struct RunnerHandle {
    runner_sender: Sender<RunnerCommand>,
    runner_receiver: Receiver<(Box<Battlescape>, BattlescapeEvents)>,
    pub bc: Option<(Box<Battlescape>, BattlescapeEvents)>,
}
impl RunnerHandle {
    pub fn new(bc: Battlescape) -> Self {
        let (runner_sender, _runner_receiver) = bounded(1);
        let (_runner_sender, runner_receiver) = bounded(1);

        spawn(move || runner(_runner_receiver, _runner_sender));

        Self {
            runner_sender,
            runner_receiver,
            bc: Some((Box::new(bc), Default::default())),
        }
    }

    /// Handle communication with the runner thread.
    ///
    /// Return the battlescape if it not being updated.
    pub fn update(&mut self) -> &Option<(Box<Battlescape>, BattlescapeEvents)> {
        // Try to fetch the battlescape.
        match self.runner_receiver.try_recv() {
            Ok(bc) => {
                if self.bc.is_none() {
                    self.bc = Some(bc);
                } else {
                    log::error!(
                        "Battlescape runner returned a battlescape, but we already had one."
                    );
                }
                &self.bc
            }
            Err(crossbeam::channel::TryRecvError::Empty) => {
                // Still updating or we already have it.
                &self.bc
            }
            Err(crossbeam::channel::TryRecvError::Disconnected) => {
                // Runner has crashed.
                log::error!("Runner disconnected.");
                panic!()
            }
        }
    }

    /// Ask to step the battlescape on another thread.
    ///
    /// **The battlescape should be on this thread.**
    ///
    /// You will be notified when it comes back when calling `update()`.
    pub fn step(&mut self, cmds: Vec<BattlescapeCommand>) {
        if let Some((bc, _)) = self.bc.take() {
            self.runner_sender.send(RunnerCommand { bc, cmds }).unwrap();
        } else {
            log::error!(
                "Asked to step the battlescape when it was not present on the main thread. Ignoring..."
            );
        }
    }
}
impl Default for RunnerHandle {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

struct RunnerCommand {
    bc: Box<Battlescape>,
    cmds: Vec<BattlescapeCommand>,
}

fn runner(
    runner_receiver: Receiver<RunnerCommand>,
    runner_sender: Sender<(Box<Battlescape>, BattlescapeEvents)>,
) {
    while let Ok(mut runner_command) = runner_receiver.recv() {
        // Step the battlescape.
        let events = runner_command.bc.step(&runner_command.cmds);

        // Send back the updated battlescape.
        runner_sender.send((runner_command.bc, events)).unwrap()
    }
}
