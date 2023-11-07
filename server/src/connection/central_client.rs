use super::*;

#[derive(Debug)]
pub enum CentralClientPacket {
    /// 1
    LoginSuccess { client_id: ClientId, token: u64 },
    /// 5
    /// Received after requesting to join a battlescape.
    /// Return 0 battlescape_id and empty addr to indicate leaving.
    ChangeBattlescape {
        battlescape_id: Option<BattlescapeId>,
        same_addr: bool,
        instance_addr: Option<SocketAddr>,
    },
    /// 10
    /// Receive a message from a client in global chat.
    GlobalMessage {
        from: ClientId,
        channel: u32,
        message: String,
    },
}
impl Packet for CentralClientPacket {
    fn serialize(self) -> Message {
        let mut buf = Vec::new();

        match self {
            CentralClientPacket::LoginSuccess { client_id, token } => {
                buf.reserve_exact(8 + 8 + 12 + 12);
                buf.put_array_var(3);
                buf.put_u32_var(1);
                buf.put_u64_var(client_id.0);
                buf.put_u64_var(token);
            }
            CentralClientPacket::ChangeBattlescape {
                battlescape_id,
                same_addr,
                instance_addr,
            } => {
                let addr = instance_addr
                    .map(|addr| addr.to_string())
                    .unwrap_or_default();
                let battlescape_id = battlescape_id
                    .map(|battlescape_id| battlescape_id.0)
                    .unwrap_or_default();

                buf.reserve_exact(8 + 8 + 12 + 8 + 8 + addr.len().next_multiple_of(4));
                buf.put_array_var(3);
                buf.put_u32_var(5);
                buf.put_u64_var(battlescape_id);
                buf.put_bool_var(same_addr);
                buf.put_string_var(addr.as_str());
            }
            CentralClientPacket::GlobalMessage {
                from,
                channel,
                message,
            } => {
                buf.reserve_exact(8 + 8 + 12 + 8 + 8 + message.len().next_multiple_of(4));
                buf.put_array_var(4);
                buf.put_u32_var(10);
                buf.put_u64_var(from.0);
                buf.put_u32_var(channel);
                buf.put_string_var(&message);
            }
        }

        Message::Binary(buf)
    }

    fn parse(msg: Message) -> Result<Option<Self>, ()> {
        let _ = msg;
        unimplemented!()
    }
}
