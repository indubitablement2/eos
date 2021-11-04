pub const MAX_DATAGRAM_SIZE: usize = 512;

/// Sent to the server over Udp ~60 times per seconds.
pub struct UdpClient {
    /// The angle of the capital ship wish direction.
    /// u16.
    pub wish_dir: f32,
    /// The angle of the capital ship's selected weapons wish direction.
    /// u16.
    pub aim_dir: f32,
    /// The absolute force of the capital ship wish direction.
    /// u8.
    pub wish_dir_force: f32,
    /// Toggle firing selected weapon group.
    /// 1 bit.
    pub fire_toggle: bool,
    // Unused.
    // 7b
}
impl UdpClient {
    pub const PAYLOAD_SIZE: usize = 6;

    /// Serialize into a buffer ready to be sent over Udp.
    pub fn serialize(&self) -> [u8; UdpClient::PAYLOAD_SIZE] {
        let mut payload = [0u8; UdpClient::PAYLOAD_SIZE];

        let wish_dir = ((self.wish_dir * u16::MAX as f32) as u16).to_be_bytes();
        payload[0] = wish_dir[0];
        payload[1] = wish_dir[1];

        let aim_dir = ((self.aim_dir * u16::MAX as f32) as u16).to_be_bytes();
        payload[2] = aim_dir[0];
        payload[3] = aim_dir[1];

        let wish_dir_force = (self.wish_dir_force * u8::MAX as f32) as u8;
        payload[4] = wish_dir_force;

        let fire_toggle = (self.fire_toggle as u8) << 7;
        payload[5] += fire_toggle;

        payload
    }

    /// Deserialize from a buffer received from Udp.
    /// # Safety
    /// buffer should be of size `UdpClient::PAYLOAD_SIZE`
    pub fn deserialize(buffer: &[u8]) -> Self {
        let wish_dir = u16::from_be_bytes([buffer[0], buffer[1]]) as f32 / u16::MAX as f32;

        let aim_dir = u16::from_be_bytes([buffer[2], buffer[3]]) as f32 / u16::MAX as f32;

        let wish_dir_force = buffer[4] as f32 / u8::MAX as f32;

        let fire_toggle = (buffer[5] & 0b10000000) != 0;

        Self {
            wish_dir,
            aim_dir,
            wish_dir_force,
            fire_toggle,
        }
    }
}

/// Server send this to clients over Udp ~10 times per seconds.
pub struct UdpServer {
    tick: u16,
    player_inputs: Vec<[u8; UdpClient::PAYLOAD_SIZE]>,
}
impl UdpServer {
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(2 + UdpClient::PAYLOAD_SIZE);

        // Add tick number.
        payload.extend_from_slice(&self.tick.to_be_bytes());

        // Add each player input.
        for i in &self.player_inputs {
            payload.extend_from_slice(i);
        }

        payload
    }
}

pub struct UdpServerDeserialized {
    pub tick: u16,
    pub player_inputs: Vec<UdpClient>,
}
impl UdpServerDeserialized {
    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        if (buffer.len() < UdpClient::PAYLOAD_SIZE + 2) || (buffer.len() - 2) % UdpClient::PAYLOAD_SIZE != 0 {
            return None;
        }

        let tick = u16::from_be_bytes([buffer[0], buffer[1]]);

        let player_inputs = buffer[2..]
            .chunks_exact(UdpClient::PAYLOAD_SIZE)
            .map(|chunk| UdpClient::deserialize(chunk))
            .collect();

        Some(Self { tick, player_inputs })
    }
}
