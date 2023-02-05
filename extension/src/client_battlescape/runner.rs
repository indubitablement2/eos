use super::render::ClientBattlescapeEventHandler;
use crate::battlescape::{command::Commands, events::BattlescapeEventHandler, Battlescape};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread::spawn;

/// Run the battlescape on a separate thread and communicate with it through channels.
pub struct RunnerHandle {
    runner_sender: SyncSender<RunnerCommand>,
    runner_receiver: Receiver<(Box<Battlescape>, Option<ClientBattlescapeEventHandler>)>,
    pub bc: Option<(Box<Battlescape>, Option<ClientBattlescapeEventHandler>)>,
}
impl RunnerHandle {
    pub fn new(bc: Battlescape) -> Self {
        let (runner_sender, _runner_receiver) = sync_channel(1);
        let (_runner_sender, runner_receiver) = sync_channel(1);

        spawn(move || runner(_runner_receiver, _runner_sender));

        Self {
            runner_sender,
            runner_receiver,
            bc: Some((Box::new(bc), None)),
        }
    }

    /// Handle communication with the runner thread.
    ///
    /// Return the battlescape if it not being updated.
    pub fn update(&mut self) -> Option<(&Battlescape, Option<ClientBattlescapeEventHandler>)> {
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
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // Still updating or we already have it.
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // Runner has crashed.
                log::error!("Runner disconnected.");
                panic!()
            }
        }

        if let Some((bc, snapshot)) = &mut self.bc {
            Some((bc, snapshot.take()))
        } else {
            None
        }
    }

    /// Ask to step the battlescape on another thread.
    ///
    /// **The battlescape should be on this thread.**
    ///
    /// You will be notified when it comes back when calling `update()` and it return `Some(...)`.
    pub fn step(&mut self, cmds: Commands, take_snapshot: bool) {
        if let Some((bc, _)) = self.bc.take() {
            self.runner_sender
                .send(RunnerCommand {
                    bc,
                    cmds,
                    take_snapshot,
                })
                .unwrap();
        } else {
            log::warn!(
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
    cmds: Commands,
    take_snapshot: bool,
}

fn runner(
    runner_receiver: Receiver<RunnerCommand>,
    runner_sender: SyncSender<(Box<Battlescape>, Option<ClientBattlescapeEventHandler>)>,
) {
    while let Ok(mut runner_command) = runner_receiver.recv() {
        let events: BattlescapeEventHandler = if runner_command.take_snapshot {
            BattlescapeEventHandler::new_client()
        } else {
            BattlescapeEventHandler::None
        };

        let events = runner_command
            .bc
            .step(&runner_command.cmds, events)
            .cast_client();

        runner_sender.send((runner_command.bc, events)).unwrap()
    }
}
