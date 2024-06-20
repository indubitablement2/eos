use redis::{aio::MultiplexedConnection, AsyncCommands};

use super::*;

pub struct SimulationConnection {
    simulation_id: u64,
    redis_connection: MultiplexedConnection,
}

pub struct Server {
    redis: redis::Client,
    redis_connection: MultiplexedConnection,
    simulations: HashMap<u64, simulation::Simulation>,
}
impl Server {
    pub async fn new() -> Self {
        let subdomain = std::env::var("SERVER_SUBDOMAIN").unwrap();
        // let addr = format!("{}.my_domain.com:11192", subdomain);
        let addr = format!("{}:11192", subdomain);
        log::info!("Server address {}", addr);

        let new_client = connection::ConnectionListener::bind(addr).unwrap();

        let redis = redis::Client::open(std::env::var("DATABASE_ADDR").unwrap()).unwrap();
        let mut redis_connection = redis.get_multiplexed_tokio_connection().await.unwrap();
        // let ret: Option<bool> = con.get("foo").unwrap();
        // println!("{:?}", ret);

        Self {
            redis,
            redis_connection,
            simulations: Default::default(),
        }
    }

    pub async fn start(self) {
        log::info!("Server started");
    }
}
