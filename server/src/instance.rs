use super::*;
use battlescape::*;
use connection::*;
use database::*;
use runner::*;

pub fn _start() {
    let mut state = State::new();

    log::info!("Started instance server");

    let mut interval = interval::Interval::new(DT_MS, DT_MS * 4);
    loop {
        interval.step();

        if state.step() {
            log::warn!("Database disconnected");
            break;
        }
    }
}

struct State {
    database_connection: Connection,

    client_listener: ConnectionListener<ClientLogin>,
    logins: AHashMap<u64, Connection>,
    next_login_token: u64,

    battlescapes: IndexMap<BattlescapeId, Sender<BattlescapeHandleCmd>, RandomState>,
}
impl State {
    fn new() -> Self {
        Self {
            database_connection: connect_to_database(),
            client_listener: ConnectionListener::bind(instance_addr()),
            logins: Default::default(),
            next_login_token: 0,
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
                DatabaseResponse::ClientAuthResult {
                    client_id,
                    response_token,
                } => {
                    if let Some(connection) = self.logins.remove(&response_token) {
                        if let Some(client_id) = client_id {
                            if !self.battlescapes.is_empty() {
                                // Add to random battlescape.
                                let battlescape_idx =
                                    response_token as usize % self.battlescapes.len();
                                let (battlescape_id, sender) =
                                    self.battlescapes.get_index(battlescape_idx).unwrap();

                                connection.queue(ClientOutbound::LoggedIn {
                                    client_id,
                                    joined_battlescape_id: *battlescape_id,
                                });
                                connection.flush();
                                // TODO: Send.
                            }
                        }
                    }
                }
                DatabaseResponse::HandleBattlescape {
                    battlescape_id,
                    battlescape_misc_save,
                } => {
                    let battlescape_handle = BattlescapeRunner::start(
                        self.database_connection.outbound.clone(),
                        bincode_decode(&battlescape_misc_save).unwrap_or_else(|err| {
                            log::error!("Failed to decode battlescape save: {}", err);
                            Default::default()
                        }),
                        battlescape_id,
                    );
                    self.battlescapes.insert(battlescape_id, battlescape_handle);
                }
                DatabaseResponse::ClientShips {
                    client_id,
                    request,
                    client_ships,
                } => todo!(),
                DatabaseResponse::ShipEntered {
                    ship_id,
                    battlescape_id,
                    entity_save,
                    owner,
                } => todo!(),
            }
        }
        if database_disconnected {
            return true;
        }

        self.database_connection.flush();

        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientLogin {
    LoginUsernamePassword { username: String, password: String },
    RegisterUsernamePassword { username: String, password: String },
}
impl Packet for ClientLogin {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
    }
}

#[derive(Serialize)]
enum ClientOutbound {
    LoggedIn {
        client_id: ClientId,
        joined_battlescape_id: BattlescapeId,
    },
    RetryAt {
        addr: String,
    },
}
impl Packet for ClientOutbound {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(_buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
}
