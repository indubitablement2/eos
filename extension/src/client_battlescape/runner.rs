use super::ClientBattlescapeEventHandler;
use crate::battlescape::BattlescapeStateInit;
use crate::battlescape::{command::Commands, events::BattlescapeEventHandler, Battlescape};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread::spawn;

/// Run the battlescape on a separate thread and communicate with it through channels.
pub struct RunnerHandle {
    runner_sender: SyncSender<RunnerCommand>,
    pub runner_receiver: Receiver<ClientBattlescapeEventHandler>,
}
impl RunnerHandle {
    // TODO: Take either state init or a serialized bs.
    pub fn new(bs_state_init: BattlescapeStateInit) -> Self {
        let (runner_sender, _runner_receiver) = sync_channel(1);
        let (_runner_sender, runner_receiver) = sync_channel(1);

        spawn(move || runner(bs_state_init, _runner_receiver, _runner_sender));

        Self {
            runner_sender,
            runner_receiver,
        }
    }

    /// Ask to step the battlescape on another thread.
    pub fn step(&mut self, cmds: Commands, event_handler: ClientBattlescapeEventHandler) {
        self.runner_sender
            .send(RunnerCommand {
                cmds,
                event_handler,
            })
            .unwrap();
    }
}
impl Default for RunnerHandle {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

struct RunnerCommand {
    cmds: Commands,
    event_handler: ClientBattlescapeEventHandler,
}

fn runner(
    bs_state_init: BattlescapeStateInit,
    runner_receiver: Receiver<RunnerCommand>,
    runner_sender: SyncSender<ClientBattlescapeEventHandler>,
) {
    let mut bs = Battlescape::new(bs_state_init);

    while let Ok(runner_command) = runner_receiver.recv() {
        let mut events = bs
            .step(
                &runner_command.cmds,
                BattlescapeEventHandler::Client(runner_command.event_handler),
            )
            .cast_client()
            .unwrap();

        if events.take_hash.is_some() {
            events.take_hash = Some(crc32fast::hash(&bs.serialize()));
        }

        runner_sender.send(events).unwrap()
    }
}
