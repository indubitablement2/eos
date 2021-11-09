use crate::collision::*;
use crate::connection_manager::*;
use crate::packets::*;
use indexmap::IndexMap;
use rapier2d::na::{vector, Vector2};
use rapier2d::prelude::*;
use std::ops::Add;
use std::time::Duration;

// command: server send those to clients inside Battlescape, so that they can update.

/// Unique client identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientID {
    pub id: u64,
}

/// Can be owned by the server or a client.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FleetID {
    pub id: u64,
}

/// A unique ActiveBattlescape identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BattlescapeID {
    pub id: u64,
}

pub struct Client {
    /// The fleet currently controlled by this client.
    pub fleet_control: Option<FleetID>,

    pub connection: Connection,

    /// What this client's next Battlescape input will be.
    pub input_battlescape: BattlescapeInput,
    /// Resend previous battlescape commands if they have not been acknowledged.
    pub unacknowledged_commands: IndexMap<u32, Vec<u8>>,
}
impl Client {
    /// The maximum number of element in unacknowledged_commands.
    /// Above that, the client should be kicked from the Battlescape.
    pub const MAX_UNACKOWLEDGED_COMMAND: usize = 32;
}

/// Client controlled fleet will cause a more polished simulation in this radius.
/// Like spawning server owned fleets when inside FactionActivity.
/// Usual attached to a fleet.
pub struct RealityBubble {
    // TODO: Should this be there? How do I avoid unowned RealityBubble?
    pub client: ClientID,
}
impl RealityBubble {
    pub const RADIUS: f32 = 256.0;
    pub const COLLISION_MEMBERSHIP: u32 = 2 ^ 0;
}

/// A fleet of ships owned by a client or the server.
/// Only used around Client. Otherwise use more crude simulation.
pub struct Fleet {
    /// If a Client own this fleet or the server.
    pub owner: Option<ClientID>,
    /// If this fleet is participating in a Battlescape.
    pub battlescape: Option<BattlescapeID>,
    pub wish_position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    /// The collider that make this fleet detected.
    detection_handle: ColliderHandle,
    /// The collider that detect other fleet.
    detector_handle: ColliderHandle,
    // TODO: Goal: What this fleet want to do? (so that it does not just chase a client forever.)
    // TODO: Add factions
    // pub faction: FactionID,
}
impl Fleet {
    pub const DETECTION_COLLISION_MEMBERSHIP: u32 = 2 ^ 1;
    pub const DETECTOR_COLLISION_MEMBERSHIP: u32 = 2 ^ 2;

    /// Fleet are composed of multiple colliders. This function move them at the same time.
    pub fn set_position(&self, collider_set: &mut ColliderSet, position: Vector2<f32>) {
        // Set detection position
        if let Some(detection_collider) = collider_set.get_mut(self.detection_handle) {
            detection_collider.set_translation(position);
        }
        // Also set detector to the same position.
        if let Some(detector_collider) = collider_set.get_mut(self.detector_handle) {
            detector_collider.set_translation(position);
        }
    }

    pub fn get_position(&self, collider_set: &ColliderSet) -> Vector2<f32> {
        if let Some(collider) = collider_set.get(self.detection_handle) {
            *collider.translation()
        } else {
            vector![0.0, 0.0]
        }
    }
}

/// An ongoing battle on the Metascape.
/// If no client are controlling a fleet, it will be crudely simulated by the Metascape.
#[derive(Debug, Clone)]
pub struct ActiveBattlescape {
    pub tick: u32,
    /// Fleets implied in this Battlescape.
    pub fleets: Vec<FleetID>,
}

/// A system with stars and planets.
pub struct System {}
impl System {
    pub const COLLISION_MEMBERSHIP: u32 = 2 ^ 3;
    /// TODO: Temporary size constant. This should come from what is inside the system.
    pub const SIZE: f32 = 32.0;
}

