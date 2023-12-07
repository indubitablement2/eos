use super::*;
use battlescape::*;
use connection::*;
use database::*;
use rayon::prelude::*;

struct State {
    database_connection: Connection,

    client_listener: ConnectionListener,
    client_auth_receiver: Receiver<(ClientLogin, tokio::sync::mpsc::Sender<Option<ClientId>>)>,
    logins: AHashMap<ClientLogin, tokio::sync::mpsc::Sender<Option<ClientId>>>,

    clients: AHashMap<ClientId, Client>,

    battlescapes: AHashMap<BattlescapeId, Battlescape>,
}
impl State {
    fn new() -> Self {
        let (client_auth_sender, client_auth_receiver) =
            unbounded::<(ClientLogin, tokio::sync::mpsc::Sender<Option<ClientId>>)>();

        Self {
            database_connection: connect_to_database(),

            client_listener: ConnectionListener::bind(
                instance_addr(),
                ClientAuth { client_auth_sender },
            ),
            client_auth_receiver,
            logins: Default::default(),

            clients: Default::default(),

            battlescapes: Default::default(),
        }
    }

    fn step(&mut self) {
        // Handle client auth request.
        for (login, sender) in self.client_auth_receiver.try_iter() {
            let request = if login.new_account {
                DatabaseRequest::Mut(DatabaseRequestMut::NewClient {
                    login: login.clone(),
                })
            } else {
                DatabaseRequest::Ref(DatabaseRequestRef::ClientAuth {
                    login: login.clone(),
                })
            };

            self.database_connection.queue(request);
            self.logins.insert(login, sender);
        }

        // Get new client connections.
        while let Some((connection, id)) = self.client_listener.recv() {
            let client_id = ClientId::from_u64(id).unwrap();
            connection.queue(ClientOutbound::LoggedIn { client_id });
            self.clients.insert(client_id, Client { connection });
        }

        // Handle database responses.
        while let Some(response) = self.database_connection.recv::<DatabaseResponse>() {
            match response {
                DatabaseResponse::ClientAuth { login, client_id } => {
                    if let Some(sender) = self.logins.remove(&login) {
                        let _ = sender.send(client_id);
                    }
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
        self.battlescapes
            .par_iter_mut()
            .for_each(|(_, battlescape)| {
                // TODO: Cmds
                battlescape.step();
            });

        self.database_connection.flush();

        // Flush client connections.
        self.clients.retain(|_, client| {
            client.connection.flush();
            client.connection.is_connected()
        });
    }
}

struct Client {
    connection: Connection,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct ClientLogin {
    pub new_account: bool,
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
struct ClientAuth {
    client_auth_sender: Sender<(ClientLogin, tokio::sync::mpsc::Sender<Option<ClientId>>)>,
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
