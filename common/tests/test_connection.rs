// use common::reliable_udp::connection::*;
// use common::reliable_udp::inbound_loop::*;
// use common::reliable_udp::*;
// use std::net::UdpSocket;
// use std::sync::atomic::AtomicBool;
// use std::sync::Arc;

// /// Create 2 connected peers.
// fn create_test_connections_pair() -> (Connection, Connection) {
//     let (s, c) = (
//         Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap()),
//         Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap()),
//     );

//     let configs = Arc::new(ConnectionConfigs::default());

//     let (sc, s_chan) = Connection::new(c.local_addr().unwrap(), s.clone(), configs.clone());
//     let (cc, c_chan) = Connection::new(s.local_addr().unwrap(), c.clone(), configs);

//     let s_ci = ConnectionsInternals::default();
//     s_ci.connections.write().unwrap().insert(sc.address(), s_chan);
//     let c_ci = ConnectionsInternals::default();
//     c_ci.connections.write().unwrap().insert(cc.address(), c_chan);

//     std::thread::spawn(move || inbound_loop(s, s_ci, Arc::new(AtomicBool::new(false))));
//     std::thread::spawn(move || inbound_loop(c, c_ci, Arc::new(AtomicBool::new(false))));

//     (sc, cc)
// }

// fn sleep_milli() {
//     std::thread::sleep(std::time::Duration::from_millis(1));
// }

// fn print_stats(sc: &Connection, cc: &Connection) {
//     println!("sc: {:#?}", sc.stats);
//     println!("cc: {:#?}", cc.stats);
// }

// /// Send and receive an empty packet.
// #[test]
// fn test_reliable_udp() {
//     let (mut sc, mut cc) = create_test_connections_pair();

//     sc.send(&[], false);
//     sleep_milli();
//     assert!(cc.recv().unwrap().slice().is_empty());

//     cc.send(&[], false);
//     sleep_milli();
//     assert!(sc.recv().unwrap().slice().is_empty());

//     print_stats(&sc, &cc);
// }

// /// Send and receive random bytes.
// /// Check that outboud == inbound.
// #[test]
// fn test_non_corrupt() {
//     use rand::Rng;
//     let mut rng = rand::thread_rng();

//     let (mut sc, mut cc) = create_test_connections_pair();

//     for _ in 0..16 {
//         let mut packets = Vec::with_capacity(u8::MAX as usize);
//         let mut recved = 0;
        
//         for i in 0..u8::MAX {
//             let mut r: Vec<u8> = (0..rng.gen::<usize>() % (MAX_PAYLOAD_SIZE - 1) + 1)
//                 .into_iter()
//                 .map(|_| rng.gen::<u8>())
//                 .collect();
//             r[0] = i;

//             if sc.is_bandwidth_saturated() {
//                 sleep_milli();

//                 while let Ok(payload) = cc.recv() {
//                     recved += 1;
//                     let i = payload.slice().first().unwrap().to_owned() as usize;
//                     assert_eq!(&packets[i], payload.slice());
//                     cc.send(&[], false);
//                     assert!(!cc.is_bandwidth_saturated());
//                 }

//                 sleep_milli();

//                 while sc.recv().is_ok() {
                    
//                 }
//                 sc.update(1.0).unwrap();
//             }

//             sc.send(&r, true);
//             packets.push(r);
//         }

//         sleep_milli();
        
//         while let Ok(payload) = cc.recv() {
//             recved += 1;
//             let i = payload.slice().first().unwrap().to_owned() as usize;
//             assert_eq!(&packets[i], payload.slice());
//             cc.send(&[], false);
//             assert!(!cc.is_bandwidth_saturated());
//         }

//         sleep_milli();

//         while sc.recv().is_ok() {
                    
//         }

//         assert_eq!(recved, u8::MAX);
//     }

//     assert!(sc.stats.compare(cc.stats));
//     print_stats(&sc, &cc);
// }
