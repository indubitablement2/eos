use super::*;
use crate::idx::ClientId;
use std::{
    net::{SocketAddrV6, UdpSocket},
    sync::Arc,
};

pub struct Connection {
    client_id: ClientId,
    peer_addr: SocketAddrV6,
    /// Receive packet from the connected peer.
    inbound_receiver: crossbeam::channel::Receiver<Vec<u8>>,
    /// Send udp packet to connected peer.
    socket: Arc<UdpSocket>,
    /// Send tcp packet to the connected peer or request a flush.
    tcp_outbound_event_sender: tokio::sync::mpsc::Sender<TcpOutboundEvent>,
}
impl Connection {
    pub fn new(
        client_id: ClientId,
        peer_addr: SocketAddrV6,
        inbound_receiver: crossbeam::channel::Receiver<Vec<u8>>,
        socket: Arc<UdpSocket>,
        tcp_outbound_event_sender: tokio::sync::mpsc::Sender<TcpOutboundEvent>,
    ) -> Self {
        Self {
            client_id,
            peer_addr,
            inbound_receiver,
            socket,
            tcp_outbound_event_sender,
        }
    }

    /// Send a packet to the connected with no delivery garanty.
    /// If the packet is small enough, it is sent over udp otherwise tcp is used.
    ///
    /// Return if the packet could be sent.
    pub fn send_packet_unreliable(&self, packet: Vec<u8>) -> bool {
        debug_assert!(packet.len() <= MAX_UDP_PACKET_SIZE);
        if packet.len() > MAX_UDP_PACKET_SIZE {
            self.send_packet_reliable(packet)
        } else{
            self.socket.send_to(&packet, self.peer_addr).is_err()
        }
    }

    /// Send a packet to the connected peer over tcp.
    ///
    /// Return if there was an error sending the packet (the channel is disconnected).
    pub fn send_packet_reliable(&self, packet: Vec<u8>) -> bool {
        debug_assert!(packet.len() < MAX_TCP_PAYLOAD_SIZE);

        self.tcp_outbound_event_sender
            .blocking_send(TcpOutboundEvent::PacketEvent(packet))
            .is_err()
    }

    /// Send buffered packets.
    ///
    /// Call this when you don't expect to send new packet for a while.
    pub fn flush_tcp_stream(&self) -> bool {
        self.tcp_outbound_event_sender
            .blocking_send(TcpOutboundEvent::FlushEvent)
            .is_err()
    }

    /// Try to receive a single packet from this connection.
    pub fn try_recv(&self) -> Result<Vec<u8>, crossbeam::channel::TryRecvError> {
        self.inbound_receiver.try_recv()
    }

    /// Get a reference to the connection's client id.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }
}
