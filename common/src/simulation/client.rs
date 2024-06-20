use super::*;
use std::f32::consts::PI;

const EXPERIENCE_PER_LEVEL: [u64; 100] = [
    100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700,
    1800, 1900, 2000, 2100, 2200, 2300, 2400, 2500, 2600, 2700, 2800, 2900, 3000, 3100, 3200, 3300,
    3400, 3500, 3600, 3700, 3800, 3900, 4000, 4100, 4200, 4300, 4400, 4500, 4600, 4700, 4800, 4900,
    5000, 5100, 5200, 5300, 5400, 5500, 5600, 5700, 5800, 5900, 6000, 6100, 6200, 6300, 6400, 6500,
    6600, 6700, 6800, 6900, 7000, 7100, 7200, 7300, 7400, 7500, 7600, 7700, 7800, 7900, 8000, 8100,
    8200, 8300, 8400, 8500, 8600, 8700, 8800, 8900, 9000, 9100, 9200, 9300, 9400, 9500, 9600, 9700,
    9800, 9900, 10000,
];

pub struct Client {
    connection: Connection,

    pub char_ref: CharacterRef,
    experience: u64,
    next_level: Option<u64>,
}
impl Client {
    pub fn new(
        connection: Connection,
        client_save: Option<Vec<u8>>,
        league_save: Option<Vec<u8>>,
        player_save: Option<Vec<u8>>,
    ) -> Self {
        let client_save: ClientSaveParsed = if let Some(save) = client_save {
            bin_decode::<ClientSaveVersionned>(&save)
                .unwrap_or_default()
                .into()
        } else {
            Default::default()
        };

        let league_save: LeagueSaveParsed = if let Some(save) = league_save {
            bin_decode::<LeagueSaveVersionned>(&save)
                .unwrap_or_default()
                .into()
        } else {
            Default::default()
        };

        let player_save: PlayerSaveParsed = if let Some(save) = player_save {
            bin_decode::<PlayerSaveVersionned>(&save)
                .unwrap_or_default()
                .into()
        } else {
            Default::default()
        };
        let experience = player_save
            .experience
            .min(EXPERIENCE_PER_LEVEL.last().copied().unwrap());
        let level = EXPERIENCE_PER_LEVEL
            .iter()
            .position(|&experience_required| experience_required > experience)
            .unwrap_or(100);
        let next_level = EXPERIENCE_PER_LEVEL.get(level).copied();

        let char_ref = Character::new(
            Default::default(),
            player_save.health_relative,
            level as u32,
        );

        Self {
            connection,
            char_ref,
            experience,
            next_level,
        }
    }

    pub fn step(&mut self, _id: ClientId, sim: &mut Simulation) {
        while let Some(packet) = self.connection.try_recv::<ClientInbound>() {
            match packet {
                ClientInbound::SpawnHull {
                    hull_data_id,
                    position,
                    rotation,
                } => {}
            }
        }
    }

    pub fn post_step_retain(&mut self, client_id: ClientId, sim: &mut Simulation) -> bool {
        let Some(mut char) = self.char_ref.get_mut() else {
            log::error!("Client {:?} has no character.", client_id);
            return false;
        };

        // Check for player level up.
        if let Some(experience_required) = self.next_level {
            if self.experience >= experience_required {
                let new_level = (char.level() + 1).min(EXPERIENCE_PER_LEVEL.len() as u32);
                char.set_level(new_level);

                // TODO: Finish this.
            }
        }

        let mut bitfield = 0u8;
        true
    }
}

fn angle_to_i32(angle: f32) -> i32 {
    (angle / PI * 1024.0) as i32
}

fn i32_to_angle(i: i32) -> f32 {
    i as f32 * PI / 1024.0
}

fn vector_to_i32(v: Vec2) -> IVec2 {
    (v * 8.0).as_ivec2()
}

fn i32_to_vector(v: IVec2) -> Vec2 {
    v.as_vec2() / 8.0
}

