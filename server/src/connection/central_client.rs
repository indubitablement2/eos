use super::*;

#[derive(Debug)]
pub enum CentralClientPacket {
    /// N/A
    LoginSuccess { client_id: ClientId, token: u64 },
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
                buf.reserve_exact(32);
                buf.put_array_var(2);
                buf.put_u64_var(client_id.0);
                buf.put_u64_var(token);
            }
            CentralClientPacket::GlobalMessage {
                from,
                channel,
                message,
            } => {
                buf.reserve_exact(44 + message.len().next_multiple_of(4));
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
