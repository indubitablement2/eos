use super::*;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum BattlescapeMode {
    #[default]
    FFA,
}
impl BattlescapeMode {
    pub const fn spawn_points(self, team: u32) -> &'static [SpawnPoint] {
        match self {
            BattlescapeMode::FFA => match team {
                0 => FFA_0,
                1 => FFA_1,
                2 => FFA_2,
                _ => FFA_3,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpawnPoint {
    pub spawn_position: na::Vector2<f32>,
    pub spawn_direction_angle: f32,
}

const UP: f32 = 0.0;
const RIGHT: f32 = FRAC_PI_2;
const DOWN: f32 = PI;
const LEFT: f32 = -FRAC_PI_2;

const FFA_0: &[SpawnPoint] = &[SpawnPoint {
    spawn_position: na::Vector2::new(0.0, -1.0),
    spawn_direction_angle: UP,
}];
const FFA_1: &[SpawnPoint] = &[SpawnPoint {
    spawn_position: na::Vector2::new(0.0, 1.0),
    spawn_direction_angle: DOWN,
}];
const FFA_2: &[SpawnPoint] = &[SpawnPoint {
    spawn_position: na::Vector2::new(-1.0, 0.0),
    spawn_direction_angle: RIGHT,
}];
const FFA_3: &[SpawnPoint] = &[SpawnPoint {
    spawn_position: na::Vector2::new(1.0, 0.0),
    spawn_direction_angle: LEFT,
}];
