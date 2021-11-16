use crate::collision::*;
use crate::connection_manager::*;
use crate::packets::*;
use ahash::AHashMap;
use glam::vec2;
use glam::Vec2;
use indexmap::IndexMap;

// command: server send those to clients inside Battlescape, so that they can update.

/// Unique client identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientId {
    pub id: u32,
}

/// Can be owned by the server or a client.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FleetId {
    id: u32,
}

/// A unique Faction identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactionId {
    id: u32,
}

/// A unique ActiveBattlescape identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BattlescapeId {
    id: u32,
}

/// A unique System identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemId {
    pub id: u16,
}

/// A unique CelestialBody identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CelestialBodyId {
    systemid: SystemId,
    id: u16,
}

pub struct Client {
    /// The fleet currently controlled by this client.
    fleet_control: FleetId,
    reality_bubble: ColliderId,

    connection: Connection,

    /// Each client has its own faction.
    faction: Faction,

    /// What this client's next Battlescape input will be.
    input_battlescape: BattlescapeInput,
    /// Resend previous battlescape commands if they have not been acknowledged.
    unacknowledged_commands: IndexMap<u32, Vec<u8>>,
}
impl Client {
    /// The maximum number of element in unacknowledged_commands.
    /// Above that, the client should be kicked from the Battlescape.
    const MAX_UNACKOWLEDGED_COMMAND: usize = 32;

    pub const REALITY_BUBBLE_RADIUS: f32 = 256.0;
}

/// If a client does not own this fleet, this is the faction that own it.
enum FleetOwner {
    Client(ClientId),
    Faction(FactionId),
}

enum FleetAIState {
    Idle,
    GoToPosition(Vec2),
    PatrolPositions { positions: Vec<Vec2>, num_loop: i32 },
    Trade { from: (), to: () },
}

/// A fleet of ships owned by a client or the server.
/// Only used around Client. Otherwise use more crude simulation.
pub struct Fleet {
    /// If a client does not own this fleet, this is the faction that own it.
    owner: FleetOwner,
    /// If this fleet is participating in a Battlescape.
    battlescape: Option<BattlescapeId>,

    velocity: Vec2,
    /// The collider that make this fleet detected.
    collider_id: ColliderId,
    /// The collider that detect other fleet.
    detector_collider: Collider,
    // /// Client controlled fleet will cause a more polished simulation in this radius.
    // /// Like spawning server owned fleets when inside FactionActivity.
    // reality_bubble_handle: Option<ColliderHandle>,

    // What this fleet wants to do.
    fleet_ai: FleetAIState,
    /// Can we despawn this fleet if not inside a reality bubble and not owned by a connected client?
    no_despawn: bool,
}
impl Fleet {
    pub const RADIUS_MAX: f32 = 128.0;
}

pub struct Faction {
    /// Relation with other faction. If a faction is not there, it default to 0 (neutral).
    relation: AHashMap<FactionId, i16>,
}
impl Faction {
    fn new() -> Self {
        Self {
            relation: AHashMap::new(),
        }
    }
}

pub struct FactionActivity {
    faction_id: FactionId,
    collider_id: ColliderId,
}
impl FactionActivity {
    pub const RADIUS_MAX: f32 = 128.0;
}

/// An ongoing battle on the Metascape.
/// If no client are controlling a fleet, it will be crudely simulated by the Metascape.
#[derive(Debug, Clone)]
pub struct ActiveBattlescape {
    pub tick: u32,
    /// Fleets implied in this Battlescape.
    pub fleets: Vec<FleetId>,
}

pub enum CelestialBodyType {
    Star,
    Planet,
}

pub struct CelestialBody {
    pub celestial_body_type: CelestialBodyType,
    pub radius: f32,
    pub orbit_radius: f32,
    /// How many timestep for a full rotation.
    pub orbit_time: u32,
    pub moons: Vec<CelestialBody>,
}

/// A system with stars and planets.
pub struct System {
    /// The body that is the center of this system. Usualy a single star.
    pub bodies: Vec<CelestialBody>,
    pub collider_id: ColliderId,
}
impl System {
    pub const RADIUS_MIN: f32 = 64.0;
    pub const RADIUS_MAX: f32 = 256.0;
    /// Final System radius is added a bound with nothing in it.
    pub const BOUND_RADIUS_MULTIPLER: f32 = 1.25;
    /// Miminum number of timestep for a full rotation for every 1.0 away from main body.
    pub const ORBIT_TIME_MIN_PER_RADIUS: u32 = 300;
}

/// The simulation structure.
pub struct Metascape {
    pub tick: u64,
    /// The maximum distance to the center.
    pub bound: f32,

    pub intersection_pipeline: IntersectionPipeline,

    pub connection_manager: ConnectionsManager,
    /// Connection that will be disconnected next update.
    pub disconnect_queue: Vec<ClientId>,
    /// Connected clients.
    pub clients: IndexMap<ClientId, Client>,

    fleets: IndexMap<FleetId, Fleet>,
    // pub active_battlescapes: IndexMap<ColliderHandle, ActiveBattlescape>,
    pub systems: IndexMap<SystemId, System>,
    faction: AHashMap<FactionId, Faction>,
}
impl Metascape {
    /// Create a new Metascape with default parameters.
    pub fn new(local: bool, bound: f32) -> tokio::io::Result<Self> {
        let connection_manager = ConnectionsManager::new(local)?;

        Ok(Self {
            tick: 0,
            bound,
            intersection_pipeline: IntersectionPipeline::new(),
            // intersection_events_receiver,
            connection_manager,
            disconnect_queue: Vec::new(),
            clients: IndexMap::new(),
            fleets: IndexMap::new(),
            systems: IndexMap::new(),
            faction: AHashMap::new(),
        })
    }

