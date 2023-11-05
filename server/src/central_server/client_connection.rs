use super::*;

pub struct ClientConnection {
    outbound: ConnectionOutbound,
    pub token: u64,
}
impl ClientConnection {
    pub fn new(outbound: ConnectionOutbound, token: u64) -> Self {
        Self { outbound, token }
    }

    pub fn send(&self, packet: CentralClientPacket) {
        self.outbound.send(packet);
    }

    pub fn send_raw(&self, msg: tokio_tungstenite::tungstenite::Message) {
        self.outbound.send_raw(msg);
    }
}
