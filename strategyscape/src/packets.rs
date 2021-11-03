use rapier2d::na::Vector2;

pub enum Packet {
    /// Server will respond to those with another ping.
    Ping,
    /// Client send his desired input.
    ClientInput {
        /// The angle of the capital ship wish direction.
        /// 16 bits.
        wish_dir: f32,
        /// The absolute force of the capital ship wish direction.
        /// 8 bits.
        wish_dir_force: f32,
        /// Toggle firing selected weapon group.
        /// 1 bit.
        fire_toggle: bool,
        /// The angle of the capital ship's selected weapons wish direction.
        /// 16 bits.
        aim_weapons: f32,
    },

    /// TODO: Client inform the server of what happened after a tick.
    ClientTickResult { tick: u64, capital_tranlation: Vector2<f32> },

    /// TODO: Server commands.
    ServerCommand { tick: u64 },

    /// TODO: A chat message.
    Message { channel: u8, message: String },
}
impl Packet {
    pub const PING_ID: u8 = 0;
    pub const PING_HEADER: u16 = (Packet::PING_ID as u16) << 13;

    pub const CLIENT_INPUT_ID: u8 = 1;
    pub const CLIENT_INPUT_HEADER: u16 = (Packet::CLIENT_INPUT_ID as u16) << 13;

    pub const CLIENT_TICK_RESULT_PACKET_ID: u8 = 2;
    pub const HEADER_CLIENT_TICK_RESULT_HEADER: u16 = (Packet::CLIENT_TICK_RESULT_PACKET_ID as u16) << 13;

    pub const SERVER_COMMAND_ID: u8 = 3;
    pub const SERVER_COMMAND_HEADER: u16 = (Packet::SERVER_COMMAND_ID as u16) << 13;

    pub const MESSAGE_ID: u8 = 4;
    pub const MESSAGE_HEADER: u16 = (Packet::MESSAGE_ID as u16) << 13;

    pub const MAX_PACKET_SIZE: u16 = 1500;

    /// Cast a Packet to an array of bytes ready to be sent over the network.
    pub fn to_buffer(&self) -> Vec<u8> {
        match self {
            Packet::Ping => {
                vec![0, 0]
            }
            Packet::ClientInput {
                wish_dir,
                wish_dir_force,
                fire_toggle,
                aim_weapons,
            } => {
                let wish_dir = (wish_dir * u16::MAX as f32) as u16;
                let wish_dir_force = (wish_dir_force * u8::MAX as f32) as u8;
                let fire_toggle = *fire_toggle as u8;
                let aim_weapons = (aim_weapons * u16::MAX as f32) as u16;

                let header = Packet::CLIENT_INPUT_HEADER + 6;

                let mut v: Vec<u8> = Vec::with_capacity(2 + 6);
                v.extend(header.to_be_bytes().into_iter());
                v.extend(wish_dir.to_be_bytes().into_iter());
                v.push(wish_dir_force);
                v.push(fire_toggle);
                v.extend(aim_weapons.to_be_bytes().into_iter());

                v
            }
            Packet::ClientTickResult {
                tick,
                capital_tranlation,
            } => todo!(),
            Packet::ServerCommand { tick } => todo!(),
            Packet::Message { channel, message } => todo!(),
        }
    }

    /// Parse a packet from a buffer.
    fn parse_client_input(buffer: &[u8]) -> Result<(Self, usize), PacketSerializationError> {
        if buffer.len() < 8 {
            return Err(PacketSerializationError::MissingBytes(8));
        }

        let wish_dir = u16::from_be_bytes([buffer[2], buffer[3]]) as f32 / u16::MAX as f32;
        let wish_dir_force = buffer[4] as f32 / u8::MAX as f32;
        let fire_toggle = buffer[5] != 0;
        let aim_weapons = u16::from_be_bytes([buffer[6], buffer[7]]) as f32 / u16::MAX as f32;

        Ok((
            Self::ClientInput {
                wish_dir,
                wish_dir_force,
                fire_toggle,
                aim_weapons,
            },
            8,
        ))
    }

    /// You can send any buffer to this function and if it can't deserialize a Packet, it will return the amount of bytes needed.
    ///
    /// Err(0) means no header were found.
    ///
    /// On successful deserialization, it also return the number of bytes used (including header's 2 bytes).
    pub fn from_buffer(buffer: &[u8]) -> Result<(Self, usize), PacketSerializationError> {
        // Check for a header.
        if buffer.len() < 2 {
            return Err(PacketSerializationError::NoHeader);
        }

        // Parse header.
        let packet_type = buffer[0] >> 5;
        let payload_size = buffer[1] as usize + (((buffer[0] & 0b00011111) as usize) << 8);

        // Check if we have anough bytes.
        if buffer.len() < payload_size + 2 {
            return Err(PacketSerializationError::MissingBytes(payload_size + 2));
        }

        match packet_type {
            Packet::PING_ID => Ok((Packet::Ping, 2)),
            Packet::CLIENT_INPUT_ID => Packet::parse_client_input(buffer),
            // TODO: ADD the rest.
            _ => Err(PacketSerializationError::UnknowPacket),
        }
    }
}

pub enum PacketSerializationError {
    /// Too short to parse a header.
    NoHeader,
    /// Buffer needs to be at least this size.
    MissingBytes(usize),
    /// Can not deserialize. Unkow packet type or gibberish. Should probably ignore this address.
    UnknowPacket,
}
