use super::*;
use std::f32::consts::PI;

pub struct Client {
    connection: Connection,

    hulls_state: IndexMap<HullId, HullState>,
}
impl Client {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            hulls_state: Default::default(),
        }
    }

    pub fn step(&mut self, _id: ClientId, sim: &mut Simulation) {
        while let Some(packet) = self.connection.try_recv::<ClientInbound>() {
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
    }

    pub fn post_step_retain(&mut self, sim: &mut Simulation) -> bool {
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

        self.connection.is_closed()
    }

    pub fn hull_update(&mut self, hull_id: HullId, hull: &Hull) {
        self.hulls_state
            .entry(hull_id)
            .or_default()
            .new_update(hull);
    }
}

fn angle_to_i32(angle: f32) -> i32 {
    (angle / PI * 512.0).round() as i32
}

fn i32_to_angle(i: i32) -> f32 {
    i as f32 * PI / 512.0
}

fn vector_to_i32(v: Vec2) -> IVec2 {
    IVec2::new(v.x.round() as i32, v.y.round() as i32)
}

fn i32_to_vector(v: IVec2) -> Vec2 {
    Vec2::new(v.x as f32, v.y as f32)
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
///
/// Always present:
/// - position_delta
/// - rotation_delta
struct HullState {
    hull_data_id: HullDataId,
    position: Vec2,
    rotation: f32,

    remove: bool,
    is_new: bool,
    position_delta: IVec2,
    rotation_delta: i32,
}
impl Default for HullState {
    fn default() -> Self {
        Self {
            hull_data_id: Default::default(),
            remove: true,
            position: Vec2::ZERO,
            rotation: 0.0,
            is_new: true,
            position_delta: IVec2::ZERO,
            rotation_delta: 0,
        }
    }
}
impl HullState {
    fn new_update(&mut self, hull: &Hull) {
        self.remove = false;

        self.position_delta = vector_to_i32(hull.position - self.position);
        self.rotation_delta = angle_to_i32(self.rotation.angle_to(hull.rotation));
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
        self.rotation += i32_to_angle(self.rotation_delta);

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
        position: Vec2,
        rotation: f32,
    },
}
