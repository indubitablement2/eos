use gdnative::core_types::Vector2;
use std::net::UdpSocket;
use flume::*;

/// Packet originating from client or server.
#[derive(Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Packet {
    Invalid,
    Movement {
        wish_dir: Option<Vector2>,
    },
    /// Send a chat message.
    Broadcast {
        message: String,
    },
}

impl Default for Packet {
    fn default() -> Self {
        Self::Invalid
    }
}

pub trait Packetable {
    /// Serialize into a Vec<u8> and a byte to be sent to a Connection.
    fn serialize(&self) -> Vec<u8>;
    /// Deserialize some bytes into a Packet. Return Default if any error occur.
    fn deserialize(packet: &[u8]) -> Self
    where
        Self: Sized;
}

impl Packetable for Packet {
    fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    fn deserialize(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        bincode::deserialize(bytes).unwrap_or_default()
    }
}

/// Specify how the packet is sent.
pub struct PacketParam {
    /// The packet guarantee to be transmitted.
    pub reliable: bool,
    /// Send multiple copy of the packet.
}

/// Easily send and receive Packet.
pub struct Connection {
    /// Queue some Packet to be sent in batch next time poll is called.
    packet_to_send_sender: Sender<Packet>,

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

#[test]
fn my_test() {
    let packet = Packet::Movement {
        wish_dir: Some(Vector2::ONE),
    };

    let serd = packet.serialize();

    println!("{:?}", &serd);
}