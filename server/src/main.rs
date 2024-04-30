use common::{connection::*, system::SystemId};

fn main() {
    common::logger::Logger::init();
    common::load_data();

    let database_connection = Connection::connect(common::DATABASE_ADDRESS).unwrap();

    let mut interval = common::interval::Interval::new(1000, 10000);
    loop {
        interval.step();
    }
}

struct SimulationRunner {}
impl SimulationRunner {
    fn start() {
        std::thread::spawn(move || {
            let database_connection = Connection::connect(common::DATABASE_ADDRESS).unwrap();
            let new_client = ConnectionListener::bind("127.0.0.1:0").unwrap();

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
}
