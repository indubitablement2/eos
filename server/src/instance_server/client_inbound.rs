use super::*;
use godot_encoding::*;

enum ClientInbound {
    Test,
}
impl Packet for ClientInbound {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        let mut buf = buf.as_slice();

        buf.get_array_var()?;
        let packet_id = buf.get_u32_var()?;

        Ok(match packet_id {
            0 => ClientInbound::Test,
            _ => anyhow::bail!("Invalid packet id: {}", packet_id),
        })
    }
}
