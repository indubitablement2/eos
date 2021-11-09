use std::{
    net::{Ipv6Addr, SocketAddrV6},
    thread::sleep,
    time::Duration,
};
use tokio::{
    io::*,
    net::{TcpListener, TcpStream},
    spawn,
};

#[tokio::main]
async fn main() {
    let server_listener = TcpListener::bind(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0))
        .await
        .unwrap();

    let mut client = BufStream::new(TcpStream::connect(server_listener.local_addr().unwrap()).await.unwrap());

    let mut server = BufStream::new(server_listener.accept().await.unwrap().0);

    spawn(async move {
        loop {
            let mut buf = [0u8; 10];
            let r_num = client.read(&mut buf).await.unwrap();
            if r_num == 0 {
                println!("done reading");
                break;
            }
            println!("Received from server: {:?}", &buf[..r_num]);
        }
    });

    for _ in 0..5 {
        let w_num = server.write(&[77]).await.unwrap();
        server.flush().await.unwrap();
        println!("server sent {} bytes", w_num);
        sleep(Duration::from_millis(200));
    }

    sleep(Duration::from_millis(200));
}
