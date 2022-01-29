use super::*;
use ahash::AHashMap;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionEvent {
    /// We or the client as not sent any packet for some time.
    Timeout,
    /// Connected peer is sending too many packets.
    Flood,
}

/// Header - 1
/// -
/// 0..1 - has ack request bool
///
/// 1..2 - fragmented bool
///
/// 2..6 - num appended ack
///
/// 6..8 - unused
///
/// Payload - 0..1024
/// -
/// Payload can have multiple packet.
///
/// Fragment - 0..4
/// -
/// If this is a fragmented packet.
///
/// u16 - Fragment id.
/// u8 - Part number.
/// u8 - Num part in fragment.
///
/// Ack request - 0..2
/// -
/// u16 - If there is an ack request.
/// Reliable fragmented packet needs its individual fragment to be acked.
///
/// Acks confirmation - 0..16
/// -
/// u16 - If these are ack confirmation. Up to 8 can be appended per packet.
pub struct Connection {
    address: SocketAddr,
    socket: Arc<UdpSocket>,
    inbound_receiver: Receiver<Vec<u8>>,

    /// The last ack that was used for a reliable packet.
    current_ack: u16,
    /// The last ack that was used for a fragmented packet.
    current_fragment: u16,

    /// Reliable packet that are awaiting to be sent.
    ///
    /// These should only need to have their ack request updated (2 last bytes).
    pending_send: Vec<Vec<u8>>,
    /// Reliable packets that were sent, but are awaiting an ack confirmantion.
    pending_ack: AHashMap<u16, (Vec<u8>, f32)>,

    buffer: Box<[u8; MAX_PACKET_SIZE]>,
    /// The total number of bytes used in the buffer.
    used_buffer: usize,
    /// The number of byte used for appended ack confirmation.
    used_appended_ack: usize,

    /// How long since the last outbound packet was explicitly sent.
    last_out: f32,
    /// How long since the last inbound packet was received.
    last_in: f32,

    /// This is updated by sending reliable packet and receiving an ack confirmation.
    ping: f32,
    /// Bytes send in the previous second.
    bps: usize,
    /// Number of packet received in the previous second,
    inbound_pps: usize,

    fragmented_packets: AHashMap<u16, FragmentedPacket>,

    /// Reliable packet we have receive, but still need to confirm to the other peer.
    /// These will be appended automaticaly to outbound packets.
    ack_to_send: Vec<u8>,

    pub stats: ConnectionStats,
    configs: Arc<ConnectionConfigs>,
}
impl Connection {
    pub fn new(
        address: SocketAddr,
        socket: Arc<UdpSocket>,
        configs: Arc<ConnectionConfigs>,
    ) -> (Self, Sender<Vec<u8>>) {
        let (inbound_sender, inbound_receiver) = unbounded();

        (
            Self {
                address,
                socket,
                inbound_receiver,
                current_ack: 0,
                pending_send: Vec::new(),
                pending_ack: AHashMap::new(),
                buffer: Box::new([0; MAX_PACKET_SIZE]),
                used_buffer: 1,
                used_appended_ack: 0,
                last_out: 0.0,
                last_in: 0.0,
                ping: 1.0,
                ack_to_send: Vec::new(),
                stats: Default::default(),
                configs,
                current_fragment: 0,
                fragmented_packets: AHashMap::new(),
                bps: 0,
                inbound_pps: 0,
            },
            inbound_sender,
        )
    }

    fn copy_payload_to_buffer(&mut self, buffer: &[u8]) {
        // Copy payload (ouch!).
        self.used_buffer = HEADER_SIZE + buffer.len();
        self.buffer[HEADER_SIZE..self.used_buffer].copy_from_slice(buffer);
    }

    fn prepare_buffer(&mut self, reliable: bool) {
        // Copy ack request.
        if reliable {
            self.buffer[self.used_buffer..self.used_buffer + 2].copy_from_slice(&self.current_ack.to_be_bytes());
            self.used_buffer += 2;
        }

        // Append some ack confirmation.
        self.append_ack_confirmation_to_buffer();

        // Update header.
        self.buffer[0] |= reliable as u8;
    }