// /// A faction is mayhem in this area.
// /// Will emit fleets if a player is nearby.
// pub struct FactionActivity {}
// impl FactionActivity {
//     pub const COLLISION_MEMBERSHIP: u32 = 2^3;
// }

/// The simulation structure.
pub struct Metascape {
    pub tick: u64,
    pub bound: AABB,

    pub collision_pipeline_bundle: CollisionPipelineBundle,
    pub query_pipeline_bundle: QueryPipelineBundle,
    pub collider_set: ColliderSet,
    pub collision_events_bundle: CollisionEventsBundle,

    connection_manager: ConnectionsManager,
    /// Connection that will be disconnected next update.
    pub disconnect_queue: Vec<ClientID>,
    /// Connected clients.
    pub clients: IndexMap<ClientID, Client>,

    pub fleets: IndexMap<FleetID, Fleet>,
    pub reality_bubbles: IndexMap<ColliderHandle, RealityBubble>,
    // pub active_battlescapes: IndexMap<ColliderHandle, ActiveBattlescape>,
    pub systems: IndexMap<ColliderHandle, System>,
}
impl Metascape {
    /// How long between each Battlescape/Metascape tick.
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(50);

    /// Create a new Metascape with default parameters.
    pub fn new() -> tokio::io::Result<Self> {
        // Create the rapier intersection channel.
        let (collision_events_bundle, channel_event_collector) = CollisionEventsBundle::new();

        let connection_manager = ConnectionsManager::new()?;

        Ok(Self {
            tick: 0,
            bound: AABB::from_half_extents(point![0.0, 0.0], vector![2048.0, 2048.0]),
            collision_pipeline_bundle: CollisionPipelineBundle::new(channel_event_collector),
            query_pipeline_bundle: QueryPipelineBundle::new(),
            collider_set: ColliderSet::new(),
            collision_events_bundle,
            connection_manager,
            disconnect_queue: Vec::new(),
            clients: IndexMap::new(),
            fleets: IndexMap::new(),
            reality_bubbles: IndexMap::new(),
            systems: IndexMap::new(),
        })
    }

