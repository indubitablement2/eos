use crate::connection::Connection;
use ahash::AHashMap;
use crossbeam_channel::*;
use std::{
    net::{SocketAddrV6, UdpSocket},
    sync::{Arc, RwLock},
    thread::sleep,
    time::{Duration, Instant},
};

/// Socket should be in nonblocking mode.
pub fn udp_in_loop(socket: Arc<UdpSocket>, udp_connections: Arc<RwLock<AHashMap<SocketAddrV6, Sender<Vec<u8>>>>>) {
    let mut buf = [0; Connection::MAX_UDP_PACKET_SIZE];

    'outer: loop {
        let start = Instant::now();

        // Receive packets.
        loop {
            match socket.recv_from(&mut buf) {
                Ok((num, origin)) => {
                    if num == 1 {
                        // This is a ping.
                        let _ = socket.send_to(&buf[..1], origin);
                    }

                    let addr = match origin {
                        std::net::SocketAddr::V4(a) => {
                            debug!("Got an udp packet from an ipv4 address ({:?}). Ignoring...", a);
                            continue;
                        }
                        std::net::SocketAddr::V6(a) => a,
                    };

                    if let Some(sender) = udp_connections.read().unwrap().get(&addr) {
                        let _ = sender.send(buf[..num].to_vec());
                    } else {
                        debug!(
                            "Got an udp packet from an unconnected address ({:?}). Ignoring...",
                            addr
                        );
                    }
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                        break;
                    } else if err.kind() == std::io::ErrorKind::Interrupted {
                        debug!("{:?} while receiving packet. Breaking from recv loop...", err);
                    } else {
                        error!("{:?} while receiving packet. Terminating thread...", err);
                        break 'outer;
                    }
                }
            }
        }

        // Sleep if we have time left.
        if let Some(remaining_duration) = Duration::from_micros(500).checked_sub(start.elapsed()) {
            sleep(remaining_duration);
        }
    }
}
