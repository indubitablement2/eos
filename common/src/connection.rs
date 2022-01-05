use crate::{idx::ClientId, tcp_loops::TcpOutboundEvent};
use std::net::SocketAddrV6;

pub struct Connection {
    pub client_id: ClientId,
    pub peer_tcp_addr: SocketAddrV6,
    pub peer_udp_addr: SocketAddrV6,
    /// Receive packet from the connected peer.
    pub inbound_receiver: crossbeam_channel::Receiver<Vec<u8>>,
    /// Send udp packet to connected peer.
    udp_outbound_sender: crossbeam_channel::Sender<(SocketAddrV6, Vec<u8>)>,
    /// Send tcp packet to the connected peer and request flush.
    tcp_outbound_event_sender: tokio::sync::mpsc::Sender<TcpOutboundEvent>,
}
impl Connection {
    /// Udp packet above this size will be truncated.
    pub const MAX_UDP_PACKET_SIZE: usize = 1024;
    /// Tcp packet above this size will cause the stream to be corrupted.
    pub const MAX_TCP_PAYLOAD_SIZE: usize = u16::MAX as usize;

    pub fn new(
        client_id: ClientId,
        peer_tcp_addr: SocketAddrV6,
        peer_udp_addr: SocketAddrV6,
        inbound_receiver: crossbeam_channel::Receiver<Vec<u8>>,
        udp_outbound_sender: crossbeam_channel::Sender<(SocketAddrV6, Vec<u8>)>,
        tcp_outbound_event_sender: tokio::sync::mpsc::Sender<TcpOutboundEvent>,
    ) -> Self {
        Self {
            client_id,
            peer_tcp_addr,
            peer_udp_addr,
            inbound_receiver,
            udp_outbound_sender,
            tcp_outbound_event_sender,
        }
    }

    /// Send packet to the connected peer over udp or tcp.
    ///
    /// Return if there was an error sending the packet (the channel is disconnected).
    pub fn send_packet(&self, packet: Vec<u8>, tcp: bool) -> bool {
        if tcp {
            self.tcp_outbound_event_sender
                .blocking_send(TcpOutboundEvent::PacketEvent(packet))
                .is_err()
        } else {
            self.udp_outbound_sender.send((self.peer_udp_addr, packet)).is_err()
        }
    }

    /// Send buffered packets.
    ///
    /// Call this when you don't expect to send new packet for a while.
    pub fn flush_tcp_stream(&self) -> bool {
        self.tcp_outbound_event_sender
            .blocking_send(TcpOutboundEvent::FlushEvent)
            .is_err()
    }
}
