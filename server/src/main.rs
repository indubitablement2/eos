use common::connection::*;
use common::database_packet::*;
use common::ids::*;
use common::*;
use flume::Sender;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn main() {
    common::logger::Logger::init();
    common::load_data();

    let server_address = std::env::var("SERVER_ADDRESS").unwrap();
    let simulation_capacity = std::thread::available_parallelism().unwrap().get() as f32 * 16.0;

    log::info!(
        "Server starting:\n\tserver_address: {}\n\tsimulation_capacity: {}",
        server_address,
        simulation_capacity
    );

    let new_client = ConnectionListener::bind(&server_address).unwrap();

    // Connect to database.
    let database_connection = Connection::connect(format!("ws://{}", database_address())).unwrap();
    database_connection.queue(AuthRequest::Server {
        database_password: database_password(),
        server_address,
        simulation_capacity,
    });
    database_connection.flush();
    let _server_id = database_connection.block_recv::<ServerId>().unwrap();

    let mut simulations: HashMap<SimulationId, Simulation> = HashMap::new();
    let restart = Arc::new(AtomicBool::new(false));

    let mut sys = sysinfo::System::new();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let mut client_first_packet = Vec::new();

    log::info!("Server started");
    let mut interval = common::interval::Interval::new(100, 500);
    loop {
        interval.step();

        // Receive new clients.
        while let Some(connection) = new_client.try_recv() {
            client_first_packet.push((connection, 0u64));
        }

        // Handle client's first packet.
        client_first_packet.retain_mut(|(connection, counter)| {
            *counter += 1;
            if let Some((client_id, token, simulation_id)) =
                connection.try_recv::<(ClientId, u64, SimulationId)>()
            {
                if let Some(simulation) = simulations.get(&simulation_id) {
                    let _ =
                        simulation
                            .new_client_serder
                            .send((client_id, token, connection.clone()));
                }
                false
            } else {
                *counter < 100
            }
        });

        // Handle database packets.
        while let Some(response) = database_connection.try_recv::<ServerResponse>() {
            match response {
                ServerResponse::StartSimulation { simulation_id } => {
                    if simulations.contains_key(&simulation_id) {
                        log::error!("Simulation already started: {:?}", simulation_id);
                        continue;
                    }

                    let (database_response_serder, database_response_receiver) = flume::unbounded();
                    let (new_client_serder, new_client_receiver) = flume::unbounded();

                    let connection = SimulationConnection::new(
                        simulation_id,
                        database_connection.clone(),
                        database_response_receiver,
                        new_client_receiver,
                        restart.clone(),
                    );

                    let join_handle = std::thread::spawn(move || {
                        let mut sim = common::simulation::Simulation::new(connection);

                        log::info!("{:?} started", simulation_id);
                        let mut interval = common::interval::Interval::new(100, 500);
                        loop {
                            interval.step();
                            sim.step();
                        }
                    });

                    simulations.insert(
                        simulation_id,
                        Simulation {
                            database_response_serder,
                            new_client_serder,
                            join_handle,
                        },
                    );
                }
                ServerResponse::SimulationResponse {
                    simulation_id,
                    response,
                } => {
                    if let Some(simulation) = simulations.get(&simulation_id) {
                        let _ = simulation.database_response_serder.send(response);
                    } else {
                        log::warn!("Simulation not found: {:?}", simulation_id);
                    }
                }
                ServerResponse::Restart => {
                    log::info!("Restart started");

                    restart.store(true, std::sync::atomic::Ordering::Relaxed);

                    loop {
                        interval.step();
                        database_connection.flush();

                        if simulations
                            .values()
                            .all(|simulation| simulation.join_handle.is_finished())
                        {
                            for _ in 0..600 {
                                interval.step();
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
    new_client_serder: Sender<(ClientId, u64, Connection)>,
    join_handle: std::thread::JoinHandle<()>,
}
