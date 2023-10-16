use super::*;

type Fleets = IndexMap<FleetId, Fleet, RandomState>;
type Factions = IndexMap<FactionId, Faction, RandomState>;

pub struct Metascape {
    time_total: f64,

    fleets: Fleets,
    factions: Factions,
}
impl Metascape {
    pub fn new() -> Self {
        todo!()
    }

    pub fn step(&mut self, delta: f32) {
        self.time_total += delta as f64;

        for fleet in self.fleets.values_mut() {
            fleet.update(delta);
        }
    }
}

/// Highest bit used to indicate standing with neutral.
/// faction good 1......
/// faction bad  0......
/// neutral      1111...
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FactionId(pub u64);
impl FactionId {
    const LIKE_NEUTRAL: u64 = 1 << 63;

    pub fn is_neutral(self) -> bool {
        self.0 == u64::MAX
    }

    pub fn like_neutral(self) -> bool {
        self.0 & FactionId::LIKE_NEUTRAL != 0
    }

    pub fn relation(self, other: Self) -> i32 {
        if self.0 == other.0 {
            1
        } else if (self.is_neutral() || other.is_neutral())
            && (self.like_neutral() && other.like_neutral())
        {
            0
        } else {
            -1
        }
    }
}

struct Faction {
    player_owned: Option<()>,

    fleets: AHashSet<FleetId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FleetId(pub u64);

struct Fleet {
    faction_id: FactionId,

    position: Vector2<f32>,
    velocity: Vector2<f32>,

    acceleration: f32,
    max_velocity: f32,

    wish_movement: Option<Vector2<f32>>,
}
impl Fleet {
    pub fn update(&mut self, delta: f32) {
        if let Some(target) = self.wish_movement {
            let to_target = target - self.position;
            if to_target.magnitude_squared() < 0.01 {
                if self.velocity.magnitude_squared() < 0.1 {
                    self.wish_movement = None;
                }
                self.velocity -= self.velocity.cap_magnitude(self.acceleration);
            } else {
                self.velocity += (to_target.cap_magnitude(self.max_velocity) - self.velocity)
                    .cap_magnitude(self.acceleration);
            }
        } else {
            // TODO: Orbit
        }

        self.position += self.velocity * delta;
    }
}
