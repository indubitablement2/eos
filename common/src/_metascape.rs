use crate::collision::*;
use crate::connection_manager::*;
use crate::packets::*;
use crate::MetascapeParameters;
use ahash::AHashMap;
use glam::vec2;
use glam::Vec2;
use indexmap::IndexMap;

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
    fleet_control: Option<FleetId>,

    connection: Connection,

    /// Each client has its own faction.
    faction_id: FactionId,

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

enum FleetAIState {
    Idle,
    GoToPosition(Vec2),
    PatrolPositions { positions: Vec<Vec2>, num_loop: i32 },
    Trade { from: (), to: () },
}

/// A fleet of ships owned by a client or the server.
/// Only used around Client. Otherwise use more crude simulation.
pub struct Fleet {
    /// If a client does own this fleet.
    owner: Option<ClientId>,
    /// If this fleet is participating in a Battlescape.
    battlescape: Option<BattlescapeId>,

    velocity: Vec2,
    /// The collider that make this fleet detected.
    fleet_collider_id: ColliderId,
    /// The collider that detect other fleet. Used as the defacto fleet position.
    detector_collider: Collider,
    /// If this fleet is controlled by a client, it also has a reality bubble following it.
    /// This gets created and deleted on the fly if this fleet is being controlled by a client.
    reality_bullbe_collider_id: Option<ColliderId>,

    // What this fleet wants to do.
    fleet_ai: FleetAIState,
    /// Can we despawn this fleet if not inside a reality bubble and not owned by a connected client?
    no_despawn: bool,

    faction_id: FactionId,
}
impl Fleet {
    pub const RADIUS_MAX: f32 = 128.0;
}

pub struct Faction {
    pub display_name: String,
    /// Relation with other faction. If a faction is not there, it default to 0 (neutral).
    pub relation: AHashMap<FactionId, i16>,
}
impl Faction {
    const RELATION_CLAMP: i16 = 10000;

    pub fn get_pretty_relation(&self) -> Vec<(FactionId, f32)> {
        self.relation
            .iter()
            .map(|(faction_id, relation)| (*faction_id, f32::from(*relation) / f32::from(Self::RELATION_CLAMP)))
            .collect()
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
    pub parameters: MetascapeParameters,

    pub intersection_pipeline: IntersectionPipeline,

    pub connection_manager: ConnectionsManager,
    /// Connection that will be disconnected next update.
    pub disconnect_queue: Vec<ClientId>,
    /// Connected clients.
    pub clients: IndexMap<ClientId, Client>,

    fleets: IndexMap<FleetId, Fleet>,
    last_fleet_id: u32,

    pub systems: IndexMap<SystemId, System>,

    pub faction: AHashMap<FactionId, Faction>,
    last_faction_id: u32,

    pub faction_activity: Vec<FactionActivity>,
}
impl Metascape {
    /// Create a new Metascape with default parameters.
    pub fn new(local: bool, parameters: MetascapeParameters) -> tokio::io::Result<Self> {
        let connection_manager = ConnectionsManager::new(local)?;

        Ok(Self {
            tick: 0,
            parameters,
            intersection_pipeline: IntersectionPipeline::new(),
            // intersection_events_receiver,
            connection_manager,
            disconnect_queue: Vec::new(),
            clients: IndexMap::new(),
            fleets: IndexMap::new(),
            last_fleet_id: 0,
            systems: IndexMap::new(),
            faction: AHashMap::new(),
            last_faction_id: 0,
            faction_activity: Vec::new(),
        })
    }