    fn get_new_connection(&mut self) {
        while let Ok(connection) = self.connection_manager.new_connection_receiver.try_recv() {
            // TODO: Add a new fleet untill load/save is implemented.
            let fleet_id = self.create_fleet(Some(connection.client_id), vector![0.0, 0.0]);

            let client_id = connection.client_id;

            // Create client.
            let client = Client {
                fleet_control: Some(fleet_id),
                connection,
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
                                if let Some(fleet_id) = client.fleet_control {
                                    // Get fleet.
                                    if let Some(fleet) = self.fleets.get_mut(&fleet_id) {
                                        fleet.wish_position = wish_position;
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

    /// Calculate fleet velocity based on wish position.
    /// TODO: Fleets engaged in the same Battlescape should aggregate.
    fn calc_fleet_velocity(&mut self) {
        for fleet in self.fleets.values_mut() {
            fleet.velocity += fleet.wish_position;
            fleet.velocity = fleet.velocity.cap_magnitude(10.0);
        }
    }

    /// Apply calculated velocity to fleets.
    fn apply_fleet_velocity(&mut self) {
        for fleet in self.fleets.values() {
            let new_pos = fleet.get_position(&self.collider_set).add(fleet.velocity);
            fleet.set_position(&mut self.collider_set, new_pos);
        }
    }

    /// Update collision and pipeline with new collider position.
    fn update_collision_pipelines(&mut self) {
        self.collision_pipeline_bundle.update(&mut self.collider_set);
        self.query_pipeline_bundle.update(&mut self.collider_set);
    }

    /// TODO: Send unacknowledged commands.
    /// TODO: Just sending every fleets position for now.
    fn send_udp(&mut self) {
        let fleets_position: Vec<Vector2<f32>> = self
            .fleets
            .values()
            .map(|fleet| self.collider_set.get(fleet.detection_handle).unwrap())
            .map(|collider| *collider.translation())
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

        self.calc_fleet_velocity();
        self.apply_fleet_velocity();

        // TODO: Get battlescape result.
        // TODO: Compare battlescape result to detect desync.
        // TODO: Apply battlescape result to Metascape.
        // TODO: split/merge/delete Battlescape.
        // TODO: Update ActiveBattlescape.
        // TODO: Make next Battlescape command and add it to Client's unacknowledged commands.

        self.update_collision_pipelines();

        self.send_udp();
        self.flush_disconnect_queue();
    }

    /// Get the position and radius of every system.
    /// TODO: Remake to send every circles position and radius.
    pub fn get_systems(&self) -> Vec<(Vector2<f32>, f32)> {
        let mut systems = Vec::with_capacity(self.systems.len());

        for collider_handle in self.systems.keys() {
            if let Some(collider) = self.collider_set.get(*collider_handle) {
                if let Some(ball) = collider.shape().as_ball() {
                    systems.push((collider.translation().to_owned(), ball.radius));
                }
            }
        }

        systems
    }

    /// Add a new fleet to the metascape and return its id.
    fn create_fleet(&mut self, owner: Option<ClientID>, translation: Vector2<f32>) -> FleetID {
        // Create colliders.
        let detection_collider = ColliderBuilder::ball(50.0)
            .sensor(true)
            .active_events(ActiveEvents::INTERSECTION_EVENTS)
            .translation(translation)
            .collision_groups(InteractionGroups {
                memberships: Fleet::DETECTION_COLLISION_MEMBERSHIP,
                filter: Fleet::DETECTOR_COLLISION_MEMBERSHIP,
            })
            .build();
        let detector_collider = ColliderBuilder::ball(90.0)
            .sensor(true)
            .active_events(ActiveEvents::INTERSECTION_EVENTS)
            .translation(translation)
            .collision_groups(InteractionGroups {
                memberships: Fleet::DETECTOR_COLLISION_MEMBERSHIP,
                filter: Fleet::DETECTION_COLLISION_MEMBERSHIP,
            })
            .build();

        // Add colliders.
        let detection_handle = self.collider_set.insert(detection_collider);
        let detector_handle = self.collider_set.insert(detector_collider);

        // Add new fleet.
        let fleet_id = FleetID { id: 100 };
        let new_fleet = Fleet {
            owner,
            battlescape: None,
            wish_position: vector![0.0, 0.0],
            velocity: vector![0.0, 0.0],
            detection_handle,
            detector_handle,
        };
        self.fleets.insert(fleet_id, new_fleet);

        fleet_id
    }

    pub fn flush_disconnect_queue(&mut self) {
        self.disconnect_queue
            .drain(..)
            .collect::<Vec<ClientID>>()
            .into_iter()
            .for_each(|client_id| self.disconnect_client(client_id));
    }

    /// Immediately disconnect a client.
    /// TODO: Save his stuff and what not.
    pub fn disconnect_client(&mut self, client_id: ClientID) {
        // Remove client.
        if let Some(client) = self.clients.remove(&client_id) {

            
            info!("Disconnected {:?}.", client_id);
        }
    }

    /// TODO: DELETE ME
    pub fn get_fleets(&self) -> Vec<(Vector2<f32>, f32, Vector2<f32>, f32)> {
        let mut fleets = Vec::with_capacity(self.fleets.len());

        for fleet in self.fleets.values() {
            if let Some(detection_collider) = self.collider_set.get(fleet.detection_handle) {
                if let Some(detection_ball) = detection_collider.shape().as_ball() {
                    if let Some(detector_collider) = self.collider_set.get(fleet.detector_handle) {
                        if let Some(detector_ball) = detector_collider.shape().as_ball() {
                            fleets.push((
                                *detection_collider.translation(),
                                detection_ball.radius,
                                *detector_collider.translation(),
                                detector_ball.radius,
                            ))
                        }
                    }
                }
            }
        }

        fleets
    }
}
