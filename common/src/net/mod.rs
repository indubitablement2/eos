pub mod connection;
pub mod packets;
pub mod tcp_loops;
pub mod login_packets;

/// The server's tcp port.
pub const SERVER_PORT: u16 = 31415;

/// Udp packet above this size will be truncated.
pub const MAX_UDP_PACKET_SIZE: usize = 1024;
/// Tcp packet above this size will cause the stream to be corrupted.
pub const MAX_TCP_PAYLOAD_SIZE: usize = u16::MAX as usize;

pub enum TcpOutboundEvent {
    /// Send a packet (without header) to the connected peer.
    PacketEvent(Vec<u8>),
    /// Request a write flush.
    FlushEvent,
}