mod connection;

use super::*;
use connection::*;

type Fleets = IndexMap<FleetId, Fleet, RandomState>;
type Factions = IndexMap<FactionId, Faction, RandomState>;
type Clients = IndexMap<ClientId, Client, RandomState>;
type Connections = IndexMap<ClientId, Connection, RandomState>;

pub struct Metascape {
    time_total: f64,

    next_fleet_id: FleetId,
    next_faction_id: FactionId,
    next_client_id: ClientId,

    fleets: Fleets,
    factions: Factions,
    clients: Clients,
    connections: Connections,

    connection_receiver: std::sync::mpsc::Receiver<Connection>,
}
impl Metascape {
    pub async fn start() {
        let connection_receiver = connection::start_server_loop().await;

        Self {
            time_total: 0.0,

            next_fleet_id: FleetId(0),
            next_faction_id: FactionId(0),
            next_client_id: ClientId(0),

            fleets: Default::default(),
            factions: Default::default(),
            clients: Default::default(),
            connections: Default::default(),

            connection_receiver,
        }
        .run();
    }

    fn run(mut self) {
        self.fleets.insert(
            FleetId(123),
            Fleet {
                faction_id: FactionId(0),
                position: Vector2::new(0.0, 0.0),
                velocity: Vector2::new(0.0, 0.0),
                acceleration: 0.0,
                max_velocity: 1.0,
                wish_movement: None,
            },
        );

        std::thread::spawn(move || {
            let mut now = std::time::Instant::now();
            loop {
                self.step(now.elapsed().as_secs_f32());
                now = std::time::Instant::now();
                // TODO: Use a better sleep method.
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    pub fn step(&mut self, delta: f32) {
        self.time_total += delta as f64;

        // Handle new connection.
        for new_connection in self.connection_receiver.try_iter() {
            self.connections
                .insert(new_connection.client_id, new_connection);
        }

        // Handle client packets.
        let mut i = 0usize;
        while i < self.connections.len() {
            let connection = &mut self.connections[i];

            while let Some(packet) = connection.recv() {
                match packet {
                    ClientPacket::MoveFleet {
                        fleet_id,
                        wish_position,
                    } => {
                        // TODO: Check for NaN/infinity
                        if let Some(fleet) = self.fleets.get_mut(&fleet_id) {
                            fleet.wish_movement = Some(wish_position);
                        }
                    }
                }
            }

            // Remove disconnected clients.
            if connection.disconnected {
                log::debug!("{:?} disconnected", connection.client_id);
                self.connections.swap_remove_index(i);
            } else {
                i += 1;
            }
        }

        for fleet in self.fleets.values_mut() {
            fleet.update(delta);
        }

        // Send server packets.
        for connection in self.connections.values_mut() {
            let remove_fleets = Vec::new();
            let mut partial_fleets_info = Vec::new();
            let full_fleets_info = Vec::new();
            let mut positions = Vec::new();
            for (&fleet_id, fleet) in self.fleets.iter() {
                positions.push((fleet_id, fleet.position));

                let known_fleet = connection.knows_fleets.entry(fleet_id).or_insert_with(|| {
                    partial_fleets_info.push((fleet_id, 1));
                    KnownFleet { full_info: false }
                });
            }

            connection.send(ServerPacket::State {
                time: self.time_total,
                partial_fleets_info,
                full_fleets_info,
                positions,
                remove_fleets,
            });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

struct Client {
    fleets: AHashSet<FleetId>,
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
