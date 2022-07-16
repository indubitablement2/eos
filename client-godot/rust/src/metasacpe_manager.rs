use std::thread::spawn;
use crossbeam::channel::{Sender, Receiver, unbounded};
use metascape::{Metascape, offline::OfflineConnectionsManager, MetascapeSave};
use utils::interval::*;

pub enum MetascapeCmd {
    /// Save will be stored in `last_save`.
    Save,
    /// Toggle updating `last_metascape_debug_info_str`.
    ToggleDebugInfoUpdate(bool),
    /// Toggle updating the metascape.
    ToggleMetascapeUpdate(bool),
}

enum MetascapeSignal {
    Save(MetascapeSave),
    DebugInfo(String),
}

/// Run the metascape on a separate thread and communicate with it through channels.
pub struct MetascapeManager {
    cmd_sender: Sender<MetascapeCmd>,
    signal_receiver: Receiver<MetascapeSignal>,
    pub last_metascape_debug_info_str: String,
    pub last_save: Option<MetascapeSave>,
}
impl MetascapeManager {
    pub fn new(metascape: Metascape<OfflineConnectionsManager>) -> Self {
        let (cmd_sender, cmd_receiver) = unbounded();
        let (signal_sender, signal_receiver) = unbounded();

        spawn(move || runner(metascape, signal_sender, cmd_receiver));

        Self {
            cmd_sender,
            signal_receiver,
            last_metascape_debug_info_str: String::new(),
            last_save: None,
        }
    }

    /// Send a command that will be handled on the runner thread the next time it update.
    pub fn send_cmd(&self, cmd: MetascapeCmd) {
        let _ = self.cmd_sender.try_send(cmd);
    }

    /// Handle communication with the runner thread.
    /// 
    /// Return `true` if the metascape is disconnected.
    pub fn update(&mut self) -> bool {
        loop {
            match self.signal_receiver.try_recv() {
                Ok(signal) => match signal {
                    MetascapeSignal::Save(save) => {
                        self.last_save = Some(save);
                    }
                    MetascapeSignal::DebugInfo(info) => {
                        self.last_metascape_debug_info_str = info;
                    }
                }
                Err(err) =>  {
                    return err.is_disconnected();
                }
            }
        }
    }
}

fn runner(
    mut metascape: Metascape<OfflineConnectionsManager>,
    signal_sender: Sender<MetascapeSignal>,
    cmd_receiver: Receiver<MetascapeCmd>
) {
    let mut interval = Interval::new(common::TICK_DURATION);
    let mut send_debug_info = false;
    let mut metascape_update = true;

    loop {
        if metascape_update {
            metascape.update();
        }

        // Handle inbound commands.
        loop {
            match cmd_receiver.try_recv() {
                Ok(cmd) => {
                    match cmd {
                        MetascapeCmd::Save => {
                            let _ = signal_sender.try_send(MetascapeSignal::Save(metascape.save()));
                        }
                        MetascapeCmd::ToggleDebugInfoUpdate(toggle) => {
                            send_debug_info = toggle;
                        }
                        MetascapeCmd::ToggleMetascapeUpdate(toggle) => {
                            metascape_update = toggle;
                        }
                    }
                }
                Err(err) => {
                    if err.is_disconnected() {
                        return;
                    }
                    break;
                }
            }
        }

        if send_debug_info {
            let _ = signal_sender.try_send(MetascapeSignal::DebugInfo(metascape_debug_info_str(&metascape)));
        }

        interval.tick();
    }
}

fn metascape_debug_info_str(metascape: &Metascape<OfflineConnectionsManager>) -> String {
    format!(
        "METASCAPE:
        Num clients: {},
        Num fleets: {},
        Num connections: {}",
        metascape.clients.len(),
        metascape.fleets.len(),
        metascape.connections.len()
    )
}