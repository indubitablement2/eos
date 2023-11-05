use super::*;

#[derive(Debug)]
pub enum InstanceClientPacket {
    Nothing,
}
impl Packet for InstanceClientPacket {
    fn serialize(self) -> Message {
        match self {
            InstanceClientPacket::Nothing => todo!(),
        }
    }

    fn parse(msg: Message) -> Result<Option<Self>, ()> {
        let _ = msg;
        unimplemented!()
    }
}
