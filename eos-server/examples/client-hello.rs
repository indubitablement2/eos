use eos_common::const_var::SERVER_ADDR;
use eos_common::idx::ClientId;
use eos_common::packet_common::*;
use eos_common::*;
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    println!("Connecting...");
    let socket = TcpStream::connect(SERVER_ADDR).unwrap();
    println!("Connected");

    let mut pt = connection_manager::PollingThread::new(false);

    let connection = pt
        .connection_starter
        .create_connection(socket, ClientId(0), "127.0.0.1:1234".parse().unwrap())
        .unwrap();

    println!("Sending client hello...");
    // assert!(connection.send_packet(
    //     ClientLoginPacket::Hello {
    //         username: "client".to_string(),
    //         app_version: const_var::APP_VERSION
    //     }
    //     .serialize()
    // ));

    println!("Reading from server.");
    loop {
        pt.poll();
        if connection.is_disconnected() {
            println!("Lost connection to server.");
            break;
        }
        connection.local_packets.read().iter().for_each(|packet| {
            println!("{:?}", packet);
        });
        std::thread::sleep(Duration::from_millis(100));
    }
}
