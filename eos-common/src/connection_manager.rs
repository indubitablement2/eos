use crate::const_var::*;
use crate::idx::ClientId;
use crate::packet_mod::*;
use bytes::{Buf, BufMut, BytesMut};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use log::*;
use parking_lot::RwLock;
use std::convert::TryInto;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use tokio::io::Interest;
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};

/// Easily send and receive Packet.
pub struct Connection {
    /// Allow manual disconnecting and check if Connection is still valid.
    disconnected: Arc<AtomicBool>,
    /// Queue some Packet to be sent in batch next time poll is called.
    sender: Sender<(Vec<u8>, u8)>,
    /// Received Packet from previous poll call.
    pub local_packets: Arc<RwLock<Vec<ClientLocalPacket>>>,
    pub login_packet: Receiver<ClientLoginPacket>,
    pub address: SocketAddr,
    send_client_id: Arc<AtomicU32>,
}
impl Connection {
    /// Send a packet without blocking. If this return false, Connection has been terminated.
    pub fn send_packet(&self, serialized_packet: (Vec<u8>, u8)) -> bool {
        self.sender.try_send(serialized_packet).is_ok()
    }

    /// Queue this Connection to be disconnected next poll. Any pending packet will be dropped.
    pub fn disconnect(&self) {
        self.disconnected.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Test if this Connection will still be polled.
    pub fn is_disconnected(&self) -> bool {
        self.disconnected.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// ClientId is used to identify client global packet. This will only work once!
    pub fn set_client_id(&self, client_id: u32) {
        self.send_client_id.store(client_id, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The other part of a Connection where raw data is parsed into Packet.
struct InverseConnection {
    socket: TcpStream,
    /// Allow manual disconnecting and check if Connection is still valid.
    disconnected: Arc<AtomicBool>,
    /// Used when a header could be read, but not the payload yet.
    current_packet_type: u8,
    /// Used when a header could be read, but not the payload yet.
    current_packet_size: usize,
    /// Data read from this socket that could not be made into a full packet yet.
    unparsed_data_buffer: BytesMut,
    /// Generic buffer used for intermediary socket write.
    /// Store data to be writen on unsuccessful socket write.
    write_bytes_buffer: BytesMut,
    /// Generic buffer used for intermediary socket read.
    read_byte_buffer: Vec<u8>,
    /// Packet received from a Connection to write to the socket.
    packet_to_send: Receiver<(Vec<u8>, u8)>,
    /// Received ClientLocalPacket from previous poll call.
    local_packets: Arc<RwLock<Vec<ClientLocalPacket>>>,
    /// Send ClientGlobalPacket if ClientId != 0.
    global_packet_sender: Sender<(ClientId, ClientGlobalPacket)>,
    /// Receive ServerPacket. Used on client.
    server_packet_sender: Sender<ServerPacket>,
    /// Send ClientLoginPacket to Connection.
    login_packet_sender: Sender<ClientLoginPacket>,
    /// Receive client id. Only read if client_id is ClientId(0).
    recv_client_id: Arc<AtomicU32>,
    /// The ClientId associated with this connection. Used when receiving global client packet.
    client_id: ClientId,
}

/// Enable starting connection from multiple thread. Cheaply clonable.
#[derive(Clone)]
pub struct ConnectionStarter {
    /// Used to start new connection.
    con_sender: Sender<InverseConnection>,
    /// Used to convert std TcpStream to tokio TcpStream.
    runtime_handle: tokio::runtime::Handle,
    /// A copy to clone to all new InverConnection.
    global_packet_sender: Sender<(ClientId, ClientGlobalPacket)>,
    /// A copy to clone to all new InverConnection.
    server_packet_sender: Sender<ServerPacket>,
}
impl ConnectionStarter {
    /// Convert std TcpStream to Connection.
    /// Add a new socket that will be polled on call to poll.
    pub fn create_connection(&self, new_socket: std::net::TcpStream, address: SocketAddr) -> Option<Connection> {
        let (packet_sender, packet_receiver) = unbounded::<(Vec<u8>, u8)>();
        let (login_packet_sender, login_packet_receiver) = unbounded();
        let unparsed_data_buffer = BytesMut::with_capacity(CONNECTION_BUF_SIZE);
        let write_bytes_buffer = BytesMut::with_capacity(CONNECTION_BUF_SIZE);
        let read_byte_buffer = vec![0; 4096];
        let disconnected = Arc::new(AtomicBool::new(false));
        let local_packets = Arc::new(RwLock::new(Vec::with_capacity(8)));
        let s_client_id = Arc::new(AtomicU32::new(0));

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

        if let Some(tokio_socket) = tokio_socket_result {
            let inv_con = InverseConnection {
                socket: tokio_socket,
                disconnected: disconnected.clone(),
                unparsed_data_buffer,
                write_bytes_buffer,
                read_byte_buffer,
                packet_to_send: packet_receiver,
                current_packet_size: 0,
                local_packets: local_packets.clone(),
                recv_client_id: s_client_id.clone(),
                client_id: ClientId(0),
                global_packet_sender: self.global_packet_sender.clone(),
                server_packet_sender: self.server_packet_sender.clone(),
                current_packet_type: u8::MAX,
                login_packet_sender,
            };

            if let Err(err) = self.con_sender.try_send(inv_con) {
                error!("Could not send inv_con to polling thread. It may have panicked: {:?}", &err);
                return Option::None;
            }

            return Some(Connection {
                disconnected,
                sender: packet_sender,
                address,
                local_packets,
                send_client_id: s_client_id,
                login_packet: login_packet_receiver,
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
    start_poll_sender: Sender<()>,
    /// Used to start new connection.
    pub connection_starter: ConnectionStarter,
    /// Packet from clients. Only used on server.
    pub global_packet_receiver: Receiver<(crate::idx::ClientId, ClientGlobalPacket)>,
    /// Packet from the server. Only used on client.
    pub server_packet_receiver: Receiver<ServerPacket>,
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
        let (start_poll_sender, start_poll_receiver) = bounded::<()>(0);

        // Create channel used to start new connection.
        let (con_sender, con_receiver) = unbounded::<InverseConnection>();

        // Create atomic counter used to update current_active_socket.
        let current_active_socket = Arc::new(AtomicUsize::new(0));
        let current_active_socket_clone = current_active_socket.clone();

        // Create channel used to send/receive server/global packet.
        let (global_packet_sender, global_packet_receiver) = unbounded();
        let (server_packet_sender, server_packet_receiver) = unbounded();

        let connection_starter = ConnectionStarter {
            con_sender,
            runtime_handle: rt.handle().clone(),
            global_packet_sender,
            server_packet_sender,
        };

        std::thread::spawn(move || {
            poll_loop(rt, start_poll_receiver, con_receiver, current_active_socket_clone);
        });

        PollingThread {
            start_poll_sender,
            current_active_socket,
            connection_starter,
            global_packet_receiver,
            server_packet_receiver,
        }
    }

    /// Start polling all Connections. If a poll is already started, return false.
    pub fn poll(&mut self) -> bool {
        self.start_poll_sender.try_send(()).is_ok()
    }
}

fn poll_loop(
    rt: Runtime,
    start_poll_receiver: Receiver<()>,
    con_receiver: Receiver<InverseConnection>,
    current_active_socket: Arc<AtomicUsize>,
) {
    // Our current connection. Packed into a contiguous array.
    let mut connections: Vec<InverseConnection> = Vec::with_capacity(128);

    let mut task_handles = Vec::with_capacity(128);

    // Loop waiting for a signal.
    while start_poll_receiver.recv().is_ok() {
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
    // * Check if we should drop.
    if con.disconnected.load(std::sync::atomic::Ordering::Relaxed) {
        return Option::None;
    }

    // * Wait for the socket to be readable.
    let ready_result = con.socket.ready(Interest::READABLE | Interest::WRITABLE).await;

    match ready_result {
        Ok(ready) => {
            // * Merge all packets to write to this socket into write_bytes_buffer.
            while let Ok((payload, packet_type)) = con.packet_to_send.try_recv() {
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
                con.disconnected.store(true, std::sync::atomic::Ordering::Relaxed);
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
                            con.disconnected.store(true, std::sync::atomic::Ordering::Relaxed);
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
                            con.disconnected.store(true, std::sync::atomic::Ordering::Relaxed);
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
                            con.disconnected.store(true, std::sync::atomic::Ordering::Relaxed);
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
                con.disconnected.store(true, std::sync::atomic::Ordering::Relaxed);
                return Option::None;
            }
        },
    }

    {
        // * Lock packets vector.
        let mut local_packets_lock = con.local_packets.write();

        // * Clear previous packets.
        local_packets_lock.clear();

        // * Check if ClientId is set.
        if con.client_id == ClientId(0) {
            con.client_id = ClientId(con.recv_client_id.load(std::sync::atomic::Ordering::Relaxed));
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
                        local_packets_lock.push(new_packet);
                    }
                    ClientGlobalPacket::ID => {
                        let new_packet = ClientGlobalPacket::deserialize(payload);
                        trace!("Received ClientGlobalPacket: {:?}", &new_packet);
                        if con.client_id == ClientId(0) {
                            debug!("Received a ClientGlobalPacket while ClientId is 0. Ignoring...");
                            continue;
                        }
                        if let Err(err) = con.global_packet_sender.send((con.client_id, new_packet)) {
                            debug!("Error sending ClientGlobalPacket: {:?}", &err);
                        }
                    }
                    ClientLoginPacket::ID => {
                        let new_packet = ClientLoginPacket::deserialize(payload);
                        trace!("Received ClientLoginPacket: {:?}", &new_packet);
                        if let Err(err) = con.login_packet_sender.send(new_packet) {
                            debug!("Error sending ClientLoginPacket: {:?}", &err);
                        }
                    }
                    ServerPacket::ID => {
                        let new_packet = ServerPacket::deserialize(payload);
                        trace!("Received ServerPacket: {:?}", &new_packet);
                        if let Err(err) = con.server_packet_sender.send(new_packet) {
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
    }

    // No more packet could be parsed from unparsed_data_buffer.
    Some(con)
}