    fn send_buffer(&self, buffer: &[u8]) -> Option<usize> {
        if self.is_bandwidth_saturated() {
            return None;
        }

        self.socket.send_to(&buffer, self.address).ok().and_then(
            |num| {
                if num == buffer.len() {
                    Some(num)
                } else {
                    None
                }
            },
        )
    }

    fn send_internal_buffer(&mut self, reliable: bool) -> bool {
        let send_success = self.send_buffer(&self.buffer[..self.used_buffer + self.used_appended_ack]);

        if let Some(num) = send_success {
            // Packet was sent successfuly.
            self.stats.outbound_packet += 1;
            self.stats.outbound_byte += num as u64;
            self.stats.outbound_ack_confirmantion += (self.used_appended_ack / 2) as u64;
            self.bps += num;

            if reliable {
                self.stats.outbound_reliable_packet += 1;
                self.push_buffer_to_pending_ack();
            }
        } else {
            // Packet was not sent successfuly.
            self.stats.outbound_fail += 1;
            self.retake_appended_ack_confirmation_from_buffer();

            if reliable {
                self.push_buffer_to_pending_send();
            }
        }

        send_success.is_some()
    }

    /// This may block, but typically for a very short time.
    pub fn send(&mut self, buffer: &[u8], reliable: bool) {
        self.last_out = 0.0;

        if reliable {
            self.current_ack = self.current_ack.wrapping_add(1);
        } else if self.is_bandwidth_saturated() {
            // Shortcut to save some copying of data.
            self.stats.outbound_fail += 1;
            return;
        }

        if buffer.len() > MAX_PAYLOAD_SIZE {
            // This is a fragmented packet.
            self.current_fragment = self.current_fragment.wrapping_add(1);
            let num_part = buffer.len() / MAX_PAYLOAD_SIZE + 1;

            for (chunk, fragment_part) in buffer.chunks(MAX_PAYLOAD_SIZE).zip(0u8..) {
                self.copy_payload_to_buffer(chunk);

                // Copy frament components.
                self.buffer[self.used_buffer..self.used_buffer + 2]
                    .copy_from_slice(&self.current_fragment.to_be_bytes());
                self.used_buffer += 2;
                self.buffer[self.used_buffer] = fragment_part;
                self.used_buffer += 1;
                self.buffer[self.used_buffer] = num_part as u8;
                self.used_buffer += 1;

                self.prepare_buffer(reliable);
                self.buffer[0] |= 0b10;
                if self.send_internal_buffer(reliable) {
                    self.stats.outbound_fragment += 1
                } else if !reliable {
                    // There is no point in sending the other parts.
                    break;
                }
            }
        } else {
            self.copy_payload_to_buffer(buffer);
            self.prepare_buffer(reliable);
            self.send_internal_buffer(reliable);
        }
    }

    /// Append a number of ack from `ack_to_send` to the buffer.
    ///
    /// This also reset the header.
    fn append_ack_confirmation_to_buffer(&mut self) {
        self.used_appended_ack = 0;

        if !self.ack_to_send.is_empty() {
            let i = self.ack_to_send.len().saturating_sub(MAX_ACK_PER_PACKET * 2);
            let acks = &self.ack_to_send[i..self.ack_to_send.len()];
            self.used_appended_ack = acks.len();
            self.buffer[self.used_buffer..self.used_buffer + self.used_appended_ack].copy_from_slice(acks);
            drop(acks);
            let new_len = self.ack_to_send.len() - self.used_appended_ack;
            self.ack_to_send.truncate(new_len);
        }

        // Update header.
        self.buffer[0] = ((self.used_appended_ack / 2) as u8) << 2;
    }

    /// Retake acks confirmation that were appended to the buffer.
    ///
    /// This is used when a packet could not be sent.
    fn retake_appended_ack_confirmation_from_buffer(&mut self) {
        if self.used_appended_ack != 0 {
            let acks = &self.buffer[self.used_buffer..self.used_buffer + self.used_appended_ack];
            self.ack_to_send.extend_from_slice(acks);
            self.used_appended_ack = 0;
        }

        // Update header.
        self.buffer[0] &= 0b11;
    }

    fn copy_buffer_to_vec(&self) -> Vec<u8> {
        self.buffer[0..self.used_buffer].to_vec()
    }

