use super::*;
use connection::*;
use database::*;
use simulation::client::Client;
use simulation::*;

pub fn _start() {
    let mut state = State::new();

    log::info!("Started instance server");

    let mut interval = interval::Interval::new(DT_MS, DT_MS);
    loop {
        interval.step();

        if state.step() {
            log::warn!("Database disconnected. Restarting instance server");
            break;
        }
    }
}

struct State {
    /// Only use in `step`.
    database_inbound: Option<ConnectionInbound>,
    database_outbound: ConnectionOutbound,

    client_listener: ConnectionListener<ClientLogin>,
    logins: AHashMap<u64, (Connection, SimulationId)>,
    next_login_token: u64,

    simulations: IndexMap<SimulationId, Sender<SimulationInbound>, RandomState>,
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
            simulations: Default::default(),
        }
    }

    /// Return if disconnected.
    fn step(&mut self) -> bool {
        // Get new client connections.
        while let Some((connection, login)) = self.client_listener.recv() {
            if !self.simulations.contains_key(&login.simulation_id) {
                connection.close("Instance does not have requested simulation");
                continue;
            }

            self.database_outbound.queue(DatabaseRequest::ClientAuth {
                login: login.login_type,
                response_token: self.next_login_token,
            });
            self.logins
                .insert(self.next_login_token, (connection, login.simulation_id));
            self.next_login_token += 1;
        }

        self.database_outbound.flush();

        // Handle database responses.
        let mut database_inbound = self.database_inbound.take().unwrap();
        let disconnected = loop {
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

        disconnected
    }

    fn handle_database_response(&mut self, response: DatabaseResponse) -> anyhow::Result<()> {
        match response {
            DatabaseResponse::ClientAuthResult {
                client_id,
                response_token,
            } => {
                let (connection, simulation_id) = self
                    .logins
                    .remove(&response_token)
                    .context("Login should be there")?;

                if let Some(client_id) = client_id {
                    let sender = self
                        .simulations
                        .get(&simulation_id)
                        .context("Client's requested simulation should be there")?;

                    sender.send(SimulationInbound::NewClient {
                        client_id,
                        client: Client::new(connection),
                    })?;
                } else {
                    connection.close("Failed to authenticate");
                }
            }
            DatabaseResponse::HandleSimulation {
                simulation_id,
                simulation_save,
            } => {
                let database_outbound = self.database_outbound.clone();
                let (simulation_outbound, simulation_inbound) = unbounded();

                self.simulations.insert(simulation_id, simulation_outbound);

                std::thread::spawn(move || {
                    simulation_loop(Simulation::new(
                        simulation_id,
                        database_outbound,
                        simulation_inbound,
                        simulation_save,
                    ));
                });
            }
            DatabaseResponse::SaveAllSystems => {
                for sender in self.simulations.values() {
                    let _ = sender.send(SimulationInbound::SaveRequest);
                }
            }
            DatabaseResponse::DatabaseSimulationResponse { to, response } => {
                let sender = self
                    .simulations
                    .get(&to)
                    .context("Simulation should be there")?;
                sender.send(SimulationInbound::DatabaseSimulationResponse(response))?;
            }
        }

        Ok(())
    }
}

fn simulation_loop(mut simulation: Simulation) {
    let mut interval = interval::Interval::new(DT_MS, DT_MS * 8);
    loop {
        interval.step();
        simulation.step();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientLogin {
    simulation_id: SimulationId,
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
