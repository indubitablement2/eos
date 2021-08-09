use eos_common::const_var::SERVER_ADDR;
use eos_common::idx::ClientId;
use eos_common::packet_common::*;
use eos_common::*;
use std::net::TcpStream;

fn main() {
    println!("Connecting...");
    let std_socket = TcpStream::connect(SERVER_ADDR).unwrap();

    let pt = connection_manager::PollingThread::new(false);

    let socket = pt.connection_starter.convert_std_to_tokio(std_socket).unwrap();
    let connection = pt.connection_starter.create_connection(socket);

    println!("Sending client hello...");
    assert!(connection.send_packet(
        OtherPacket::ClientLogin {
            app_version: const_var::APP_VERSION,
            steam_id: ClientId(123),
            ticket: String::new(),
        }
        .serialize()
    ));

    println!("Reading from server...");
    while let Ok(packet) = connection.other_packet_receiver.recv() {
        println!("{:?}", &packet);
    }
    println!("Lost connection to server.");
}