    fn get_new_connection(&mut self) {
        while let Ok(connection) = self.connection_manager.new_connection_receiver.try_recv() {
            // TODO: Add a new fleet untill load/save is implemented.
            let faction_id = self.create_faction("display_name".to_string());
            let fleet_id = self.create_fleet(Some(connection.client_id), faction_id, vec2(0.0, 0.0));

            let client_id = connection.client_id;

            // Create client.
            let client = Client {
                fleet_control: Some(fleet_id),
                connection,
                faction_id,
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
                                if let Some(fleet_id) = &client.fleet_control {
                                    if let Some(fleet) = self.fleets.get_mut(fleet_id) {
                                        fleet.fleet_ai = FleetAIState::GoToPosition(wish_position);
                                    }
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

    /// Update velocity based on ai.
    /// TODO: Fleets engaged in the same Battlescape should aggregate.
    fn fleet_movement(&mut self) {
        for fleet in self.fleets.values_mut() {
            // Get old position.
            let old_pos = fleet.detector_collider.position;

            match &fleet.fleet_ai {
                FleetAIState::Idle => {
                    // Try to cancel any velocity we have.
                    fleet.velocity -= fleet.velocity.clamp_length_max(0.1);
                }
                FleetAIState::GoToPosition(wish_pos) => {
                    // TODO: Stop threshold.
                    if old_pos.distance_squared(*wish_pos) < 10.0 {
                        fleet.fleet_ai = FleetAIState::Idle;
                    } else {
                        // Add velocity toward fleet's wish position at full speed.
                        fleet.velocity += (*wish_pos - old_pos).clamp_length_max(1.0);
                    }
                }
                FleetAIState::PatrolPositions { positions, num_loop } => todo!(),
                FleetAIState::Trade { from, to } => todo!(),
            }

            // Apply some friction.
            fleet.velocity *= self.parameters.movement_friction;

            // Get updated position.
            let new_pos = old_pos + fleet.velocity;

            // Set detector collider position.
            fleet.detector_collider.position = new_pos;

            // Set Fleet collider position.
            if let Some(fleet_collider) = self.intersection_pipeline.get_collider_mut(fleet.fleet_collider_id) {
                fleet_collider.position = new_pos;
            } else {
                warn!("Can not find fleet collider for a fleet. Ignoring...");
            }
        }
    }

    /// Manage reality bubble and queue despawning of fleets.
    fn fleet_handle(&mut self) {
        for (fleet_id, fleet) in self.fleets.iter_mut() {
            // Check if we should have a reality bubble.
            // Do we have a owner?
            if let Some(client_id) = &fleet.owner {
                // Is our owner connected?
                if let Some(client) = self.clients.get(client_id) {
                    // Is our owner controlling us?
                    if Some(*fleet_id) == client.fleet_control {
                        // Do we have a reality bubble collider id?
                        if let Some(reality_bubble_collider_id) = fleet.reality_bullbe_collider_id {
                            // Do we have a reality bubble collider?
                            if let Some(reality_bubble_collider) =
                                self.intersection_pipeline.get_collider_mut(reality_bubble_collider_id)
                            {
                                reality_bubble_collider.position = fleet.detector_collider.position;
                            }
                        } else {
                            // Create a reality bubble collider.
                            let reality_bubble_id = self.intersection_pipeline.insert_collider(
                                Collider {
                                    radius: Client::REALITY_BUBBLE_RADIUS,
                                    position: fleet.detector_collider.position,
                                    custom_data: fleet_id.id,
                                },
                                Membership::RealityBubble,
                            );
                            fleet.reality_bullbe_collider_id = Some(reality_bubble_id);
                        }
                    } else {
                        // TODO: We should not have a reality bubble as our owner is not actively controlling us.
                    }
                } else {
                    // TODO: We should not have a reality bubble as our owner is no connected.
                    // TODO: We should try to despawn as our owner is no connected.
                }
            } else {
                // TODO: We should not have a reality bubble as we have no owner.
            }
        }
    }

    /// TODO: Send unacknowledged commands.
    /// TODO: Just sending every fleets position for now.
    fn send_udp(&mut self) {
        let fleets_position: Vec<Vec2> = self.fleets.values().map(|fleet| fleet.detector_collider.position).collect();

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

        self.fleet_movement();
        self.fleet_handle();

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

    /// A fleet needs this to get detected.
    fn create_fleet_collider(&mut self, fleet_id: FleetId, position: Vec2, radius: f32) -> ColliderId {
        let detection_collider = Collider {
            radius,
            position,
            custom_data: fleet_id.id,
        };

        self.intersection_pipeline
            .insert_collider(detection_collider, Membership::Fleet)
    }

    /// Add a new fleet to the metascape and return its id.
    fn create_fleet(&mut self, owner: Option<ClientId>, faction_id: FactionId, position: Vec2) -> FleetId {
        // Get a FleetId.
        self.last_fleet_id += 1;
        let fleet_id = FleetId { id: self.last_fleet_id };

        let fleet_collider_id = self.create_fleet_collider(fleet_id, vec2(0.0, 0.0), 10.0);

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
            fleet_collider_id,
            detector_collider,
            reality_bullbe_collider_id: None,
            fleet_ai: FleetAIState::Idle,
            no_despawn: true,
            faction_id,
        };
        self.fleets.insert(fleet_id, new_fleet);

        fleet_id
    }

    fn create_faction(&mut self, display_name: String) -> FactionId {
        // Get FactionId.
        self.last_faction_id += 1;
        let faction_id = FactionId {
            id: self.last_faction_id,
        };

        // Create faction.
        let faction = Faction {
            display_name,
            relation: AHashMap::new(),
        };
        self.faction.insert(faction_id, faction);

        faction_id
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
