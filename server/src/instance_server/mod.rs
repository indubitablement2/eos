mod simulation_runner;

use super::*;
use battlescape::*;
use connection::*;
use database::*;
use simulation_runner::*;

pub fn _start() {
    let mut database_connection = connect_to_database();

    let mut client_auth_manager = ClientAuthManager::new();
    let mut clients: AHashMap<ClientId, Client> = Default::default();

    let mut battlescape_handles: AHashMap<BattlescapeId, BattlescapeRunnerHandle> =
        Default::default();

    log::info!("Started instance server");

    let mut interval = interval::Interval::new(DT_MS, DT_MS * 4);
    loop {
        interval.step();

        client_auth_manager.update(&database_connection);

        // Get new client connections.
        while let Some((connection, id)) = client_auth_manager.client_listener.recv() {
            let client_id = ClientId::from_u64(id).unwrap();
            connection.queue(ClientOutbound::LoggedIn { client_id });
            clients.insert(client_id, Client { connection });
        }

        // Handle database responses.
        while let Some(response) = database_connection.recv::<DatabaseResponse>() {
            match response {
                DatabaseResponse::ClientAuth { login, client_id } => {
                    client_auth_manager.handle_client_auth_response(login, client_id);
                }
            }
        }

        // Handle client packets.
        for (client_id, client) in clients.iter_mut() {
            while let Some(packet) = client.connection.recv::<ClientInbound>() {
                match packet {
                    ClientInbound::Test => todo!(),
                }
            }
        }

        // Step battlescapes.
        for handle in battlescape_handles.values_mut() {
            // TODO: Handle overloaded runner. Use request_sender's len.
            let _ = handle.request_sender.send(BattlescapeRunnerRequest::Step);

            // Handle battlescape responses.
            for battlescape_response in handle.response_receiver.try_iter() {
                match battlescape_response {}
            }
        }

        // Flush client connections.
        clients.retain(|_, client| {
            client.connection.flush();
            client.connection.is_connected()
        });

        // Flush database connection.
        database_connection.flush();
        if database_connection.is_disconnected() {
            log::error!("Database disconnected");
            break;
        }
    }
}

struct ClientAuthManager {
    client_listener: ConnectionListener,
    client_auth_receiver: Receiver<(ClientLogin, tokio::sync::mpsc::Sender<Option<ClientId>>)>,
    logins: AHashMap<ClientLogin, tokio::sync::mpsc::Sender<Option<ClientId>>>,
}
impl ClientAuthManager {
    fn new() -> Self {
        let (client_auth_sender, client_auth_receiver) =
            unbounded::<(ClientLogin, tokio::sync::mpsc::Sender<Option<ClientId>>)>();

        Self {
            client_listener: ConnectionListener::bind(
                instance_addr(),
                ClientAuth { client_auth_sender },
            ),
            client_auth_receiver,
            logins: Default::default(),
        }
    }

    fn handle_client_auth_response(&mut self, login: ClientLogin, client_id: Option<ClientId>) {
        if let Some(sender) = self.logins.remove(&login) {
            let _ = sender.send(client_id);
        }
    }

    fn update(&mut self, database_connection: &Connection) {
        // Handle client auth request.
        for (login, sender) in self.client_auth_receiver.try_iter() {
            database_connection.queue(DatabaseRequest::Ref(DatabaseRequestRef::ClientAuth {
                login: login.clone(),
            }));

            self.logins.insert(login, sender);
        }
    }
}

struct Client {
    connection: Connection,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct ClientLogin {
    pub username: String,
    pub password: String,
    // TODO: Create new account
    pub new_account: bool,
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
