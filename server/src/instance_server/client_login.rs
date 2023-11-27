use super::*;
use godot_encoding::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientLogin {
    username: String,
    password: String,
    token: u64,
}
impl Packet for ClientLogin {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        let mut buf = buf.as_slice();

        buf.get_array_var()?;

        Ok(Self {
            username: buf.get_string_var()?,
            password: buf.get_string_var()?,
            token: buf.get_u64_var()?,
        })
    }
}
