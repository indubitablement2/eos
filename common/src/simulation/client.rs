use super::*;
use std::f32::consts::PI;

/// Client authentification in progress.
pub struct ClientAuth {
    connection: Connection,
}
impl ClientAuth {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn step(&mut self) -> Option<Result<(ClientId, Client), ()>> {
        // TODO: Implement client auth
        Some(Ok((
            ClientId::default(),
            Client::new(self.connection.clone()),
        )))
    }
}

pub struct Client {
    connection: Connection,

    hulls_state: IndexMap<HullId, HullState>,
}
impl Client {
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            hulls_state: Default::default(),
        }
    }

    pub fn step_retain(&mut self, _id: ClientId, sim: &mut Simulation) -> bool {
        while let Some(packet) = self.connection.try_recv::<ClientInbound>() {
            let Ok(packet) = packet else {
                return false;
            };

            match packet {
                ClientInbound::SpawnHull {
                    hull_data_id,
                    position,
                    rotation,
                } => {
                    sim.physics.hulls.insert(HullSave {
                        hull_data_id,
                        position,
                        rotation,
                        ..Default::default()
                    });
                }
            }
        }

        true
    }

    pub fn post_step(&mut self, sim: &mut Simulation) {
        self.hulls_state.sort_unstable_keys();

        let capacity = self
            .hulls_state
            .values()
            .fold(0, |acc, state| acc + state.serialize_size());

        let mut buf = Vec::with_capacity(capacity + 1 + 8);
        bin_encode_into(ClientOutbound::State, &mut buf);
        bin_encode_into(sim.sim_time, &mut buf);

        self.hulls_state
            .retain(|hull_id, state| state.serialize_into_retain(*hull_id, &mut buf));

        self.connection.queue_raw(buf);
        self.connection.flush();
    }

    pub fn hull_update(&mut self, hull_id: HullId, hull: &Hull) {
        self.hulls_state
            .entry(hull_id)
            .or_default()
            .new_update(hull);
    }
}

fn angle_to_i8(angle: f32) -> i8 {
    (angle / PI * i8::MAX as f32).round() as i8
}

fn i8_to_angle(i: i8) -> f32 {
    i as f32 * PI / i8::MAX as f32
}

fn vector_to_i32(v: Vector2<f32>) -> Vector2<i32> {
    vector![v.x.round() as i32, v.y.round() as i32]
}

fn i32_to_vector(v: Vector2<i32>) -> Vector2<f32> {
    vector![v.x as f32, v.y as f32]
}

/// Bitfield:
/// - 0: remove
///     - Doesn't send anything else
/// - 1: is new
///     - hull id
///     - data id
/// - 2: turret data id
///     - Send data id for each turret or none (0) for empty turrets
/// - 3: turret rotation delta
///     - Send rotation delta for each turret which isn't empty
/// - 4: turret ammo
///     - Send ammo for each turret which has any
/// - 5: armor cells
///     - Send armor cells for each cell which has changed (cell_id: u8, cell: u8)
///
/// Always present:
/// - position_delta
/// - rotation_delta
struct HullState {
    hull_data_id: HullDataId,
    position: Vector2<f32>,
    rotation: f32,

    remove: bool,
    is_new: bool,
    position_delta: Vector2<i32>,
    rotation_delta: i8,
}
impl Default for HullState {
    fn default() -> Self {
        Self {
            hull_data_id: Default::default(),
            remove: true,
            position: vector![0.0, 0.0],
            rotation: 0.0,
            is_new: true,
            position_delta: vector![0, 0],
            rotation_delta: 0,
        }
    }
}
impl HullState {
    fn new_update(&mut self, hull: &Hull) {
        self.remove = false;

        self.position_delta = vector_to_i32(hull.position - self.position);
        self.rotation_delta = angle_to_i8(hull.rotation.angle() - self.rotation);
    }

    fn serialize_size(&self) -> usize {
        if self.remove {
            return 1;
        }

        // bitfield
        let mut size = 1;

        if self.is_new {
            // hull_id
            size += 10;
            // hull_data_id
            size += 10;
        }

        // position delta
        size += 5 + 5;
        // rotation delta
        size += 1;

        size
    }

    fn serialize_into_retain(&mut self, hull_id: HullId, mut buf: &mut Vec<u8>) -> bool {
        if self.remove {
            buf.push(0b1);
            return false;
        }
        self.remove = true;

        let bitfield_idx = buf.len();
        buf.push(0);

        if self.is_new {
            buf[bitfield_idx] |= 0b01;
            bin_encode_into(hull_id, &mut buf);
            bin_encode_into(self.hull_data_id, &mut buf);
            self.is_new = false;
        }

        bin_encode_into(self.position_delta, &mut buf);
        bin_encode_into(self.rotation_delta, &mut buf);
        self.position += i32_to_vector(self.position_delta);
        self.rotation += i8_to_angle(self.rotation_delta);

        true
    }
}

#[derive(Serialize)]
enum ClientOutbound {
    State,
}

#[derive(Deserialize)]
enum ClientInbound {
    SpawnHull {
        hull_data_id: HullDataId,
        position: Vector2<f32>,
        rotation: f32,
    },
}
