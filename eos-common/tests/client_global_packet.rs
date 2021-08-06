use eos_common::connection_manager::*;
use eos_common::packet_common::*;
use std::time::Duration;

#[test]
fn all_packets() {
    // Connect.
    let server_listener = std::net::TcpListener::bind("127.0.0.1:8484").expect("Can not bind.");
    let client_to_server = std::net::TcpStream::connect("127.0.0.1:8484").expect("Can not connect.");
    let server_to_client = server_listener.accept().expect("Can not accept.").0;

    // Make ThreadPool.
    let tp_server = PollingThread::new(false);
    let tp_client = PollingThread::new(false);

    // Make connection.
    let con_to_client = tp_server.connection_starter.create_connection(server_to_client).unwrap();
    let con_to_server = tp_client.connection_starter.create_connection(client_to_server).unwrap();

    let mut packet_to_test = Vec::new();

    // ! Make change here to add/remove packet to test.
    packet_to_test.push(ClientGlobalPacket::Broadcast {
        message: "client".to_string(),
    });
    // packet_to_test.push(ClientGlobalPacket:: ClientFleetWishLocation {
    //     fleet_id: FleetId(255),
    //     location: Location {
    //         sector_id: SectorId(10),
    //         local_position: vec2(100.1, 12345.123),
    //     },
    // });
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
    con_to_client.client_global_packets_receiver.try_iter().for_each(|packet| {
        assert_ne!(packet, ClientGlobalPacket::Invalid);

        let mut result = false;
        for packet_test in packet_to_test.iter() {
            if packet_test == &packet {
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
}
