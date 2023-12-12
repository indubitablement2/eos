use super::*;
use battlescape::*;
use connection::*;
use database::*;
use rayon::prelude::*;

/// How many tick between battlescape saves. (30 minutes)
const BATTLESCAPE_SAVE_INTERVAL: u64 = (1000 / DT_MS) * 60 * 30;

struct State {
    database_connection: Connection,

    client_listener: ConnectionListener<ClientLogin>,
    logins: AHashMap<u64, Connection>,
    next_login_token: u64,

    clients: AHashMap<ClientId, Client>,

    battlescapes: AHashMap<BattlescapeId, BattlescapeHandle>,
}
struct Client {
    connection: Connection,
}
struct BattlescapeHandle {
    tick_since_last_save: u64,
    battlescape: Battlescape,
    cmds: Vec<BattlescapeCommand>,
}
impl State {
    fn new() -> Self {
        Self {
            client_listener: ConnectionListener::bind(instance_addr()),
            database_connection: connect_to_database(),
            logins: Default::default(),
            next_login_token: 0,
            clients: Default::default(),
            battlescapes: Default::default(),
        }
    }

    /// Return if disconnected.
    fn step(&mut self) -> bool {
        // Get new client connections.
        while let Some((connection, login)) = self.client_listener.recv() {
            self.database_connection.queue(DatabaseRequest::ClientAuth {
                login,
                response_token: self.next_login_token,
            });
            self.logins.insert(self.next_login_token, connection);
            self.next_login_token += 1;
        }

        // Handle database responses.
        let mut database_disconnected = false;
        while let Some(response) = self
            .database_connection
            .recv_deferred::<DatabaseResponse>(&mut database_disconnected)
        {
            match response {
                DatabaseResponse::ClientAuth {
                    client_id,
                    response_token,
                } => {
                    if let Some(connection) = self.logins.remove(&response_token) {
                        if let Some(client_id) = client_id {
                            connection.queue(ClientOutbound::LoggedIn { client_id });
                            self.clients.insert(client_id, Client { connection });
                        }
                    }
                }
            }
        }
        if database_disconnected {
            return true;
        }

        // TODO: Handle in thread
        // // Handle client packets.
        // for (client_id, client) in self.clients.iter_mut() {
        //     while let Some(packet) = client.connection.recv::<ClientInbound>() {
        //         match packet {
        //             ClientInbound::Test => todo!(),
        //         }
        //     }
        // }

        // Step battlescapes.
        let num_battlescapes = self.battlescapes.len() as u64;
        self.battlescapes
            .par_iter_mut()
            .for_each(|(&battlescape_id, handle)| {
                handle.tick_since_last_save += 1;

                for cmd in handle.cmds.drain(..) {
                    handle.battlescape.apply_cmd(&cmd);
                }

                handle.battlescape.step();

                if handle.tick_since_last_save >= BATTLESCAPE_SAVE_INTERVAL
                    && battlescape_id.as_u64() % num_battlescapes == 0
                {
                    handle.tick_since_last_save = 0;

                    self.database_connection
                        .queue(DatabaseRequest::SaveBattlescape {
                            battlescape_id,
                            battlescape_misc_save: bincode_encode(&handle.battlescape.misc_save()),
                        });
                    // TODO: Save ships
                    // TODO: Save planets?
                }
            });

        self.database_connection.flush();

        // // Flush client connections.
        // self.clients.retain(|_, client| {
        //     client.connection.flush();
        //     client.connection.is_connected()
        // });

        false
    }
}

pub fn _start() {
    let mut state = State::new();

    log::info!("Started instance server");

    let mut interval = interval::Interval::new(DT_MS, DT_MS * 4);
    loop {
        interval.step();

        if state.step() {
            log::warn!("Database disconnected");
            state = State::new();
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ClientLogin {
    pub new_account: bool,
    pub username: String,
    pub password: String,
}
impl Packet for ClientLogin {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
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
    Hello,
    LoggedIn { client_id: ClientId },
}
impl Packet for ClientOutbound {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(_buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
}
