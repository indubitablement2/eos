use super::*;
use ahash::AHashMap;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionEvent {
    /// We or the client did not sent any packet for some time.
    Timeout,
    /// Connected peer is sending too many packets.
    Flood,
}

pub trait PacketHeader {
    const HEADER_ACK_REQUEST_MASK: u8 = 0b1;
    const HEADER_FRAGMENTED_MASK: u8 = 0b10;
    const HEADER_NUM_APPENDED_ACK_MASK: u8 = 0b11100;

    fn reset(&mut self);
    fn ack_request(&self) -> bool;
    fn set_ack_request(&mut self, v: bool);
    fn fragmented(&self) -> bool;
    fn set_fragmented(&mut self, v: bool);
    fn num_appended_ack(&self) -> u8;
    fn set_num_appended_ack(&mut self, v: u8);
}
impl PacketHeader for u8 {
    fn reset(&mut self) {
        *self = 0;
    }

    fn ack_request(&self) -> bool {
        self & Self::HEADER_ACK_REQUEST_MASK != 0
    }

    fn set_ack_request(&mut self, v: bool) {
        if v {
            *self |= Self::HEADER_ACK_REQUEST_MASK;
        } else {
            *self &= !Self::HEADER_ACK_REQUEST_MASK;
        }
    }

    fn fragmented(&self) -> bool {
        self & Self::HEADER_FRAGMENTED_MASK != 0
    }

    fn set_fragmented(&mut self, v: bool) {
        if v {
            *self |= Self::HEADER_FRAGMENTED_MASK;
        } else {
            *self &= !Self::HEADER_FRAGMENTED_MASK;
        }
    }

    fn num_appended_ack(&self) -> u8 {
        (*self & Self::HEADER_NUM_APPENDED_ACK_MASK) >> 2
    }

    fn set_num_appended_ack(&mut self, v: u8) {
        *self &= !Self::HEADER_NUM_APPENDED_ACK_MASK;
        *self |= v << 2;
    }
}

#[derive(Debug, Clone)]
struct PacketBuffer {
    data: Vec<u8>,
}
impl PacketBuffer {
    pub fn reset(&mut self)  {
        self.data.truncate(1);
        self.header_mut().unwrap().reset();
    }

    /// Return the packet payload without the header.
    pub fn slice(&self) -> &[u8] {
        &self.data[HEADER_SIZE..]
    }

    pub fn header(&self) -> Option<&u8> {
        self.data.first()
    }

    pub fn header_mut(&mut self) -> Option<&mut u8> {
        self.data.first_mut()
    }

    fn append_fragment_components(&mut self, fragment_id: u16, part: u8, num_part: u8) {
        debug_assert!(!self.header().unwrap().ack_request());
        debug_assert!(!self.header().unwrap().fragmented());
        debug_assert_eq!(self.header().unwrap().num_appended_ack(), 0);

        if let Some(header) = self.header_mut() {
            header.set_fragmented(true);
        }

        self.data.extend_from_slice(&fragment_id.to_be_bytes());
        self.data.push(part);
        self.data.push(num_part);
    }

    fn append_ack_request(&mut self, ack: u16) {
        debug_assert_eq!(self.header().unwrap().num_appended_ack(), 0);

        if let Some(header) = self.header_mut() {
            header.set_ack_request(true);
        }

        self.data.extend_from_slice(&ack.to_be_bytes());
    }

    fn append_acks(&mut self, acks: &[u8]) {
        debug_assert_eq!(self.header().unwrap().num_appended_ack(), 0);

        if let Some(header) = self.header_mut() {
            header.set_num_appended_ack(acks.len() as u8 / 2);
        }

        self.data.extend_from_slice(acks);
    }

    fn remove_appended_acks(&mut self) -> Vec<u8> {
        let num = if let Some(header) = self.header_mut() {
            let num = header.num_appended_ack() as usize;
            header.set_num_appended_ack(0);
            num
        } else {
            0
        };

        let new_len = self.data.len() - num;
        self.data.split_off(new_len)
    }

    fn truncate_acks(&mut self, old_len: usize) {
        self.data.truncate(old_len);
        self.header_mut().unwrap().set_num_appended_ack(0);
    }

    /// Return the metadata of the packet.
    fn split_packet(&self) -> Option<PacketMetadata> {
        if self.data.len() < HEADER_SIZE {
            return None;
        }

        // Parse header.
        let (header, rest) = self.data.split_first().unwrap();
        let reliable = header.ack_request();
        let fragmented = header.fragmented();
        let num_ack = header.num_appended_ack() as usize;

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
        let acks = acks.iter().map(|v| u16::from_be_bytes(*v)).collect();

        Some(
            PacketMetadata {
                ack_request,
                payload_len,
                fragment_data,
                acks,
            }
        )
    }
}
impl Default for PacketBuffer {
    fn default() -> Self {
        let mut data = Vec::with_capacity(MAX_PACKET_SIZE);
        data.push(0);
        Self { data }
    }
}

