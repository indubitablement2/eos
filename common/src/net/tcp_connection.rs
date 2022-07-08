use super::{tcp_loops::*, *};
use tokio::{net::TcpStream, spawn};

pub struct TcpConnection<InP>
where
    InP: Send + Packet,
{
    /// Send tcp packets to the connected peer.
    outbound_tcp_buffer_sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    pub inbound_tcp_receiver: crossbeam::channel::Receiver<InP>,
    tcp_buffer: Vec<u8>,
}
impl<InP: 'static> TcpConnection<InP>
where
    InP: Send + Packet,
{
    pub async fn new(stream: TcpStream) -> Self {
        // Create channels.
        let (outbound_tcp_buffer_sender, outbound_tcp_buffer_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (inbound_tcp_sender, inbound_tcp_receiver) = crossbeam::channel::unbounded();

        let (read, write) = stream.into_split();

        // Start the tcp loops.
        spawn(tcp_in_loop(inbound_tcp_sender, read));
        spawn(tcp_out_loop(outbound_tcp_buffer_receiver, write));

        Self {
            outbound_tcp_buffer_sender,
            inbound_tcp_receiver,
            tcp_buffer: Vec::new(),
        }
    }

    /// Send a packet to the connected peer over tcp.
    /// The serialized packet should not be larger than `MAX_RELIABLE_PACKET_SIZE`.
    ///
    /// Return if the packet could be sent.
    pub fn send_tcp_packet<P>(&mut self, packet: &P) -> bool
    where
        P: Packet,
    {
        let buf = packet.serialize();

        if buf.len() <= MAX_RELIABLE_PACKET_SIZE {
            // Add header to buffer.
            self.tcp_buffer.extend_from_slice(&(buf.len() as u16).to_be_bytes());

            // Add serialized packet to buffer.
            self.tcp_buffer.extend_from_slice(&buf);
            true
        } else {
            log::error!("Tcp packet failed to sent as it was too big {}.", buf.len());
            false
        }
    }

    /// Send buffered tcp packets.
    /// Call this when you don't expect to send new packet for a while.
    ///
    /// Return if it could be sent (eg: its not disconnected).
    pub fn flush_tcp_buffer(&mut self) -> bool {
        if self
            .outbound_tcp_buffer_sender
            .send(std::mem::take(&mut self.tcp_buffer))
            .is_err()
        {
            false
        } else {
            true
        }
    }
}
