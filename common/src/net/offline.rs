use super::*;

pub struct OfflineConnection {
    pub disconnected: bool,
    pub inbound: crossbeam::channel::Receiver<ClientPacket>,
    pub outbound: crossbeam::channel::Sender<ServerPacket>,
}
impl Connection for OfflineConnection {
    fn send_reliable(&mut self, packet: &ServerPacket) {
        if self.outbound.send(packet.to_owned()).is_err() {
            self.disconnected = true;
        }
    }

    fn send_unreliable(&mut self, packet: &ServerPacket) {
        self.send_reliable(packet);
    }

    fn recv_packets(&mut self, mut closure: impl FnMut(ClientPacket) -> bool) {
        loop {
            match self.inbound.try_recv() {
                Ok(packet) => {
                    if closure(packet) {
                        return;
                    }
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

pub struct OfflineConnectionsManager {
    pub pending_connection: Option<(Auth, OfflineConnection)>,
}
impl ConnectionsManager for OfflineConnectionsManager {
    type ConnectionType = OfflineConnection;

    fn get_new_login(&mut self, mut closure: impl FnMut(&Auth) -> LoginResponse) -> Option<Self::ConnectionType> {
        if let Some((auth, mut connection)) = self.pending_connection.take() {
            let response = closure(&auth);
            if let LoginResponse::Accepted(_) = &response {
                connection.send_reliable(&ServerPacket::LoginResponse(response));
                Some(connection)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn disconnect(&mut self, _connection: Self::ConnectionType) {}

    fn new(_configs: &ConnectionConfigs, _rt: &tokio::runtime::Runtime) -> Self {
        Self {
            pending_connection: None,
        }
    }
}
