use crate::connection::Connection;
use ahash::AHashMap;
use crossbeam_channel::*;
use std::{
    net::{SocketAddrV6, UdpSocket},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

pub enum UdpConnectionEvent {
    Connected {
        addr: SocketAddrV6,
        inbound_sender: crossbeam_channel::Sender<Vec<u8>>,
    },
    Disconnected {
        addr: SocketAddrV6,
    },
}

pub fn udp_out_loop(socket: Arc<UdpSocket>, udp_outbound_receiver: Receiver<(SocketAddrV6, Vec<u8>)>) {
    // Send packets.
    for (addr, packet) in udp_outbound_receiver.iter() {
        // Check packet size.
        if packet.len() > Connection::MAX_UDP_PACKET_SIZE {
            warn!("Attempted to send an udp packet of {} bytes. Ignoring...", packet.len());
            continue;
        }

        let _ = socket.send_to(&packet, addr);
    }
}

pub fn udp_in_loop(socket: Arc<UdpSocket>, udp_connection_event_receiver: Receiver<UdpConnectionEvent>) {
    let mut connections: AHashMap<SocketAddrV6, Sender<Vec<u8>>> = AHashMap::new();
    let mut connection_to_remove = Vec::new();
    let mut buf = [0; Connection::MAX_UDP_PACKET_SIZE];

    'outer: loop {
        // Handle connection events.
        loop {
            match udp_connection_event_receiver.try_recv() {
                Ok(event) => match event {
                    UdpConnectionEvent::Connected { addr, inbound_sender } => {
                        connections.insert(addr, inbound_sender);
                    }
                    UdpConnectionEvent::Disconnected { addr } => {
                        connections.remove(&addr);
                    }
                },
                Err(err) => match err {
                    TryRecvError::Empty => {
                        break;
                    }
                    TryRecvError::Disconnected => {
                        break 'outer;
                    }
                },
            }
        }

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

                    if let Some(sender) = connections.get(&addr) {
                        if sender.send(buf[..num].to_vec()).is_err() {
                            connection_to_remove.push(addr);
                        }
                    } else {
                        debug!(
                            "Got an udp packet from an unconnected address ({:?}). Ignoring...",
                            addr
                        );
                    }
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                    } else if err.kind() == std::io::ErrorKind::Interrupted {
                        debug!("{:?} while receiving packet. Breaking from recv loop...", err);
                    } else {
                        error!("{:?} while receiving packet. Terminating thread...", err);
                        break 'outer;
                    }
                    break;
                }
            }
        }

        // Remove disconnected connection.
        for addr in connection_to_remove.drain(..) {
            connections.remove(&addr);
        }

        sleep(Duration::MILLISECOND);
    }
}
