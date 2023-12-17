use super::*;
use battlescape::*;
use connection::*;
use database::*;

/// How many tick between battlescape saves. (30 minutes)
const BATTLESCAPE_SAVE_INTERVAL: u64 = (1000 / DT_MS) * 60 * 30;

pub enum BattlescapeHandleCmd {}

#[derive(Serialize, Deserialize, Default)]
pub struct ClientView {
    battlescape: BattlescapeId,
    ship: Option<ShipId>,
    translation: Vector2<f32>,
    zoom: f32,
}

pub struct BattlescapeRunner {
    database_outbound: ConnectionOutbound,
    cmd_receiver: Receiver<BattlescapeHandleCmd>,

    clients: AHashMap<ClientId, Connection>,

    tick_since_last_save: u64,
    battlescape_id: BattlescapeId,
    battlescape: Battlescape,
}
impl BattlescapeRunner {
    pub fn start(
        database_outbound: ConnectionOutbound,
        save: BattlescapeMiscSave,
        battlescape_id: BattlescapeId,
    ) -> Sender<BattlescapeHandleCmd> {
        let (cmd_sender, cmd_receiver) = unbounded();

        let mut runner = Self {
            database_outbound,
            cmd_receiver,
            clients: Default::default(),
            tick_since_last_save: 0,
            battlescape_id,
            battlescape: Battlescape::new(save),
        };

        std::thread::spawn(move || {
            let mut interval = interval::Interval::new(DT_MS, DT_MS * 4);
            loop {
                interval.step();
                runner.step();
            }
        });

        cmd_sender
    }

    fn step(&mut self) {
        loop {
            match self.cmd_receiver.try_recv() {
                Ok(cmd) => match cmd {},
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        // Handle client packets.
        self.clients.retain(
            |client_id, connection| match connection.recv::<ClientInbound>() {
                Ok(packet) => {
                    match packet {
                        ClientInbound::Test => {}
                    }
                    true
                }
                Err(TryRecvError::Empty) => true,
                Err(TryRecvError::Disconnected) => false,
            },
        );

        self.battlescape.step();

        if self.tick_since_last_save >= BATTLESCAPE_SAVE_INTERVAL
            && self.battlescape_id.as_u64() % self.battlescape.tick == 0
        {
            self.tick_since_last_save = 0;

            self.database_outbound
                .queue(DatabaseRequest::SaveBattlescape {
                    battlescape_id: self.battlescape_id,
                    battlescape_misc_save: bincode_encode(&self.battlescape.misc_save()),
                });
            // TODO: Save ships
            // TODO: Save planets?
        }
    }
}

#[derive(Deserialize)]
enum ClientInbound {
    Test,
}
impl Packet for ClientInbound {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
    }
}

#[derive(Serialize)]
enum ClientOutbound {
    ClientShips { ships: Vec<u8> },
}
impl Packet for ClientOutbound {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(_buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
}
