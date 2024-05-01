use common::connection::*;
use common::database_packet::*;
use common::ids::ClientId;
use common::server::ServerId;
use flume::Sender;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    common::logger::Logger::init();
    common::load_data();

    let ws_addr = std::env::var("SERVER_IP_ADDR").unwrap();
    log::info!("Server address {}", ws_addr);

    let server_id = ServerId(
        ServerId::data()
            .into_iter()
            .find(|x| x.ws_addr == ws_addr)
            .unwrap(),
    );

    let new_client = ConnectionListener::bind(ws_addr).unwrap();

    // Connect to database.
    let database_connection = Connection::connect(common::DATABASE_ADDRESS).unwrap();
    database_connection.queue(ServerAuthRequest {
        password: std::env::var("DATABASE_PASSWORD").unwrap(),
        server_id,
    });
    database_connection.flush();
    let response = database_connection
        .block_recv::<ServerAuthResponse>()
        .unwrap();

    let mut simulations = HashMap::new();
    let restart = Arc::new(AtomicBool::new(false));

    // Start simulations.
    for (system_id, save) in response.system_saves {
        let (database_response_serder, database_response_receiver) = flume::unbounded();
        let (new_client_serder, new_client_receiver) = flume::unbounded();

        let connection = SimulationConnection::new(
            system_id,
            database_connection.clone(),
            database_response_receiver,
            new_client_receiver,
            restart.clone(),
        );

        let join_handle = std::thread::spawn(move || {
            let mut sim = common::simulation::Simulation::new(connection, save.as_deref());

            let mut interval = common::interval::Interval::new(100, 500);
            loop {
                interval.step();
                sim.step();
            }
        });

        simulations.insert(
            system_id,
            Simulation {
                database_response_serder,
                new_client_serder,
                join_handle,
            },
        );
    }

    let mut sys = sysinfo::System::new();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let mut client_first_packet = Vec::new();

    let mut next_client_auth_token = 0;
    let mut client_auth = HashMap::new();

    let mut interval = common::interval::Interval::new(10, 50);
    loop {
        interval.step();

        // Receive new clients
        while let Some(connection) = new_client.try_recv() {
            client_first_packet.push((connection, 0u32));
        }

        // Auth clients
        client_first_packet.retain_mut(|(connection, counter)| {
            *counter += 1;
            if *counter > 1000 {
                false
            } else if let Some(request) = connection.try_recv::<ClientLogin>() {
                let token = next_client_auth_token;
                next_client_auth_token += 1;

                client_auth.insert(token, connection.clone());
                database_connection.queue(ServerRequest::ClientLogin { request, token });

                false
            } else {
                true
            }
        });

        while let Some(response) = database_connection.try_recv::<ServerResponse>() {
            match response {
                ServerResponse::ClientLogin { token, result } => {
                    let Some(connection) = client_auth.remove(&token) else {
                        log::warn!("Invalid token {}", token);
                        continue;
                    };
                    let Some((client_id, system_id)) = result else {
                        continue;
                    };
                    let Some(simulation) = simulations.get(&system_id) else {
                        log::warn!("System not run by this server");
                        continue;
                    };

                    connection.queue(client_id);
                    if let Err(err) = simulation.new_client_serder.send((client_id, connection)) {
                        log::error!("Failed to send new client to simulation: {}", err);
                    };
                }
                ServerResponse::SimulationResponse {
                    system_id,
                    response,
                } => {
                    let _ = simulations[&system_id]
                        .database_response_serder
                        .send(response);
                }
                ServerResponse::Restart => {
                    log::info!("Restart started");

                    restart.store(true, std::sync::atomic::Ordering::Relaxed);

                    loop {
                        std::thread::sleep(Duration::from_millis(100));
                        database_connection.flush();

                        if simulations
                            .values()
                            .all(|simulation| simulation.join_handle.is_finished())
                        {
                            for _ in 0..1000 {
                                std::thread::sleep(Duration::from_millis(100));
                                database_connection.flush();
                            }
                            return;
                        }
                    }
                }
            }
        }

        // TODO: send stats to database
        // sys.refresh_cpu_usage();
        // sys.refresh_memory();
        // sys.global_cpu_info().cpu_usage()

        database_connection.flush();

        if database_connection.is_closed() {
            log::error!("Database connection closed");
            break;
        }
    }
}

struct Simulation {
    database_response_serder: Sender<SimulationResponse>,
    new_client_serder: Sender<(ClientId, Connection)>,
    join_handle: std::thread::JoinHandle<()>,
}
