use eos_common::connection_manager::*;
use eos_common::idx::*;
use eos_common::location::Location;
use eos_common::packet_common::*;
use glam::vec2;
use std::time::Duration;

#[test]
fn all_packets() {
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

    let mut packet_to_test = Vec::new();

    // ! Make change here to add/remove packet to test.
    packet_to_test.push(ClientLocalPacket::Broadcast {
        message: "client".to_string(),
    });
    packet_to_test.push(ClientLocalPacket::ClientFleetWishLocation {
        fleet_id: FleetId(255),
        location: Location {
            sector_id: SectorId(10),
            local_position: vec2(100.1, 12345.123),
        },
    });
    // !

    // Send packets to connection to be writen to socket.
    packet_to_test.iter().for_each(|packet| {
        assert!(con_to_server.send_packet(packet.serialize()));
    });

    // Poll socket.
    tp_client.poll();
    std::thread::sleep(Duration::from_millis(10));
    tp_server.poll();
    std::thread::sleep(Duration::from_millis(10));

    // Recv packet from Connection.
    con_to_client.local_packets.read().iter().for_each(|packet| {
        assert_ne!(packet, &ClientLocalPacket::Invalid);

        let mut result = false;
        for packet_test in packet_to_test.iter() {
            if packet_test == packet {
                println!("Origin: {:?}", packet_test);
                println!("Serial: {:?}", packet);
                let serialized = packet_test.serialize();
                println!("{} {:?}", &serialized.0.len(), &serialized);

                result = true;
                break;
            }
        }
        assert!(result);
    });

    assert_eq!(packet_to_test.len(), con_to_client.local_packets.read().len());
}
