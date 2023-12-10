use super::*;
use battlescape::*;
use connection::*;
use database::*;
use rayon::prelude::*;

/// How many tick between battlescape saves. (30 minutes)
const BATTLESCAPE_SAVE_INTERVAL: u64 = (1000 / DT_MS) * 60 * 30;

struct State {
    database_connection: Connection,

    client_listener: ConnectionListener,
    logins: AHashMap<u64, Connection>,

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
        let database_connection = connect_to_database();

        Self {
            client_listener: ConnectionListener::bind(
                instance_addr(),
                ClientAuth {
                    database_connection: database_connection.connection_outbound.clone(),
                },
            ),
            database_connection,
            logins: Default::default(),
            clients: Default::default(),
            battlescapes: Default::default(),
        }
    }

    fn step(&mut self) {
        // Get new client connections.
        while let Some((connection, login_token)) = self.client_listener.recv() {
            self.logins.insert(login_token, connection);
        }

        // Handle database responses.
        while let Some(response) = self.database_connection.recv::<DatabaseResponse>() {
            match response {
                DatabaseResponse::ClientAuth {
                    client_id,
                    response_token,
                } => {
                    // if let Some(sender) = self.logins.remove(&login) {
                    //     let _ = sender.send(client_id);
                    // }
                }
            }
        }

        // Handle client packets.
        for (client_id, client) in self.clients.iter_mut() {
            while let Some(packet) = client.connection.recv::<ClientInbound>() {
                match packet {
                    ClientInbound::Test => todo!(),
                }
            }
        }

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

        // Flush client connections.
        self.clients.retain(|_, client| {
            client.connection.flush();
            client.connection.is_connected()
        });
    }
}

pub fn _start() {
    let mut state = State::new();

    log::info!("Started instance server");

    let mut interval = interval::Interval::new(DT_MS, DT_MS * 4);
    loop {
        interval.step();
        state.step();

        if state.database_connection.is_disconnected() {
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

#[derive(Clone)]
struct ClientAuth {
    database_connection: ConnectionOutbound,
}
impl Authentication for ClientAuth {
    async fn login_packet(&mut self) -> impl Packet {
        ClientOutbound::Hello
    }

    async fn verify_first_packet(&mut self, first_packet: Vec<u8>) -> anyhow::Result<u64> {
        let login: ClientLogin = bincode_decode(&first_packet)?;

        let (client_id_sender, mut client_id_receiver) = tokio::sync::mpsc::channel(1);

        self.client_auth_sender
            .send((login, client_id_sender))
            .context("stopped accepting new client")?;

        let client_id = client_id_receiver
            .recv()
            .await
            .context("channel dropped without answering")?
            .context("auth failed")?;

        Ok(client_id.as_u64())
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
