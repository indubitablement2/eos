// use env_logger::Env;
use eos_common::connection_manager::*;
use eos_common::idx::*;
use eos_common::packet_common::*;
use std::time::Duration;

#[test]
fn send_receive_packet() {
    // let env = Env::default()
    //     .filter_or("LOG_LEVEL", "trace")
    //     .write_style_or("LOG_STYLE", "always");
    // env_logger::init_from_env(env);

    // Connect.
    let server_listener = std::net::TcpListener::bind("127.0.0.1:8484").expect("Can not bind.");
    let client_to_server = std::net::TcpStream::connect("127.0.0.1:8484").expect("Can not connect.");
    let server_to_client = server_listener.accept().expect("Can not accept.").0;

    // Make ThreadPool.
    let mut tp_server = PollingThread::new(false);
    let mut tp_client = PollingThread::new(false);

    // Make connection.
    let con_to_client = tp_server
        .connection_starter
        .create_connection(server_to_client, ClientId(1234), "127.0.0.0:1001".parse().unwrap())
        .unwrap();
    let con_to_server = tp_client
        .connection_starter
        .create_connection(client_to_server, ClientId(0), "127.0.0.0:1001".parse().unwrap())
        .unwrap();

    // Make packet.
    let client_hello = ClientLocalPacket::Broadcast {
        message: "client".to_string(),
    };
    let server_hello = ClientLocalPacket::Broadcast {
        message: "server".to_string(),
    };

    // Send packet to connection to be writen to socket.
    assert!(con_to_server.send_packet(client_hello.serialize()));
    assert!(con_to_client.send_packet(server_hello.serialize()));

    // Poll socket.
    tp_server.poll();
    std::thread::sleep(Duration::from_millis(10));
    tp_client.poll();
    std::thread::sleep(Duration::from_millis(10));
    // Recv packet from Connection.
    con_to_server.local_packets.read().iter().for_each(|packet| {
        assert_eq!(packet, &server_hello);
    });

    // Poll socket.
    tp_server.poll();
    std::thread::sleep(Duration::from_millis(10));
    // Recv packet from Connection.
    con_to_client.local_packets.read().iter().for_each(|packet| {
        assert_eq!(packet, &client_hello);
    });

    // Check num packets received is 1 for both.
    assert_eq!(con_to_server.local_packets.read().len(), 1);
    assert_eq!(con_to_client.local_packets.read().len(), 1);

    println!("Origin: {:?}", server_hello);
    println!("Serial: {:?}", con_to_server.local_packets.read().first().unwrap());
}
