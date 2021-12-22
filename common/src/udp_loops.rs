use crate::connection::Connection;
use std::{net::{SocketAddrV6, UdpSocket}, sync::Arc, time::{Instant, Duration}, thread::sleep};
use ahash::AHashMap;
use crossbeam_channel::*;

pub enum UdpConnectionEvent {
    Connected {
        client_udp_addr: SocketAddrV6,
        udp_packet_received_sender: crossbeam_channel::Sender<Vec<u8>>,
    },
    Disconnected {
        addr: SocketAddrV6,
    },
}

pub fn udp_out_loop(
    socket: Arc<UdpSocket>,
    udp_packet_to_send_receiver: Receiver<(SocketAddrV6, Vec<u8>)>,
) {
    // Send packets.
    for (addr, packet) in udp_packet_to_send_receiver.iter() {
        // Check packet size.
        if packet.len() > Connection::MAX_UDP_PACKET_SIZE {
            error!("Attempted to send an udp packet of {} bytes. Ignoring...", packet.len());
            continue;
        }

        if let Err(err) = socket.send_to(&packet, addr) {
            if err.kind() == std::io::ErrorKind::WouldBlock || err.kind() == std::io::ErrorKind::Interrupted {
                debug!("{:?} while sending packet. Ignoring...",err);
            } else {
                error!("{:?} while sending packet. Terminating udp out thread...", err);
                break;
            }
        }
    }
}

pub fn udp_in_loop(
    socket: Arc<UdpSocket>,
    udp_connection_event_receiver: Receiver<UdpConnectionEvent>,
) {
    let mut connections: AHashMap<SocketAddrV6, Sender<Vec<u8>>> = AHashMap::new();
    let mut connection_to_remove = Vec::new();
    let mut buf = [0; Connection::MAX_UDP_PACKET_SIZE];

    'outer: loop {
        let start = Instant::now();

        // Handle connection events.
        loop {
            match udp_connection_event_receiver.try_recv() {
                Ok(event) => {
                    match event {
                        UdpConnectionEvent::Connected { client_udp_addr, udp_packet_received_sender } => {
                            connections.insert(client_udp_addr, udp_packet_received_sender);
                        }
                        UdpConnectionEvent::Disconnected { addr } => {
                            connections.remove(&addr);
                        }
                    }
                }
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
                    let addr = match origin {
                        std::net::SocketAddr::V4(a) => {
                            trace!("Got an udp packet from an ipv4 address ({:?}). Ignoring...", a);
                            continue;
                        }
                        std::net::SocketAddr::V6(a) => a,
                    };

                    if let Some(sender) = connections.get(&addr) {
                        if sender.send(buf[..num].to_vec()).is_err() {
                            connection_to_remove.push(addr);
                        }
                    } else {
                        trace!(
                            "Got an udp packet from an unconnected address ({:?}). Ignoring...",
                            addr
                        );
                    }
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {}
                    else if err.kind() == std::io::ErrorKind::Interrupted {
                        trace!("{:?} while receiving packet. Breaking from recv loop...", err);
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

        if let Some(remaining) = Duration::MILLISECOND.checked_sub(start.elapsed()) {
            sleep(remaining);
        }
    }
}