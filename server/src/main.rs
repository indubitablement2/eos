use std::sync::atomic::AtomicI64;
use std::time::Duration;

use common::database_packet::{AuthRequest, ServerRequest};
use common::ids::ServerId;
use common::{connection::*, system::SystemId};

static STAND_BY_RUNNER: AtomicI64 = AtomicI64::new(0);

fn main() {
    common::logger::Logger::init();
    common::load_data();

    let database_connection = Connection::connect(common::DATABASE_ADDRESS).unwrap();
    database_connection.queue(AuthRequest::Server {
        password: std::env::var("DATABASE_PASSWORD").unwrap(),
    });
    database_connection.flush();
    let server_id = database_connection
        .block_recv::<common::ids::ServerId>()
        .unwrap();

    let mut sys = sysinfo::System::new();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    loop {
        std::thread::sleep(Duration::from_secs(5));

        // TODO: send stats to database
        // sys.refresh_cpu_usage();
        // sys.refresh_memory();
        // sys.global_cpu_info().cpu_usage()

        if STAND_BY_RUNNER.load(std::sync::atomic::Ordering::Relaxed) < 10 {
            start_runner(server_id);
        }
    }
}

fn start_runner(server_id: ServerId) {
    STAND_BY_RUNNER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    std::thread::spawn(move || {
        let database_connection = Connection::connect(common::DATABASE_ADDRESS).unwrap();
        let new_client = ConnectionListener::bind("127.0.0.1:0").unwrap();

        database_connection.queue(AuthRequest::Simulation {
            password: std::env::var("DATABASE_PASSWORD").unwrap(),
            server_id,
            client_address: new_client.local_addr(),
        });

        let (system_id, save) = database_connection
            .block_recv::<(SystemId, Option<Vec<u8>>)>()
            .unwrap();

        let mut sim = common::simulation::Simulation::new(
            database_connection,
            new_client,
            system_id,
            save.as_deref(),
        );

        let mut interval = common::interval::Interval::new(100, 500);
        loop {
            interval.step();
            sim.step();
        }
    });
}
