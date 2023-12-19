use super::*;
use battlescape::client::Client;
use battlescape::*;
use connection::*;
use database::*;

pub fn _start() {
    let mut state = State::new();

    log::info!("Started instance server");

    let mut interval = interval::Interval::new(DT_MS, DT_MS);
    loop {
        interval.step();

        if state.step() {
            log::warn!("Database disconnected");
            break;
        }
    }
}

struct State {
    /// Only use in `step`.
    database_inbound: Option<ConnectionInbound>,
    database_outbound: ConnectionOutbound,

    client_listener: ConnectionListener<ClientLogin>,
    logins: AHashMap<u64, (Connection, BattlescapeId)>,
    next_login_token: u64,

    battlescapes: IndexMap<BattlescapeId, Sender<BattlescapeInbound>, RandomState>,
}
impl State {
    fn new() -> Self {
        let mut result = Err(anyhow::anyhow!("No suitable address found"));
        for (instance_id, instance_data) in data().instances.iter() {
            result = ConnectionListener::bind(instance_data.addr)
                .map(|listener| (*instance_id, listener));
            if result.is_ok() {
                break;
            }
        }
        let (instance_id, client_listener) = result.unwrap();
        log::info!("Bound as: {:?}", instance_id);

        let (database_outbound, database_inbound) = connect_to_database(instance_id).split();

        Self {
            database_inbound: Some(database_inbound),
            database_outbound,
            client_listener,
            logins: Default::default(),
            next_login_token: 0,
            battlescapes: Default::default(),
        }
    }

    /// Return if disconnected.
    fn step(&mut self) -> bool {
        // Get new client connections.
        while let Some((connection, login)) = self.client_listener.recv() {
            if !self.battlescapes.contains_key(&login.battlescape_id) {
                connection.close("Instance does not have requested battlescape");
                continue;
            }

            self.database_outbound.queue(DatabaseRequest::ClientAuth {
                login: login.login_type,
                response_token: self.next_login_token,
            });
            self.logins
                .insert(self.next_login_token, (connection, login.battlescape_id));
            self.next_login_token += 1;
        }

        self.database_outbound.flush();

        // Handle database responses.
        let mut database_inbound = self.database_inbound.take().unwrap();
        let disconencted = loop {
            match database_inbound.recv::<DatabaseResponse>() {
                Ok(response) => {
                    if let Err(err) = self.handle_database_response(response) {
                        log::warn!("Failed to handle database response: {}", err);
                    }
                }
                Err(TryRecvError::Empty) => break false,
                Err(TryRecvError::Disconnected) => break true,
            }
        };
        self.database_inbound = Some(database_inbound);

        disconencted
    }

    fn handle_database_response(&mut self, response: DatabaseResponse) -> anyhow::Result<()> {
        match response {
            DatabaseResponse::ClientAuthResult {
                client_id,
                response_token,
            } => {
                let (connection, battlescape_id) = self
                    .logins
                    .remove(&response_token)
                    .context("Client should be awaiting login")?;
                if let Some(client_id) = client_id {
                    let sender = self
                        .battlescapes
                        .get(&battlescape_id)
                        .context("Client's requested battlescape should be there")?;

                    connection.queue(ClientLoginSuccess {
                        client_id,
                        joined_battlescape_id: battlescape_id,
                    });
                    connection.flush();

                    sender.send(BattlescapeInbound::NewClient {
                        client_id,
                        client: Client::new(connection),
                    })?;
                } else {
                    connection.close("Failed to authenticate");
                }
            }
            DatabaseResponse::HandleBattlescape {
                battlescape_id,
                battlescape_misc_save,
                epoch,
            } => {
                let database_outbound = self.database_outbound.clone();
                let (battlescape_outbound, battlescape_inbound) = unbounded();
                let save = bin_decode(&battlescape_misc_save)?;

                self.battlescapes
                    .insert(battlescape_id, battlescape_outbound);

                std::thread::spawn(move || {
                    battlescape_loop(Battlescape::new(
                        battlescape_id,
                        epoch,
                        database_outbound,
                        battlescape_inbound,
                        save,
                    ));
                });
            }
            DatabaseResponse::DatabaseBattlescapeResponse { from, response } => {
                let sender = self
                    .battlescapes
                    .get(&from)
                    .context("Battlescape should be there")?;
                sender.send(BattlescapeInbound::DatabaseBattlescapeResponse(response))?;
            }
        }

        Ok(())
    }
}

fn battlescape_loop(mut battlescape: Battlescape) {
    let mut interval = interval::Interval::new(DT_MS, DT_MS * 8);
    loop {
        interval.step();
        battlescape.step();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientLogin {
    battlescape_id: BattlescapeId,
    login_type: ClientLoginType,
}
impl Packet for ClientLogin {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bin_decode(&buf)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientLoginType {
    LoginUsernamePassword { username: String, password: String },
    RegisterUsernamePassword { username: String, password: String },
}

#[derive(Serialize)]
struct ClientLoginSuccess {
    client_id: ClientId,
    joined_battlescape_id: BattlescapeId,
}
impl Packet for ClientLoginSuccess {
    fn serialize(self) -> Vec<u8> {
        bin_encode(self)
    }

    fn parse(_buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
}