/// Bitfield:
/// - 0: remove
///     - Doesn't send anything else
/// - 1: is new
///     - unsigned: entity data id
/// - 2: position changed
///     - ivec2: position delta from last position
/// - 3: animation changed
///     - unsigned: animation idx
/// - 4: health approx changed
///     - byte: health approx
/// - 5: hits
///     - unsigned: num hits
///         - byte: damage type bitfield
///         - unsigned: damage amount
/// - 6: dot damage
///     - unsigned: damage amount
/// - 7: effects changed
///    - unsigned: num effects
///       - unsigned: effect id
struct HullState {
    last_position: Vec2,
    last_sprite_idx: u32,
}
impl Default for HullState {
    fn default() -> Self {
        Self {
            last_position: Default::default(),
            last_sprite_idx: u32::MAX,
        }
    }
}
impl HullState {
    // fn serialize_into_retain(&mut self, hull_id: EntityId, mut buf: &mut Vec<u8>) -> bool {
    //     if self.remove {
    //         buf.push(0b1);
    //         return false;
    //     }
    //     self.remove = true;

    //     let bitfield_idx = buf.len();
    //     buf.push(0);

    //     if self.is_new {
    //         buf[bitfield_idx] |= 0b10;
    //         bin_encode_into(self.hull_data_id, &mut buf);
    //         bin_encode_into(hull_id, &mut buf);
    //         self.is_new = false;
    //     }

    //     bin_encode_into(self.position_delta, &mut buf);
    //     bin_encode_into(self.rotation_delta, &mut buf);
    //     self.position += i32_to_vector(self.position_delta);
    //     self.rotation += i32_to_angle(self.rotation_delta);

    //     true
    // }
}

#[derive(Serialize)]
enum ClientOutbound {
    State,
}

#[derive(Deserialize)]
enum ClientInbound {
    SpawnHull {
        hull_data_id: CharacterDataId,
        position: Vec2,
        rotation: f32,
    },
}

// ####################################################################################
// ################################### CLIENT SAVE ####################################
// ####################################################################################

struct ClientSaveParsed {}
impl Default for ClientSaveParsed {
    fn default() -> Self {
        Self {}
    }
}
impl From<ClientSaveVersionned> for ClientSaveParsed {
    fn from(value: ClientSaveVersionned) -> Self {
        match value {
            ClientSaveVersionned::V1 => Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
enum ClientSaveVersionned {
    #[default]
    V1,
}
impl From<ClientSaveParsed> for ClientSaveVersionned {
    fn from(value: ClientSaveParsed) -> Self {
        Self::V1
    }
}

// ####################################################################################
// ################################### LEAGUE SAVE ####################################
// ####################################################################################

struct LeagueSaveParsed {}
impl Default for LeagueSaveParsed {
    fn default() -> Self {
        Self {}
    }
}
impl From<LeagueSaveVersionned> for LeagueSaveParsed {
    fn from(value: LeagueSaveVersionned) -> Self {
        match value {
            LeagueSaveVersionned::V1 => Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
enum LeagueSaveVersionned {
    #[default]
    V1,
}
impl From<LeagueSaveParsed> for LeagueSaveVersionned {
    fn from(value: LeagueSaveParsed) -> Self {
        Self::V1
    }
}

// ####################################################################################
// ################################### PLAYER SAVE ####################################
// ####################################################################################

struct PlayerSaveParsed {
    health_relative: f32,
    experience: u64,
}
impl Default for PlayerSaveParsed {
    fn default() -> Self {
        Self {
            health_relative: 1.0,
            experience: 0,
        }
    }
}
impl From<PlayerSaveVersionned> for PlayerSaveParsed {
    fn from(value: PlayerSaveVersionned) -> Self {
        match value {
            PlayerSaveVersionned::V1 => Self {
                health_relative: 1.0,
                experience: 0,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
enum PlayerSaveVersionned {
    #[default]
    V1,
}
impl From<PlayerSaveParsed> for PlayerSaveVersionned {
    fn from(value: PlayerSaveParsed) -> Self {
        Self::V1
    }
}
