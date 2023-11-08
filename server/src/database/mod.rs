use super::*;

pub enum DatabaseCommand {
    // todo
}

pub struct Database {
    inbound: Receiver<(BattlescapeId, Vec<DatabaseCommand>)>,
    new_battlescape_sender: Sender<Box<Battlescape>>,

    battlescapes: AHashMap<BattlescapeId, ()>,

    next_client_id: ClientId,
    clients: AHashMap<ClientId, ()>,
    username: AHashMap<String, ClientId>,
    client_connection: AHashMap<ClientId, ()>,
}
impl Database {
    pub fn queue(cmd: DatabaseCommand) {
        CMD_QUEUE.with(|queue| queue.borrow_mut().push(cmd));
    }

    pub fn flush(battlescape_id: BattlescapeId) {
        let cmds = CMD_QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()));
        unsafe {
            OUTBOUND
                .as_ref()
                .unwrap_unchecked()
                .send((battlescape_id, cmds))
                .unwrap();
        }
    }

    pub fn start(new_battlescape_sender: Sender<Box<Battlescape>>) {
        let (outbound, inbound) = sync_channel(512);
        unsafe {
            OUTBOUND = Some(outbound);
        }

        // TODO: load from disk
        Self {
            inbound,
            new_battlescape_sender,

            battlescapes: Default::default(),

            next_client_id: ClientId(0),
            clients: Default::default(),
            username: Default::default(),
            client_connection: Default::default(),
        }
        .run();
    }

    fn run(mut self) {
        loop {
            let (battlescape_id, cmds) = self.inbound.recv().unwrap();
            for cmd in cmds {
                self.handle_command(battlescape_id, cmd);
            }

            // TODO: chech if we should start a save
        }
    }

    fn handle_command(&mut self, battlescape_id: BattlescapeId, cmd: DatabaseCommand) {
        match cmd {
            // todo
        }
    }
}

static mut OUTBOUND: Option<SyncSender<(BattlescapeId, Vec<DatabaseCommand>)>> = None;
thread_local! {
    static CMD_QUEUE: std::cell::RefCell<Vec<DatabaseCommand>> = const {std::cell::RefCell::new(Vec::new())};
}
