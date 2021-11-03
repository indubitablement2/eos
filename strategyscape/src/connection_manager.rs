// use crate::packets::Packet;
// use bytes::BytesMut;
// use rapier2d::crossbeam::channel::*;
// use tokio::net::{TcpStream, UdpSocket};

// /// We will fragment larger Packet.
// const MAX_DATAGRAM_SIZE: usize = 400;

// /// How is the Packet sent.
// pub struct PacketSendCommand {
//     /// Redundantly send the same packet multiple times.
//     num_send: u32,
//     /// Requesting an acknowledgement that the packet was received.
//     reliable_send: bool,
// }

// /// Easily send and receive Packet.
// pub struct Connection {
//     /// Queue some Packet to be sent in batch next time poll is called.
//     pub packet_sender: Sender<(Packet, PacketSendCommand)>,
//     /// Packet usualy from the server.
//     pub packet_receiver: Receiver<Packet>,
// }

// /// The other part of a Connection where raw data is handled.
// struct InverseConnection {
//     socket: UdpSocket,

//     /// Used when a header could be read, but not the payload yet.
//     current_packet_type: u8,
//     /// Used when a header could be read, but not the payload yet.
//     current_packet_size: usize,
//     /// Data read from this socket that could not be made into a full packet yet.
//     unparsed_data_buffer: BytesMut,

//     /// Generic buffer used for intermediary socket write.
//     /// Keep data to be writen on unsuccessful socket write.
//     write_bytes_buffer: BytesMut,
//     /// Generic buffer used for intermediary socket read.
//     read_byte_buffer: [u8; 1400],

//     /// Packet received from a Connection to write to the socket.
//     packet_to_send_receiver: Receiver<Packet>,
//     /// Send Packet to the connection.
//     other_packet_sender: Sender<Packet>,
// }

// /// Enable starting connection from multiple thread. Cheaply clonable.
// #[derive(Clone)]
// pub struct ConnectionStarter {
//     /// Used to start new connection.
//     con_sender: Sender<InverseConnection>,
//     /// Used to convert std TcpStream to tokio TcpStream.
//     pub runtime_handle: tokio::runtime::Handle,
// }
// impl ConnectionStarter {
//     /// Convert a std TcpStream Convert to tokio TcpStream.
//     pub fn convert_std_to_tokio(&self, new_socket: std::net::TcpStream) -> Option<tokio::net::TcpStream> {
//         if new_socket.set_nonblocking(true).is_ok() {
//             return self.runtime_handle.block_on(async {
//                 if let Ok(result) = TcpStream::from_std(new_socket) {
//                     return Some(result);
//                 }
//                 trace!("Error while converting to async.");
//                 Option::None
//             });
//         } else {
//             trace!("Error while setting non blocking.");
//             return Option::None;
//         }
//     }

//     /// Add a new socket that will be polled on call to poll.
//     pub fn create_connection(&self, new_socket: tokio::net::TcpStream) -> Connection {
//         let (packet_to_send_sender, packet_to_send_receiver) = bounded(PACKET_BUFFER);
//         let (client_global_packet_sender, client_global_packets_receiver) = bounded(PACKET_BUFFER);
//         let (other_packet_sender, other_packet_receiver) = bounded(PACKET_BUFFER);
//         let client_local_packets = Vec::with_capacity(PACKET_BUFFER);
//         let client_local_packets_pointer = &client_local_packets as *const Vec<ClientLocalPacket>;

//         let inv_con = InverseConnection {
//             socket: new_socket,
//             current_packet_type: u8::MAX,
//             current_packet_size: 0,
//             unparsed_data_buffer: BytesMut::with_capacity(CONNECTION_BUF_SIZE),
//             write_bytes_buffer: BytesMut::with_capacity(CONNECTION_BUF_SIZE),
//             read_byte_buffer: [0; CONNECTION_READ_BUF_SIZE],
//             packet_to_send_receiver,
//             client_local_packets,
//             client_global_packet_sender,
//             other_packet_sender,
//         };

//         if let Err(err) = self.con_sender.try_send(inv_con) {
//             error!("Could not send inv_con to polling thread. It may have panicked: {:?}", &err);
//         }

//         Connection {
//             packet_to_send_sender,
//             client_local_packets_pointer,
//             client_global_packets_receiver,
//             other_packet_receiver,
//         }
//     }
// }
