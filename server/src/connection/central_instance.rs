use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CentralInstancePacket {
    // /// Send after a client authentication request was received.
    // AuthClientResult {
    //     client_id: ClientId,
    //     token: u64,
    //     success: bool,
    // },
    ClientChangedBattlescape {
        client_id: ClientId,
        token: u64,
        /// None when the client left this instance.
        /// If some, the battlescape is garanteed to be in this instance.
        battlescape_id: Option<BattlescapeId>,
    },
}
impl Packet for CentralInstancePacket {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CentralInstanceLoginResult {
    pub nothing: bool,
}
impl Packet for CentralInstanceLoginResult {
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
