use super::*;

#[derive(Debug)]
pub enum ClientInstancePacket {
    Test,
}
impl Packet for ClientInstancePacket {
    fn serialize(self) -> Message {
        unimplemented!()
    }

    fn parse(msg: Message) -> Result<Option<Self>, ()> {
        match msg {
            Message::Text(_) => Err(()),
            Message::Binary(buf) => {
                let mut buf = buf.as_slice();

                buf.get_array_var()?;
                let packet_id = buf.get_u32_var()?;

                match packet_id {
                    _ => {
                        log::debug!("Invalid packet id {}", packet_id);
                        Err(())
                    }
                }
            }
            Message::Ping(_) => Ok(None),
            Message::Pong(_) => Ok(None),
            Message::Close(_) => Err(()),
            Message::Frame(_) => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct ClientInstanceLoginPacket {
    pub client_id: ClientId,
    pub token: u64,
}
impl Packet for ClientInstanceLoginPacket {
    fn serialize(self) -> Message {
        unimplemented!()
    }

    fn parse(msg: Message) -> Result<Option<Self>, ()> {
        match msg {
            Message::Text(_) => Err(()),
            Message::Binary(buf) => {
                let mut buf = buf.as_slice();
                buf.get_array_var()?;
                Ok(Some(Self {
                    client_id: ClientId(buf.get_u64_var()?),
                    token: buf.get_u64_var()?,
                }))
            }
            Message::Ping(_) => Ok(None),
            Message::Pong(_) => Ok(None),
            Message::Close(_) => Err(()),
            Message::Frame(_) => Err(()),
        }
    }
}
