use super::*;

pub trait ConnectionClientSide {
    fn send_reliable(&mut self, packet: &ClientPacket);
    fn send_unreliable(&mut self, packet: &ClientPacket);
    fn recv_packets(&mut self, closure: impl FnMut(ServerPacket));
    /// Send buffered packets.
    ///
    /// Return if disconnected.
    #[must_use = "this is the only place to detect disconnection"]
    fn flush(&mut self) -> bool;
}

pub trait Connection {
    fn send_reliable(&mut self, packet: &ServerPacket);
    fn send_unreliable(&mut self, packet: &ServerPacket);
    /// Return true to exit early.
    fn recv_packets(&mut self, closure: impl FnMut(ClientPacket) -> bool);
    /// Send buffered packets.
    ///
    /// Return if disconnected.
    #[must_use = "this is the only place to detect disconnection"]
    fn flush(&mut self) -> bool;
}

pub trait ConnectionsManager {
    type ConnectionType: Connection;
    /// Get a new login attempt.
    fn get_new_login(&mut self, closure: impl FnMut(&Auth) -> LoginResponse) -> Option<Self::ConnectionType>;
    fn disconnect(&mut self, connection: Self::ConnectionType);
    fn new(configs: &ConnectionConfigs, rt: &tokio::runtime::Runtime) -> Self;
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ConnectionConfigs {
    /// The port number that will be used for udp and tcp.
    /// 0 to bind to any free port.
    pub port: u16,
    /// Maximum number of pending connections handled per tick.
    pub max_connection_handled_per_update: usize,
    /// How many pendings connections before a queue update is considered.
    pub min_pending_queue_size_for_update: usize,
    /// How many tick does the connection queue  need to be above `min_pending_queue_size_for_update`
    /// before an update is done (sending queue size, checking disconnect).
    pub connection_queue_update_interval: u32,
}
impl Default for ConnectionConfigs {
    fn default() -> Self {
        Self {
            port: 0,
            max_connection_handled_per_update: 32,
            min_pending_queue_size_for_update: 20,
            connection_queue_update_interval: 50,
        }
    }
}
