use super::{offline::OfflineConnection, *};

pub struct OfflineConnectionClientSide {
    disconnected: bool,
    inbound: crossbeam::channel::Receiver<ServerPacket>,
    outbound: crossbeam::channel::Sender<ClientPacket>,
}
impl OfflineConnectionClientSide {
    /// Return connected connection.
    /// `OfflineConnection` is to be directly inserted into the server metascape.
    pub fn new() -> (Self, OfflineConnection) {
        let (outbound_client, inbound_server) = crossbeam::channel::unbounded();
        let (outbound_server, inbound_client) = crossbeam::channel::unbounded();

        (
            Self {
                disconnected: false,
                inbound: inbound_client,
                outbound: outbound_client,
            },
            OfflineConnection {
                disconnected: false,
                inbound: inbound_server,
                outbound: outbound_server,
            },
        )
    }
}
impl ConnectionClientSide for OfflineConnectionClientSide {
    fn send_reliable(&mut self, packet: &ClientPacket) {
        if self.outbound.send(packet.to_owned()).is_err() {
            self.disconnected = true;
        }
    }

    fn send_unreliable(&mut self, packet: &ClientPacket) {
        self.send_reliable(packet);
    }

    fn recv_packets(&mut self, mut closure: impl FnMut(ServerPacket)) {
        loop {
            match self.inbound.try_recv() {
                Ok(packet) => {
                    closure(packet);
                }
                Err(err) => {
                    if err.is_disconnected() {
                        self.disconnected = true;
                    }
                    break;
                }
            }
        }
    }

    fn flush(&mut self) -> bool {
        self.disconnected
    }
}