    fn push_buffer_to_pending_send(&mut self) {
        self.retake_appended_ack_confirmation_from_buffer();

        self.pending_send.push(self.copy_buffer_to_vec());
    }

    fn push_buffer_to_pending_ack(&mut self) {
        self.pending_ack
            .insert(self.current_ack, (self.copy_buffer_to_vec(), 0.0));
    }

    pub fn is_bandwidth_saturated(&self) -> bool {
        self.bps > self.configs.max_bps
    }

    /// This should idealy be called before sending reliable packets.
    ///
    /// Otherwise reliable packet sent before will be added `delta` in-flight time right away.
    ///
    /// Optimal order is recv -> update -> send
    ///  -
    /// `recv()` will remove packets pending ack that were acked and
    /// add acks that we need to send.
    ///
    /// `update()` will resend reliable packet that did not get ack in time and
    /// increment in-flight by `delta`.
    ///
    /// `send()` will be appended extra acks from reliable packet we have received.
    /// Reliable packet will also be added to the pending ack queue with an in-flight time of 0.
    pub fn update(&mut self, delta: f32) -> Result<(), ConnectionEvent> {
        // Update stats.
        self.bps = self.bps.saturating_sub((self.configs.max_bps as f32 * delta) as usize);
        self.inbound_pps = self
            .inbound_pps
            .saturating_sub((self.configs.max_inbound_pps as f32 * delta) as usize);

        // Move expired pending packet to the send queue.
        let resend_threshold =
            (self.ping * self.configs.expected_in_flight_time_modifier).max(self.configs.min_in_flight_time);
        self.pending_send.extend(
            self.pending_ack
                .drain_filter(|_, (_, in_flight)| {
                    // Increment in-flight time.
                    *in_flight += delta;

                    // Check if packet is over resend threshold.
                    if *in_flight > resend_threshold {
                        self.stats.outbound_unacked += 1;
                        true
                    } else {
                        false
                    }
                })
                .map(|(_, (packet, _))| packet),
        );

        // Resend pending packets.
        while !self.is_bandwidth_saturated() {
            let mut packet = if let Some(packet) = self.pending_send.pop() {
                packet
            } else {
                break;
            };

            println!("sent");

            // Update ack request.
            self.current_ack = self.current_ack.wrapping_add(1);
            let i = packet.len() - 2;
            packet[i..].copy_from_slice(&self.current_ack.to_be_bytes());

            // Resend packet.
            if let Some(num) = self.send_buffer(&packet) {
                // Packet was sent successfuly.
                self.stats.outbound_packet += 1;
                self.stats.outbound_byte += num as u64;
                self.stats.outbound_reliable_packet += 1;
                self.bps += num;
                self.pending_ack.insert(self.current_ack, (packet, 0.0));
            } else {
                // Packet was not sent successfuly.
                self.stats.outbound_fail += 1;
                self.pending_send.push(packet);
            }
        }

        // Return a connection event.
        if self.inbound_pps > self.configs.max_inbound_pps {
            Err(ConnectionEvent::Flood)
        } else if self.last_in > self.configs.timeout_duration || self.last_out > self.configs.timeout_duration {
            Err(ConnectionEvent::Timeout)
        } else {
            Ok(())
        }
    }

