use super::*;
use ahash::AHashMap;
use crossbeam::channel::Sender;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct ConnectionsInternals {
    pub connections: Arc<RwLock<AHashMap<SocketAddr, Sender<Vec<u8>>>>>,
}

pub fn inbound_loop(socket: Arc<UdpSocket>, connections_internals: ConnectionsInternals, shutdown: Arc<AtomicBool>) {
    let mut packets_buffer = Vec::new();
    let mut buf = [0; MAX_PACKET_SIZE];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((num, origin)) => {
                if num != 0 {
                    if let Ok(connections) = connections_internals.connections.try_read() {
                        // Send the new packet.
                        if let Some(inbound_sender) = connections.get(&origin) {
                            if let Err(err) = inbound_sender.send(buf[..num].to_vec()) {
                                trace!("{:?} can not send packet in channel. Ignoring...", err);
                            }
                        } else {
                            trace!("Got a packet from an unconnected origin ({:?}). Ignoring...", origin);
                        }

                        // Empty our packet buffer while we have the lock.
                        for (origin, packet) in packets_buffer.drain(..) {
                            if let Some(inbound_sender) = connections.get(&origin) {
                                let _ = inbound_sender.send(packet);
                            }
                        }
                    } else {
                        // Buffering internally intil we can get the read lock.
                        packets_buffer.push((origin, buf[..num].to_vec()));
                    }
                }
            }
            Err(err) => {
                if err.kind() != std::io::ErrorKind::Interrupted {
                    error!("{:?}, while reading socket. Terminating inbound thread...", err);
                    break;
                }
            }
        }

        // Check if we should shutdown.
        if shutdown.load(Ordering::Relaxed) {
            info!("Terminating inbound thread as requested.");
            break;
        }
    }

    // Empty our internal packet buffer.
    let connections = connections_internals.connections.read().unwrap();
    for (origin, packet) in packets_buffer.drain(..) {
        if let Some(inbound_sender) = connections.get(&origin) {
            let _ = inbound_sender.send(packet);
        }
    }
}
