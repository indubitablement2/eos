use super::*;
use futures_util::*;
use tokio::net::TcpStream;

pub struct Server {
    simulations: Vec<System>,
    rt: tokio::runtime::Runtime,
}
impl Server {
    pub fn load() -> Result<Self> {
        let rt = tokio::runtime::Runtime::new()?;

        rt.spawn(accept_loop());

        Ok(Self {
            simulations: vec![System::new()],
            rt,
        })
    }

    pub fn step(&mut self) {
        for simulation in &mut self.simulations {
            simulation.step();
        }
    }
}

async fn accept_loop() -> Result<()> {
    let addr = "127.0.0.1:2345";

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(new_connection(stream));
    }
}

async fn new_connection(stream: TcpStream) -> Result<()> {
    stream.set_nodelay(false)?;

    let addr = stream.peer_addr()?;
    debug!("New tcp connection established with: {}", addr);

    let stream = tokio_tungstenite::accept_async(stream).await?;
    debug!("New websocket connection established with: {}", addr);

    let (mut s, mut r) = stream.split();

    s.send(tokio_tungstenite::tungstenite::Message::Text(
        "Hello from the server!".to_string(),
    ))
    .await?;

    while let Some(msg) = r.next().await {
        info!("Received a message from websocket: {:?}", msg?.into_text());
    }

    Ok(())
}
