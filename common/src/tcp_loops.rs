use crate::idx::*;
use crate::packets::*;
use crate::udp_loops::UdpConnectionEvent;
use std::{
    net::SocketAddrV6,
};
use tokio::{
    io::*,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

pub async fn tcp_out_loop(
    mut tcp_to_send: tokio::sync::mpsc::Receiver<TcpPacket>,
    mut write: OwnedWriteHalf,
    client_id: ClientId,
) {
    let mut buf: Vec<u8> = Vec::new();

    loop {
        if let Some(packet) = tcp_to_send.recv().await {
            // Resize buf so that serialized packet fit into.
            let packet_size = if let Some(packet_size) = packet.serialized_size() {
                if packet_size + 2 > buf.len() {
                    buf.resize(packet_size + 2, 0);
                }
                packet_size
            } else {
                continue;
            };

            // Add header.
            if let Ok(header) = u16::try_from(packet_size) {
                buf[..2].copy_from_slice(&header.to_be_bytes());
            } else {
                warn!("Tried to send a packet of {} bytes which is above size limit of {}. Ignoring...", packet_size, TcpPacket::MAX_SIZE);
                continue;
            }

            // Add packet.
            if !packet.serialize_into(&mut buf[2..]) {
                continue;
            }

            if let Err(err) = write.write_all(&buf[..packet_size + 2]).await {
                if is_err_fatal(&err) {
                    debug!(
                        "Fatal error while writting to {:?} 's tcp stream. Disconnecting...",
                        client_id
                    );
                    break;
                }
            }
        } else {
            debug!("Tcp sender channel for {:?} shutdown. Disconnecting...", client_id);
            break;
        }
    }
}

pub async fn tcp_in_loop(
    tcp_received: crossbeam_channel::Sender<TcpPacket>,
    mut buf_read: BufReader<OwnedReadHalf>,
    client_id: ClientId,
    udp_connection_event_sender: crossbeam_channel::Sender<UdpConnectionEvent>,
    udp_address: SocketAddrV6,
) {
    let mut header_buffer = [0u8; 2];
    let mut payload_buffer: Vec<u8> = Vec::new();
    loop {
        // Get a header.
        match buf_read.read_exact(&mut header_buffer).await {
            Ok(num) => {
                if num == 0 {
                    debug!("{:?} disconnected.", client_id);
                    break;
                }

                let next_payload_size = u16::from_be_bytes(header_buffer) as usize;

                if next_payload_size > payload_buffer.len() {
                    // Next packet will need a bigger buffer.
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

                        // Try to deserialize.
                        match TcpPacket::deserialize(&payload_buffer[..next_payload_size]) {
                            Some(packet) => {
                                // Send packet to channel.
                                if tcp_received.send(packet).is_err() {
                                    debug!("Tcp sender channel for {:?} shutdown. Disconnecting...", client_id);
                                    break;
                                }
                            }
                            None => {
                                debug!(
                                    "Error while deserializing {:?} 's TcpPacket. Disconnecting...",
                                    client_id
                                );
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        if is_err_fatal(&err) {
                            debug!(
                                "Fatal error while reading {:?} 's tcp stream for a payload. Disconnecting...",
                                client_id
                            );
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                if is_err_fatal(&err) {
                    debug!(
                        "Fatal error while reading {:?} 's tcp stream for a header. Disconnecting...",
                        client_id
                    );
                    break;
                }
            }
        }
    }

    // Also remove udp address.
    udp_connection_event_sender.send(UdpConnectionEvent::Disconnected {
        addr: udp_address,
    }).expect("should be hable to send udp UdpConnectionEvent::Disconnected");
    debug!(
        "Tcp in loop for {:?} shutdown. Also sent disconnected event to udp loop.",
        client_id,
    );
}

/// If the io error is fatal, return true and print the err to debug log.
fn is_err_fatal(err: &Error) -> bool {
    if err.kind() == std::io::ErrorKind::WouldBlock || err.kind() == std::io::ErrorKind::Interrupted {
        false
    } else {
        debug!("Fatal io error {:?}.", err);
        true
    }
}
