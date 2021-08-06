use crate::const_var::*;
use crate::packet_common::*;
use bytes::{Buf, BufMut, BytesMut};
use flume::{bounded, unbounded, Receiver, Sender};
use log::*;
use std::convert::TryInto;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::io::Interest;
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};

/// Easily send and receive Packet.
pub struct Connection {
    /// Queue some Packet to be sent in batch next time poll is called.
    packet_to_send_sender: Sender<(Vec<u8>, u8)>,

    /// Potentially creat a data race you try to read this while a poll is in progress.
    pub client_local_packets_pointer: *const Vec<ClientLocalPacket>,
    /// Received Packet from previous poll call.
    pub client_global_packets_receiver: Receiver<ClientGlobalPacket>,

    /// Packet usualy from the server. Only used on client or for client login.
    pub other_packet_receiver: Receiver<OtherPacket>,
}
impl Connection {
    /// Send a packet without blocking. If this return false, Connection has been terminated.
    pub fn send_packet(&self, serialized_packet: (Vec<u8>, u8)) -> bool {
        self.packet_to_send_sender.try_send(serialized_packet).is_ok()
    }
}

/// The other part of a Connection where raw data is parsed into Packet.
struct InverseConnection {
    socket: TcpStream,

    /// Used when a header could be read, but not the payload yet.
    current_packet_type: u8,
    /// Used when a header could be read, but not the payload yet.
    current_packet_size: usize,
    /// Data read from this socket that could not be made into a full packet yet.
    unparsed_data_buffer: BytesMut,
    /// Generic buffer used for intermediary socket write.
    /// Keep data to be writen on unsuccessful socket write.
    write_bytes_buffer: BytesMut,
    /// Generic buffer used for intermediary socket read.
    read_byte_buffer: [u8; CONNECTION_READ_BUF_SIZE],

    /// Packet received from a Connection to write to the socket. Second value is packet type.
    packet_to_send_receiver: Receiver<(Vec<u8>, u8)>,

    /// Received ClientLocalPacket from previous poll call.
    client_local_packets: Vec<ClientLocalPacket>,
    /// Send ClientGlobalPacket.
    client_global_packet_sender: Sender<ClientGlobalPacket>,

    /// Send OtherPacket.
    other_packet_sender: Sender<OtherPacket>,
}

/// Enable starting connection from multiple thread. Cheaply clonable.
#[derive(Clone)]
pub struct ConnectionStarter {
    /// Used to start new connection.
    con_sender: Sender<InverseConnection>,
    /// Used to convert std TcpStream to tokio TcpStream.
    runtime_handle: tokio::runtime::Handle,
}
impl ConnectionStarter {
    /// Convert std TcpStream to Connection.
    /// Add a new socket that will be polled on call to poll.
    pub fn create_connection(&self, new_socket: std::net::TcpStream) -> Option<Connection> {
        // Convert to tokio TcpStream.
        let tokio_socket_result;
        if new_socket.set_nonblocking(true).is_ok() {
            tokio_socket_result = self.runtime_handle.block_on(async {
                if let Ok(result) = TcpStream::from_std(new_socket) {
                    return Some(result);
                }
                trace!("Error while converting async.");
                Option::None
            });
        } else {
            trace!("Error while setting non blocking.");
            return Option::None;
        }

        let (packet_to_send_sender, packet_to_send_receiver) = bounded(PACKET_BUFFER);
        let (client_global_packet_sender, client_global_packets_receiver) = bounded(PACKET_BUFFER);
        let (other_packet_sender, other_packet_receiver) = bounded(PACKET_BUFFER);
        let client_local_packets = Vec::with_capacity(PACKET_BUFFER);
        let client_local_packets_pointer = &client_local_packets as *const Vec<ClientLocalPacket>;

        if let Some(tokio_socket) = tokio_socket_result {
            let inv_con = InverseConnection {
                socket: tokio_socket,
                current_packet_type: u8::MAX,
                current_packet_size: 0,
                unparsed_data_buffer: BytesMut::with_capacity(CONNECTION_BUF_SIZE),
                write_bytes_buffer: BytesMut::with_capacity(CONNECTION_BUF_SIZE),
                read_byte_buffer: [0; CONNECTION_READ_BUF_SIZE],
                packet_to_send_receiver,
                client_local_packets,
                client_global_packet_sender,
                other_packet_sender,
            };

            if let Err(err) = self.con_sender.try_send(inv_con) {
                error!("Could not send inv_con to polling thread. It may have panicked: {:?}", &err);
                return Option::None;
            }

            return Some(Connection {
                packet_to_send_sender,
                client_local_packets_pointer,
                client_global_packets_receiver,
                other_packet_receiver,
            });
        }
        trace!("Could not create connection.");
        Option::None
    }
}