    fn get_new_connection(&mut self) {
        while let Ok(connection) = self.connection_manager.new_connection_receiver.try_recv() {
            // TODO: Add a new fleet untill load/save is implemented.
            let fleet_id = self.create_fleet(FleetOwner::Client(connection.client_id), vec2(0.0, 0.0));
            let faction = Faction::new();

            let client_id = connection.client_id;

            // Create reality bubble.
            let reality_bubble = self.intersection_pipeline.insert_collider(
                Collider {
                    radius: Client::REALITY_BUBBLE_RADIUS,
                    position: vec2(0.0, 0.0),
                    custom_data: fleet_id.id,
                },
                Membership::RealityBubble,
            );

            // Create client.
            let client = Client {
                fleet_control: fleet_id,
                reality_bubble,
                connection,
                faction,
                input_battlescape: BattlescapeInput::default(),
                unacknowledged_commands: IndexMap::new(),
            };

            // Add to Metascape.
            if self.clients.insert(client_id, client).is_some() {
                info!("{:?} was disconnected as a new connection took this client.", client_id);
            }
        }
    }

    /// Get and process clients udp packets.
    fn get_client_udp_input(&mut self) {
        for (client_id, client) in &self.clients {
            loop {
                match client.connection.udp_receiver.try_recv() {
                    Ok(packet) => {
                        match packet {
                            UdpClient::Battlescape {
                                wish_input,
                                acknowledge_command,
                            } => {
                                // TODO: Set next as battlescape input.

                                // TODO: Remove an acknowledged command.

                                todo!();
                            }
                            UdpClient::Metascape { wish_position } => {
                                // Get controlled fleet.
                                if let Some(fleet) = self.fleets.get_mut(&client.fleet_control) {
                                    fleet.fleet_ai = FleetAIState::GoToPosition(wish_position);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        if err == crossbeam_channel::TryRecvError::Disconnected {
                            self.disconnect_queue.push(*client_id);
                        }
                        break;
                    }
                }
            }
        }
    }

    /// Calculate fleet velocity based on wish position.
    /// TODO: Fleets engaged in the same Battlescape should aggregate.
    fn fleet_velocity(&mut self) {
        for fleet in self.fleets.values_mut() {
            if let Some(detection_collider) = self.intersection_pipeline.get_collider_mut(fleet.collider_id) {
                match &fleet.fleet_ai {
                    FleetAIState::Idle => {}
                    FleetAIState::GoToPosition(wish_pos) => {
                        // Update velocity fleet movement toward fleet's wish position.
                        fleet.velocity += (*wish_pos - detection_collider.position).clamp_length_max(1.0);
                    }
                    FleetAIState::PatrolPositions { positions, num_loop } => todo!(),
                    FleetAIState::Trade { from, to } => todo!(),
                }

                // Apply velocity.
                detection_collider.position += fleet.velocity;
            }
        }
    }

    /// TODO: Send unacknowledged commands.
    /// TODO: Just sending every fleets position for now.
    fn send_udp(&mut self) {
        let fleets_position: Vec<Vec2> = self
            .fleets
            .values()
            .filter_map(|fleet| Some(self.intersection_pipeline.get_collider(fleet.collider_id)?.position))
            .collect();

        let packet = UdpServer::Metascape { fleets_position };

        for (client_id, client) in &self.clients {
            if client.connection.udp_sender.blocking_send(packet.clone()).is_err() {
                self.disconnect_queue.push(*client_id);
            }
        }
    }

    pub fn update(&mut self) {
        self.tick += 1;
        self.get_new_connection();
        self.get_client_udp_input();
        self.flush_disconnect_queue();

        self.fleet_velocity();

        // TODO: Get battlescape result.
        // TODO: Compare battlescape result to detect desync.
        // TODO: Apply battlescape result to Metascape.
        // TODO: split/merge/delete Battlescape.
        // TODO: Update ActiveBattlescape.
        // TODO: Make next Battlescape command and add it to Client's unacknowledged commands.

        self.intersection_pipeline.update();

        self.send_udp();
        self.flush_disconnect_queue();
    }

    /// Add a new fleet to the metascape and return its id.
    fn create_fleet(&mut self, owner: FleetOwner, position: Vec2) -> FleetId {
        // TODO: Get a FleetId.
        let fleet_id = FleetId { id: 100 };

        // Create Fleet detection Collider.
        let detection_collider = Collider {
            radius: 20.0,
            position,
            custom_data: fleet_id.id,
        };
        let collider_id = self
            .intersection_pipeline
            .insert_collider(detection_collider, Membership::Fleet);

        // Create Fleet detector Collider.
        let detector_collider = Collider {
            radius: 30.0,
            position,
            custom_data: 0,
        };

        // Create new Fleet.
        let new_fleet = Fleet {
            owner,
            battlescape: None,
            velocity: Vec2::ZERO,
            collider_id,
            detector_collider,
            fleet_ai: FleetAIState::Idle,
            no_despawn: true,
        };
        self.fleets.insert(fleet_id, new_fleet);

        fleet_id
    }

    /// Immediately disconnect a client.
    /// TODO: Save his stuff and what not.
    fn disconnect_client(&mut self, client_id: ClientId) {
        // Remove client.
        if let Some(client) = self.clients.remove(&client_id) {
            info!("Disconnected {:?}.", client_id);
        }
    }

    fn flush_disconnect_queue(&mut self) {
        self.disconnect_queue
            .drain(..)
            .collect::<Vec<ClientId>>()
            .into_iter()
            .for_each(|client_id| self.disconnect_client(client_id));
    }
}
