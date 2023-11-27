use super::*;
use godot_encoding::*;

enum ClientOutbound {
    Test,
}
impl Packet for ClientOutbound {
    fn serialize(self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            ClientOutbound::Test => {
                buf.put_array_var(1);
                buf.put_u32_var(0);
            }
        }

        buf
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
}