pub struct PollingThread {
    /// Update after polling, so this is the number from the last poll.
    pub current_active_socket: Arc<AtomicUsize>,
    /// Used to start polling.
    start_poll_sender: Sender<bool>,
    /// Used to start new connection.
    pub connection_starter: ConnectionStarter,
}
impl PollingThread {
    /// Init the thread pool. There should not be more than one of these.
    pub fn new(multithreaded: bool) -> PollingThread {
        // Create tokio runtime.
        let rt: Runtime;
        match multithreaded {
            true => rt = Builder::new_multi_thread().enable_all().build().unwrap(),
            false => rt = Builder::new_multi_thread().worker_threads(1).enable_all().build().unwrap(),
        }

        // Create channel used to start polling.
        let (start_poll_sender, start_poll_receiver) = bounded::<bool>(0);

        // Create channel used to start new connection.
        let (con_sender, con_receiver) = unbounded::<InverseConnection>();

        // Create atomic counter used to update current_active_socket.
        let current_active_socket = Arc::new(AtomicUsize::new(0));
        let current_active_socket_clone = current_active_socket.clone();

        let connection_starter = ConnectionStarter {
            con_sender,
            runtime_handle: rt.handle().clone(),
        };

        std::thread::spawn(move || {
            poll_loop(rt, start_poll_receiver, con_receiver, current_active_socket_clone);
        });

        PollingThread {
            start_poll_sender,
            current_active_socket,
            connection_starter,
        }
    }

    /// Start polling all Connections. If a poll is already started, return false.
    pub fn poll(&self) -> bool {
        self.start_poll_sender.try_send(true).is_ok()
    }

    /// Wait untill a poll could be started. Return false if receiver is dropped.
    pub fn wait_untill_poll(&self) -> bool {
        self.start_poll_sender.send(true).is_ok()
    }

    /// Test if it is currently polling.
    pub fn is_ready(&self) -> bool {
        self.start_poll_sender.try_send(false).is_ok()
    }

    /// Block untill polling is done. Return false if receiver is dropped.
    pub fn wait_untill_ready(&self) -> bool {
        self.start_poll_sender.send(false).is_ok()
    }
}

fn poll_loop(
    rt: Runtime,
    start_poll_receiver: Receiver<bool>,
    con_receiver: Receiver<InverseConnection>,
    current_active_socket: Arc<AtomicUsize>,
) {
    // Our current connection. Packed into a contiguous array.
    let mut connections: Vec<InverseConnection> = Vec::with_capacity(128);

    let mut task_handles = Vec::with_capacity(128);

    // Loop waiting for a signal.
    while let Ok(start) = start_poll_receiver.recv() {
        // * Check if this is just a test.
        if !start {
            continue;
        }

        // * Get new connection.
        while let Ok(new_con) = con_receiver.try_recv() {
            connections.push(new_con);
        }

        // * Spawn task.
        connections.drain(..).for_each(|con| {
            task_handles.push(rt.spawn(poll_connection(con)));
        });

        // * Wait for all task to finish.
        task_handles.drain(..).for_each(|handle| {
            if let Ok(Some(inv_con)) = rt.block_on(handle) {
                // Task has returned connection.
                connections.push(inv_con);
            }
        });

        // * Set current_active_socket.
        current_active_socket.store(connections.len(), std::sync::atomic::Ordering::Relaxed);
    }

    warn!("Poll loop terminated.");
}

