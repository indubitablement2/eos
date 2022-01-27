pub mod connection;
pub mod inbound_loop;
pub mod packets;

pub const HEADER_SIZE: usize = 1;
pub const MAX_PAYLOAD_SIZE: usize = 1024;
pub const MAX_ACK_PER_PACKET: usize = 8;
/// 1047
pub const MAX_PACKET_SIZE: usize = HEADER_SIZE + MAX_PAYLOAD_SIZE + 4 + 2 + MAX_ACK_PER_PACKET * 2;

#[derive(Debug, Clone, Copy)]
pub struct ConnectionConfigs {
    /// Maximum number of outbound bytes per seconds.
    /// Does not include udp header.
    ///
    /// When reached, unreliable packet will be discarded
    /// and reliable packet will be buffered.
    ///
    /// Default: 30000
    pub max_bps: usize,
    /// Maximum number of inbound packet per seconds before to trigger a disconnect.
    ///
    /// Default: 100
    pub max_inbound_pps: usize,
    /// Number of seconds without receiving or sending any packet to trigger a disconnect.
    ///
    /// Default: 10.0
    pub timeout_duration: f32,
    /// The minimum number of seconds, before a reliable packet is resent.
    ///
    /// Default: 0.2
    pub min_in_flight_time: f32,
    /// Reliable packet resend threshold is
    /// `(ping * expected_in_flight_time_modifier).max(min_in_flight_time)`.
    ///
    /// Lower this if you can not tolerate delay on lost packet at the cost of extra bandwidth.
    ///
    /// Min: 1.2
    ///
    /// Default: 2.0
    pub expected_in_flight_time_modifier: f32,
}
impl Default for ConnectionConfigs {
    fn default() -> Self {
        Self {
            max_bps: 30000,
            max_inbound_pps: 100,
            timeout_duration: 10.0,
            min_in_flight_time: 0.2,
            expected_in_flight_time_modifier: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ConnectionStats {
    /// The total number of packets that were successfully sent,
    /// but not necessarily received.
    pub outbound_packet: u64,
    /// The total number of bytes that were successfully sent,
    /// but not necessarily received.
    pub outbound_byte: u64,
    /// The total number of packets that could not be sent for any reason.
    pub outbound_fail: u64,
    /// The total number of reliable packets that needed to be resent,
    /// because we did not receive an ack in time.
    pub outbound_no_ack: u64,

    /// The total number of packets that were successfully received
    /// and sent to the proper connection.
    pub inbound_packet: u64,
    /// The total number of bytes that were successfully received
    /// and sent to the proper connection.
    pub inbound_byte: u64,
}
