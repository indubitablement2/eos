use super::*;

#[derive(Serialize, Deserialize)]
pub enum DatabaseRequest {
    TestRequest,
    // TODO
    // Subscribe to new ship in battlescape
    // move ship to battlescape
    // notify client ship changes
    // unsubscibe
}
impl Packet for DatabaseRequest {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn parse(buf: Vec<u8>) -> Result<Self, &'static str> {
        bincode::deserialize(&buf).map_err(|_| "Bincode failed to deserialize")
    }
}

#[derive(Serialize, Deserialize)]
pub enum DatabaseResponse {
    TestResponse,
}
impl Packet for DatabaseResponse {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn parse(buf: Vec<u8>) -> Result<Self, &'static str> {
        bincode::deserialize(&buf).map_err(|_| "Bincode failed to deserialize")
    }
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseLogin {
    private_key: u64,
}
impl Packet for DatabaseLogin {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn parse(buf: Vec<u8>) -> Result<Self, &'static str> {
        bincode::deserialize(&buf).map_err(|_| "Bincode failed to deserialize")
    }
}

struct Database {
    battlescapes: AHashMap<BattlescapeId, ()>,

    next_client_id: ClientId,
    clients: AHashMap<ClientId, ()>,
    username: AHashMap<String, ClientId>,
    client_connection: AHashMap<ClientId, ()>,
}

#[derive(Default)]
struct State {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct InstanceId(u32);

struct Requests {
    instance_id: InstanceId,
    requests_ref: Vec<DatabaseRequestRef>,
    requests_mut: Vec<DatabaseRequestMut>,
    closed: bool,
}

pub fn _start(addr: SocketAddr) {
    let mut state = State::default();
    let mut database = load_database();

    let (requests_sender, requests_receiver) = sync_channel::<Requests>(256);
    let (responses_sender, responses_receiver) = channel::<(InstanceId, Vec<u8>)>();

    let mut listener = connection::ConnectionListener::bind(addr);
    std::thread::spawn(move || {
        let mut next_instance_id = InstanceId(0);
        let mut connections = AHashMap::new();

        let mut interval = interval::Interval::new(4);
        loop {
            interval.step();

            // Accept
            listener.accept(
                |login: DatabaseLogin| {
                    if login.private_key == PRIVATE_KEY {
                        Ok(())
                    } else {
                        Err("Invalid private key")
                    }
                },
                |connection| {
                    connections.insert(next_instance_id, connection);
                    next_instance_id = InstanceId(next_instance_id.0 + 1);
                },
            );

            // Recv + handle close
            connections.retain(|&instance_id, connection| {
                let mut requests = Requests {
                    instance_id,
                    requests_ref: Vec::new(),
                    requests_mut: Vec::new(),
                    closed: false,
                };

                while let Some(request) = connection.recv::<DatabaseRequest>() {
                    match request.split() {
                        Ok(r) => requests.requests_ref.push(r),
                        Err(m) => {
                            // TODO: Save to file.
                            requests.requests_mut.push(m);
                        }
                    }
                }

                let closed = connection.is_closed();
                requests.closed = closed;

                if !requests.requests_ref.is_empty() || closed {
                    let _ = requests_sender.send(requests);
                }

                !closed
            });

            // Queue
            for (instance_id, response) in responses_receiver.try_iter() {
                if let Some(connection) = connections.get_mut(&instance_id) {
                    connection.queue(response);
                }
            }

            // Flush
            for connection in connections.values_mut() {
                connection.flush();
            }
        }
    });

    // Database loop
    for requests in requests_receiver.iter() {
        for m in requests.requests_mut {
            if let Some(response) = m.process(requests.instance_id, &mut state, &mut database) {
                let _ = responses_sender.send((requests.instance_id, response.serialize()));
            }
        }

        for r in requests.requests_ref {
            if let Some(response) = r.process(requests.instance_id, &mut state, &database) {
                let _ = responses_sender.send((requests.instance_id, response.serialize()));
            }
        }

        if requests.closed {
            // TODO
        }
    }
}

impl DatabaseRequest {
    fn split(self) -> Result<DatabaseRequestRef, DatabaseRequestMut> {
        match self {
            DatabaseRequest::TestRequest => Ok(DatabaseRequestRef::TestRequest),
        }
    }
}

enum DatabaseRequestRef {
    TestRequest,
}
impl DatabaseRequestRef {
    fn process(
        self,
        instance_id: InstanceId,
        state: &mut State,
        database: &Database,
    ) -> Option<DatabaseResponse> {
        None
    }
}

#[derive(Serialize, Deserialize)]
enum DatabaseRequestMut {
    TestRequest,
}
impl DatabaseRequestMut {
    fn process(
        self,
        instance_id: InstanceId,
        state: &mut State,
        database: &mut Database,
    ) -> Option<DatabaseResponse> {
        None
    }
}

fn load_database() -> Database {
    // TODO: Load from file + parse
    todo!()
}
