use crate::idx::*;
use ahash::AHashMap;
use std::{
    net::SocketAddrV6,
    sync::{Arc, RwLock},
};
use tokio::{
    io::*,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    task::spawn_blocking,
};

pub enum TcpOutboundEvent {
    /// Send a packet (without header) to the connected peer.
    PacketEvent(Vec<u8>),
    /// Request a write flush.
    FlushEvent,
}

pub async fn tcp_out_loop(
    mut tcp_outbound_event_receiver: tokio::sync::mpsc::Receiver<TcpOutboundEvent>,
    mut buf_write: BufWriter<OwnedWriteHalf>,
    client_id: ClientId,
) {
    loop {
        if let Some(event) = tcp_outbound_event_receiver.recv().await {
            match event {
                TcpOutboundEvent::PacketEvent(packet) => {
                    // Write header.
                    match u16::try_from(packet.len()) {
                        Ok(payload_size) => {
                            if let Err(err) = buf_write.write_u16(payload_size).await {
                                debug!(
                                    "{:?} while writting header to {:?} 's tcp buf stream. Disconnecting...",
                                    err, client_id
                                );
                                break;
                            }
                        }
                        Err(err) => {
                            warn!(
                                "{:?} while getting payload size for {:?}. Ignoring packet and disconnecting...",
                                err, client_id
                            );
                            break;
                        }
                    }

                    // Write payload.
                    if let Err(err) = buf_write.write_all(&packet).await {
                        debug!(
                            "{:?} while writting to {:?} 's tcp buf stream. Disconnecting...",
                            err, client_id
                        );
                        break;
                    }
                }
                TcpOutboundEvent::FlushEvent => {
                    if let Err(err) = buf_write.flush().await {
                        debug!(
                            "{:?} while flushing {:?}'s tcp buf stream. Disconnecting...",
                            err, client_id
                        );
                        break;
                    }
                }
            }
        } else {
            debug!("Tcp sender channel for {:?} shutdown. Disconnecting...", client_id);
            break;
        }
    }
}

pub async fn tcp_in_loop(
    inbound_sender: crossbeam_channel::Sender<Vec<u8>>,
    mut buf_read: BufReader<OwnedReadHalf>,
    client_id: ClientId,
    udp_connections: Arc<RwLock<AHashMap<SocketAddrV6, crossbeam_channel::Sender<Vec<u8>>>>>,
    client_udp_address: SocketAddrV6,
) {
    let mut payload_buffer: Vec<u8> = Vec::new();
    loop {
        // Get a header.
        match buf_read.read_u16().await {
            Ok(next_payload_size) => {
                if next_payload_size == 0 {
                    debug!("{:?} sent a payload of 0 byte. Ignoring...", client_id);
                    continue;
                }

                let next_payload_size = next_payload_size as usize;

                // Increase buffer if needed.
                if next_payload_size > payload_buffer.len() {
                    payload_buffer.resize(next_payload_size, 0);
                }

                // Get packet.
                match buf_read.read_exact(&mut payload_buffer[..next_payload_size]).await {
                    Ok(num) => {
                        if num != next_payload_size {
                            debug!(
                                "{:?} disconnected while sending a packet. Ignoring packet...",
                                client_id
                            );
                            break;
                        }

                        if inbound_sender
                            .send(payload_buffer[..next_payload_size].to_vec())
                            .is_err()
                        {
                            debug!(
                                "Tcp inbound sender channel for {:?} shutdown. Disconnecting...",
                                client_id
                            );
                            break;
                        }
                    }
                    Err(err) => {
                        debug!(
                            "{:?} while reading {:?} 's tcp stream for a payload. Disconnecting...",
                            err, client_id
                        );
                        break;
                    }
                }
            }
            Err(err) => {
                debug!(
                    "{:?} while reading {:?} 's tcp stream for a header. Disconnecting...",
                    err, client_id
                );
                break;
            }
        }
    }

    // Also remove udp connection.
    spawn_blocking(move || {
        udp_connections.write().unwrap().remove(&client_udp_address);
    });

    debug!(
        "Tcp in loop for {:?} shutdown. Also queued udp connection removal.",
        client_id,
    );
}