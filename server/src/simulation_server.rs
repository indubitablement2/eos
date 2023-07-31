use super::*;
use std::net::SocketAddr;
use tokio::{net::TcpStream, spawn};

pub struct Server {
    simulations: Vec<System>,
}
impl Server {
    pub async fn load() -> Result<Self> {
        // Connect to master server.
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;

            // todo: connect to master server.
            break;

            // match master_server::connect().await {
            //     Ok(_) => break,
            //     Err(err) => {
            //         error!("Failed to connect to master server: {}", err);
            //     }
            // }
        }

        spawn(accept_loop());

        Ok(Self {
            simulations: vec![System::new()],
        })
    }

    pub fn step(&mut self) {
        for simulation in &mut self.simulations {
            simulation.step();
        }
    }
}

async fn accept_loop() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("[::]:0").await?;
    info!("Listening on: {}", listener.local_addr()?);

    loop {
        let (stream, addr) = listener.accept().await?;
        // use tokio_util::sync::
        tokio::spawn(new_connection(stream, addr));
    }
}

async fn new_connection(stream: TcpStream, addr: SocketAddr) -> Result<()> {
    match client_connection::init_client_connection(stream, addr).await {
        Ok(mut client_connection) => loop {
            // Send connection to simulation server.
        },
        Err(err) => {
            error!("Failed to initialize client connection: {}", err);
        }
    }

    Ok(())
}
