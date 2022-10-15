use common::net::*;
use metascape::offline_client::*;
use metascape::online_client::*;
use metascape::ConnectionClientSide;

/// Enum that can handle both type of connection.
pub enum ConnectionClientSideWrapper {
    Online(OnlineConnectionClientSide),
    Offline(OfflineConnectionClientSide),
}
impl ConnectionClientSide for ConnectionClientSideWrapper {
    fn send_reliable(&mut self, packet: &ClientPacket) {
        match self {
            ConnectionClientSideWrapper::Online(c) => c.send_reliable(packet),
            ConnectionClientSideWrapper::Offline(c) => c.send_reliable(packet),
        }
    }

    fn send_unreliable(&mut self, packet: &ClientPacket) {
        match self {
            ConnectionClientSideWrapper::Online(c) => c.send_unreliable(packet),
            ConnectionClientSideWrapper::Offline(c) => c.send_unreliable(packet),
        }
    }

    fn recv_packets(&mut self, closure: impl FnMut(ServerPacket)) {
        match self {
            ConnectionClientSideWrapper::Online(c) => c.recv_packets(closure),
            ConnectionClientSideWrapper::Offline(c) => c.recv_packets(closure),
        }
    }

    fn flush(&mut self) -> bool {
        match self {
            ConnectionClientSideWrapper::Online(c) => c.flush(),
            ConnectionClientSideWrapper::Offline(c) => c.flush(),
        }
    }
}