async fn poll_connection(mut con: InverseConnection) -> Option<InverseConnection> {
    // * Clear previous client_local_packets.
    con.client_local_packets.fill(ClientLocalPacket::Invalid);

    // * Wait for the socket to be readable.
    let ready_result = con.socket.ready(Interest::READABLE | Interest::WRITABLE).await;

    match ready_result {
        Ok(ready) => {
            // * Merge all packets to write to this socket into write_bytes_buffer.
            while let Ok((payload, packet_type)) = con.packet_to_send_receiver.try_recv() {
                if let Ok(payload_size) = payload.len().try_into() {
                    if payload_size != 0 {
                        con.write_bytes_buffer.put_u8(packet_type);
                        con.write_bytes_buffer.put_u16(payload_size);
                        con.write_bytes_buffer.extend_from_slice(&payload);
                    } else {
                        trace!("Tried sending a 0 size payload.");
                    }
                } else {
                    warn!("Tried sending a payload over 65535 bytes long.");
                }
            }

            // * Check that write_bytes_buffer is not getting too large.
            if con.write_bytes_buffer.len() > 65535 {
                debug!(
                    "Terminating connection because write_bytes_buffer too large: {}",
                    con.write_bytes_buffer.len()
                );
                return Option::None;
            }

            // * Write write_bytes_buffer to socket.
            if !con.write_bytes_buffer.is_empty() && ready.is_writable() {
                match con.socket.try_write(&con.write_bytes_buffer) {
                    Ok(num) => {
                        trace!("Write bytes: {}", num);
                        // Remove writen bytes.
                        con.write_bytes_buffer.advance(num);
                    }
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            debug!("Unsuccessful socket write: {:?}", &err);
                        }
                        _ => {
                            debug!(
                                "Terminating connection because of error while trying to write socket: {:?}",
                                err
                            );
                            return Option::None;
                        }
                    },
                }
            }

            // * Read from socket.
            if ready.is_readable() {
                match con.socket.try_read(&mut con.read_byte_buffer) {
                    // TODO: Maybe read directly into unparsed_data_buffer.
                    Ok(num) => {
                        trace!("read bytes: {}", num);
                        if num > 0 {
                            // Got some data.
                            con.unparsed_data_buffer.reserve(num);
                            con.unparsed_data_buffer.put_slice(&con.read_byte_buffer[0..num]);
                        } else {
                            debug!("Terminating connection because read 0 byte.");
                            return Option::None;
                        }
                    }
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            trace!("Unsuccessful socket read: {:?}", &err);
                        }
                        _ => {
                            debug!(
                                "Terminating connection because of error while trying to write socket: {:?}",
                                err
                            );
                            return Option::None;
                        }
                    },
                }
            }
        }
        // Unsuccessful readiness.
        Err(err) => match err.kind() {
            std::io::ErrorKind::WouldBlock => {
                debug!("Cancelling connection poll: {:?}", &err);
                return Some(con);
            }
            _ => {
                debug!(
                    "Terminating connection because of error while waiting for readiness: {:?}",
                    &err
                );
                return Option::None;
            }
        },
    }

    loop {
        // * Parse a header, if we don't have one.
        if con.current_packet_type == u8::MAX && con.unparsed_data_buffer.remaining() >= 3 {
            // Read our new header.
            con.current_packet_type = con.unparsed_data_buffer.get_u8();
            con.current_packet_size = con.unparsed_data_buffer.get_u16().into();
        }

        // * Split packet from unparsed_data_buffer.
        if con.current_packet_type != u8::MAX && con.unparsed_data_buffer.remaining() >= con.current_packet_size {
            let payload = &con.unparsed_data_buffer.split_to(con.current_packet_size);

            // * Packet is complete. Deserialize and send.
            match con.current_packet_type {
                ClientLocalPacket::ID => {
                    let new_packet = ClientLocalPacket::deserialize(payload);
                    trace!("Received ClientLocalPacket: {:?}", &new_packet);
                    con.client_local_packets.push(new_packet);
                }
                ClientGlobalPacket::ID => {
                    let new_packet = ClientGlobalPacket::deserialize(payload);
                    trace!("Received ClientGlobalPacket: {:?}", &new_packet);
                    if let Err(err) = con.client_global_packet_sender.send(new_packet) {
                        debug!("Error sending ClientGlobalPacket: {:?}", &err);
                    }
                }
                OtherPacket::ID => {
                    let new_packet = OtherPacket::deserialize(payload);
                    trace!("Received OtherPacket: {:?}", &new_packet);
                    if let Err(err) = con.other_packet_sender.send(new_packet) {
                        debug!("Error sending ServerPacket: {:?}", &err);
                    }
                }
                _ => {
                    debug!("Invalid packet type: {}. Ignoring...", con.current_packet_type);
                }
            }

            // Packet was sent. Reset current packet size/type to default.
            con.current_packet_type = u8::MAX;
            con.current_packet_size = 0;
        } else {
            break;
        }
    }

    // No more packet could be parsed from unparsed_data_buffer.
    Some(con)
}
