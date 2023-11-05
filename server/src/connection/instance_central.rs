use super::*;

#[derive(Serialize, Deserialize)]
pub enum InstanceCentralPacket {
    AuthClient { client_id: ClientId, token: u64 },
}
impl Packet for InstanceCentralPacket {
    fn serialize(self) -> Message {
        Message::Binary(bincode::serialize(&self).unwrap())
    }

    fn parse(msg: Message) -> Result<Option<Self>, ()> {
        match msg {
            Message::Text(_) => Err(()),
            Message::Binary(buf) => Ok(Some(bincode::deserialize(&buf).unwrap())),
            Message::Ping(_) => Ok(None),
            Message::Pong(_) => Ok(None),
            Message::Close(_) => Err(()),
            Message::Frame(_) => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceCentralLoginPacket {
    pub private_key: u64,
}
impl Packet for InstanceCentralLoginPacket {
    fn serialize(self) -> Message {
        Message::Binary(bincode::serialize(&self).unwrap())
    }

    fn parse(msg: Message) -> Result<Option<Self>, ()> {
        match msg {
            Message::Text(_) => Err(()),
            Message::Binary(buf) => {
                if let Ok(packet) = bincode::deserialize(&buf) {
                    Ok(Some(packet))
                } else {
                    Err(())
                }
            }
            Message::Ping(_) => Ok(None),
            Message::Pong(_) => Ok(None),
            Message::Close(_) => Err(()),
            Message::Frame(_) => Err(()),
        }
    }
}
impl InstanceCentralLoginPacket {
    pub fn new() -> Self {
        Self {
            private_key: PRIVATE_KEY,
        }
    }
}