/// Header - 1
/// -
/// 0..1 - has ack request bool
///
/// 1..2 - fragmented bool
///
/// 2..5 - num appended ack
///
/// 5..8 - unused
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
    pending_send: Vec<PacketBuffer>,
    /// Reliable packets that were sent, but are awaiting an ack confirmantion.
    pending_ack: AHashMap<u16, (PacketBuffer, f32)>,

    buffer: PacketBuffer,

    /// How long since the last outbound packet was explicitly sent.
    last_out: f32,
    /// How long since the last inbound packet was received.
    last_in: f32,

    /// This is updated by sending reliable packet and receiving an ack confirmation.
    ping: f32,
    /// Bytes send in the previous second.
    bps: usize,
    /// Number of packet received in the previous second.
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
                pending_ack: Default::default(),
                buffer: Default::default(),
                last_out: 0.0,
                last_in: 0.0,
                ping: 1.0,
                ack_to_send: Default::default(),
                stats: Default::default(),
                configs,
                current_fragment: 0,
                fragmented_packets: Default::default(),
                bps: 0,
                inbound_pps: 0,
            },
            inbound_sender,
        )
    }

    fn get_ack_confirmation(&mut self) -> Vec<u8> {
        let at = self.ack_to_send.len().saturating_sub(MAX_ACK_PER_PACKET * 2);
        self.ack_to_send.split_off(at)
    }

    fn send_buffer(&mut self, buffer: &[u8], reliable: bool, fragmented: bool, num_acks: u64) -> bool {
        if self.is_bandwidth_saturated() {
            self.stats.outbound_fail += 1;

            return false;
        }

        if let Ok(num) = self.socket.send_to(buffer, self.address) {
            debug_assert_eq!(num, buffer.len());

            // Packet was sent successfuly.
            self.stats.outbound_packet += 1;
            self.stats.outbound_byte += num as u64;
            self.stats.outbound_ack_confirmantion += num_acks;
            self.bps += num;

            if fragmented {
                self.stats.outbound_fragment += 1;
            }

            if reliable {
                self.stats.outbound_reliable_packet += 1;
            }

            true
        } else {
            // Packet was not sent successfuly.
            self.stats.outbound_fail += 1;

            false
        }
    }

    /// This may block, but typically for a very short time.
    pub fn send<T: ?Sized>(&mut self, value: &T, reliable: bool)
    where T: serde::Serialize
    {
        self.last_out = 0.0;

        // Reset the buffer.
        self.buffer.reset();

        // Serialize the packet.
        if bincode::serialize_into(&mut self.buffer.data, value).is_err() {
            error!("Can not serialize packet. Ignoring...");
            return;
        }

        let num_part = (self.buffer.data.len() - 1) / MAX_PAYLOAD_SIZE + 1;

        if num_part > 1 {
            // This is a fragmented packet.
            self.current_fragment = self.current_fragment.wrapping_add(1);

            // Create a new buffer to old the indibidual parts.
            let mut large_buffer = std::mem::take(&mut self.buffer);

            for (chunk, part) in large_buffer.data[1..].chunks(MAX_PAYLOAD_SIZE).zip(0u8..) {
                // Copy payload chunk to the new buffer.
                self.buffer.data.extend_from_slice(chunk);

                // Copy fragment components.
                self.buffer.append_fragment_components(self.current_fragment, part, num_part as u8);

                if reliable {
                    self.current_ack = self.current_ack.wrapping_add(1);
                    self.buffer.append_ack_request(self.current_ack);
                }

                let acks = self.get_ack_confirmation();
                let old_len = self.buffer.data.len();
                self.buffer.append_acks(&acks);

                // Send part.
                if self.send_buffer(&self.buffer.data, reliable, false, acks.len() as u64) {
                    if reliable {
                        self.buffer.truncate_acks(old_len);
                        let packet = std::mem::take(&mut self.buffer);
                        self.pending_ack.insert(self.current_ack, (packet, 0.0));
                    }
                } else {
                    if reliable {
                        self.buffer.truncate_acks(old_len);
                        self.ack_to_send.extend_from_slice(&acks);
                        let packet = std::mem::take(&mut self.buffer);
                        self.pending_send.push(packet);
                    } else {
                        // There is no point in sending the other parts.
                        break;
                    }
                }

                // Reset buffer.
                self.buffer.reset();
            }
        } else {
            if reliable {
                self.current_ack = self.current_ack.wrapping_add(1);
                self.buffer.append_ack_request(self.current_ack);
            }

            let acks = self.get_ack_confirmation();
            let old_len = self.buffer.data.len();
            self.buffer.append_acks(&acks);

            if self.send_buffer(&self.buffer.data, reliable, false, acks.len() as u64) {
                if reliable {
                    self.buffer.truncate_acks(old_len);
                    let packet = std::mem::take(&mut self.buffer);
                    self.pending_ack.insert(self.current_ack, (packet, 0.0));
                }
            } else {
                if reliable {
                    self.buffer.truncate_acks(old_len);
                    self.ack_to_send.extend_from_slice(&acks);
                    let packet = std::mem::take(&mut self.buffer);
                    self.pending_send.push(packet);
                }
            }
        }
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

            // Update ack request.
            self.current_ack = self.current_ack.wrapping_add(1);
            let i = packet.data.len() - 2;
            packet.data[i..].copy_from_slice(&self.current_ack.to_be_bytes());

            let acks = self.get_ack_confirmation();
            let old_len = self.buffer.data.len();
            self.buffer.append_acks(&acks);

            // Resend packet.
            if self.send_buffer(&packet.data, true, packet.header().unwrap().fragmented(), acks.len() as u64) {
                if reliable {
                    self.buffer.truncate_acks(old_len);
                    let packet = std::mem::take(&mut self.buffer);
                    self.pending_ack.insert(self.current_ack, (packet, 0.0));
                }
            } else {
                if reliable {
                    self.buffer.truncate_acks(old_len);
                    self.ack_to_send.extend_from_slice(&acks);
                    let packet = std::mem::take(&mut self.buffer);
                    self.pending_send.push(packet);
                }
            }

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
    acks: Vec<u16>,
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
