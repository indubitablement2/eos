use common::reliable_udp::connection::*;
use common::reliable_udp::inbound_loop::*;
use common::reliable_udp::packets::*;
use common::reliable_udp::*;
use std::net::UdpSocket;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Create 2 connected peers.
fn create_test_connections_pair() -> (Connection, Connection) {
    let (s, c) = (
        Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap()),
        Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap()),
    );

    let configs = Arc::new(ConnectionConfigs::default());

    let (sc, s_chan) = Connection::new(c.local_addr().unwrap(), s.clone(), configs.clone());
    let (cc, c_chan) = Connection::new(s.local_addr().unwrap(), s.clone(), configs);

    let s_ci = ConnectionsInternals::default();
    s_ci.connections.write().unwrap().insert(sc.address(), s_chan);
    let c_ci = ConnectionsInternals::default();
    c_ci.connections.write().unwrap().insert(cc.address(), c_chan);

    std::thread::spawn(move || inbound_loop(s, s_ci, Arc::new(AtomicBool::new(false))));
    std::thread::spawn(move || inbound_loop(c, c_ci, Arc::new(AtomicBool::new(false))));

    (sc, cc)
}

fn sleep_milli() {
    std::thread::sleep(std::time::Duration::from_millis(1));
}

fn print_stats(sc: &Connection, cc: &Connection) {
    println!("{:#?}", sc.stats);
    println!("{:#?}", cc.stats);
}

/// Send and receive an empty packet.
#[test]
fn test_reliable_udp() {
    let (mut sc, mut cc) = create_test_connections_pair();

    sc.send(&[], false);
    sleep_milli();
    assert!(cc.recv().unwrap().get_slice().is_empty());

    print_stats(&sc, &cc);
}

/// Send and receive random bytes.
/// Check that outboud == inbound.
#[test]
fn test_non_corrupt() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let (mut sc, mut cc) = create_test_connections_pair();

    for _ in 0..100 {
        let mut packets = Vec::with_capacity(u8::MAX as usize);
        for i in 0..u8::MAX {
            let mut r: Vec<u8> = (0..rng.gen::<usize>() % MAX_PAYLOAD_SIZE)
                .into_iter()
                .map(|_| rng.gen::<u8>())
                .collect();

            if r.is_empty() {
                r.push(i);
            } else {
                r[0] = i;
            }

            sc.send(&r, true);
            packets.push(r);
        }

        sleep_milli();

        let mut recved = 0;
        while let Ok(payload) = cc.recv() {
            recved += 1;

            let i = payload.get_slice().first().unwrap().to_owned() as usize;

            assert_eq!(&packets[i], payload.get_slice());
        }

        assert_eq!(recved, u8::MAX);
    }

    print_stats(&sc, &cc);
}