    /// Try to receive a packet.
    ///
    /// This will not block waiting for packets.
    pub fn recv(&mut self) -> Result<Payload, ConnectionRecvError> {
        if let Ok(mut packet) = self.inbound_receiver.try_recv() {
            self.stats.inbound_packet += 1;
            self.stats.inbound_byte += packet.len() as u64;
            self.last_in = 0.0;
            self.inbound_pps += 1;

            if let Some((metadata, acks)) = split_packet(&packet) {
                if let Some(ack_request) = metadata.ack_request {
                    self.stats.inbound_reliable_packet += 1;
                    // We will append this ack to future packet send.
                    self.ack_to_send.extend_from_slice(&ack_request);
                }

                // Remove packet pending ack that we just received confirmation.
                self.stats.inbound_ack_confirmation += acks.len() as u64;
                for ack in acks {
                    let ack = u16::from_be_bytes(*ack);
                    if let Some((_, in_flight)) = self.pending_ack.remove(&ack) {
                        // Update ping.
                        self.ping = self.ping.mul_add(0.9, in_flight * 0.1);
                    }
                }

                if let Some((fragment_id, _, _)) = metadata.fragment_data {
                    self.stats.inbound_fragment += 1;
                    let complete = if let Some(fragmented_packet) = self.fragmented_packets.get_mut(&fragment_id) {
                        fragmented_packet.add(metadata, packet)
                    } else {
                        self.fragmented_packets
                            .insert(fragment_id, FragmentedPacket::new(metadata, packet));
                        false
                    };

                    if complete {
                        if let Some(fragmented_packet) = self.fragmented_packets.remove(&fragment_id) {
                            return Ok(Payload::new(fragmented_packet.data));
                        }
                    }
                } else {
                    // Remove appended data from the packet.
                    unsafe { packet.set_len(HEADER_SIZE + metadata.payload_len) };

                    // Return payload wrapper.
                    return Ok(Payload::new(packet));
                }

                // This packet is a fragment.
                Err(ConnectionRecvError::Fragment)
            } else {
                // This packet is corrupted.
                self.stats.inbound_corrupt += 1;
                Err(ConnectionRecvError::Corrupted)
            }
        } else {
            Err(ConnectionRecvError::Empty)
        }
    }

    /// Get a reference to the connection's address.
    pub fn address(&self) -> SocketAddr {
        self.address
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionRecvError {
    Fragment,
    Corrupted,
    Empty,
}

/// To avoid a memcpy, Payload buffer still has the header.
///
/// call `get_slice()` to get only the packet payload.
pub struct Payload {
    buffer: Vec<u8>,
}
impl Payload {
    /// This is only intended to be used on received packet.
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }

    /// Return the packet payload without the header.
    pub fn slice(&self) -> &[u8] {
        &self.buffer[HEADER_SIZE..]
    }
}

struct PacketMetadata {
    payload_len: usize,
    fragment_data: Option<(u16, u8, u8)>,
    ack_request: Option<[u8;2]>,
}

/// Return the metadata of the packet.
fn split_packet(buffer: &[u8]) -> Option<(PacketMetadata, &[[u8; 2]])> {
    if buffer.len() < HEADER_SIZE {
        return None;
    }

    // Parse header.
    let (header, rest) = buffer.split_first().unwrap();
    let reliable = *header & 0b1 != 0;
    let fragmented = *header & 0b10 != 0;
    let num_ack = (*header >> 2) as usize;

    // The amount of byte that are appended to this packet.
    let appended_len = (reliable as usize * 2) + (fragmented as usize * 4) + num_ack * 2;

    let payload_len = if let Some(payload_len) = rest.len().checked_sub(appended_len) {
        payload_len
    } else {
        return None;
    };

    // This is safe as payload_len is just len minus some value.
    let (_, rest) = unsafe { rest.split_at_unchecked(payload_len) };
    if rest.len() != appended_len {
        return None;
    }

    // Get the fragment data.
    let (fragment_data, rest) = if fragmented {
        let (fragment_data, rest) = rest.split_at(4);
        let fragment_id = u16::from_be_bytes([fragment_data[0], fragment_data[1]]);
        let fragment_part = fragment_data[2];
        let fragment_num_part = fragment_data[3];
        (Some((fragment_id, fragment_part, fragment_num_part)), rest)
    } else {
        (None, rest)
    };

    // Get the ack request.
    let (ack_request, rest) = if reliable {
        let (request, rest) = rest.split_array_ref();
        (Some(request.to_owned()), rest)
    } else {
        (None, rest)
    };

    // Get the ack confirmation.
    let acks = unsafe { rest.as_chunks_unchecked() };

    Some((
        PacketMetadata {
            ack_request,
            payload_len,
            fragment_data,
        },
        acks,
    ))
}

struct FragmentedPacket {
    data: Vec<u8>,
    reliable: bool,
    in_flight: f32,
}
impl FragmentedPacket {
    fn new(metadata: PacketMetadata, buffer: Vec<u8>) -> Self {
        todo!()
    }

    fn add(&mut self, metadata: PacketMetadata, buffer: Vec<u8>) -> bool {
        todo!()
    }
}