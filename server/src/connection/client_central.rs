use super::*;

#[derive(Debug)]
pub enum ClientCentralPacket {
    /// 10
    GlobalMessage { channel: u32, message: String },
}
impl Packet for ClientCentralPacket {
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
                    10 => Ok(Some(Self::GlobalMessage {
                        channel: buf.get_u32_var()?,
                        message: buf.get_string_var()?,
                    })),
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
pub struct ClientCentralLoginPacket {
    pub new_account: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}
impl Packet for ClientCentralLoginPacket {
    fn serialize(self) -> Message {
        unimplemented!()
    }

    fn parse(msg: Message) -> Result<Option<Self>, ()> {
        match msg {
            Message::Text(_) => Err(()),
            Message::Binary(buf) => {
                let mut buf = buf.as_slice();

                buf.get_array_var()?;

                let new_account = buf.get_bool_var()?;

                let username = buf.get_string_var()?;
                let username = if username.is_empty() {
                    None
                } else {
                    Some(username)
                };
                let password = buf.get_string_var()?;
                let password = if password.is_empty() {
                    None
                } else {
                    Some(password)
                };

                Ok(Some(Self {
                    new_account,
                    username,
                    password,
                }))
            }
            Message::Ping(_) => Ok(None),
            Message::Pong(_) => Ok(None),
            Message::Close(_) => Err(()),
            Message::Frame(_) => Err(()),
        }
    }
}
