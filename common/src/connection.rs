use crate::{idx::ClientId, packets::TcpPacket};
use std::net::SocketAddrV6;

pub struct Connection {
    pub client_id: ClientId,
    pub peer_tcp_addr: SocketAddrV6,
    pub peer_udp_addr: SocketAddrV6,
    /// Receive udp packet from the connected peer.
    pub udp_packet_received: crossbeam_channel::Receiver<Vec<u8>>,
    /// Receive tcp packet from the connected peer.
    pub tcp_packet_received: crossbeam_channel::Receiver<TcpPacket>,
    /// Send udp packet to connected peer.
    /// 
    /// Consider using `send_udp_packet` which automaticaly use the udp address in this struc.
    pub udp_packet_to_send: crossbeam_channel::Sender<(SocketAddrV6, Vec<u8>)>,
    /// Send tcp packet to connected peer.
    pub tcp_packet_to_send: tokio::sync::mpsc::Sender<TcpPacket>,
}
impl Connection {
    /// Udp packet above this size will be ignored.
    pub const MAX_UDP_PACKET_SIZE: usize = 1024;

    /// Send udp packet to connected peer.
    pub fn send_udp_packet(&self, packet: Vec<u8>) -> std::result::Result<(), crossbeam_channel::SendError<(SocketAddrV6, Vec<u8>)>> {
        self.udp_packet_to_send.send((self.peer_udp_addr, packet))
    }
}
