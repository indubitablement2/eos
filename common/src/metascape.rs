use crate::collision::*;
use crate::connection_manager::*;
use crate::packets::*;
use glam::vec2;
use glam::Vec2;
use indexmap::IndexMap;
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
    pub fleet_control: FleetID,

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

    pub const REALITY_BUBBLE_RADIUS: f32 = 256.0;
    pub const REALITY_BUBBLE_COLLISION_MEMBERSHIP: u32 = 2 ^ 0;
}

/// A fleet of ships owned by a client or the server.
/// Only used around Client. Otherwise use more crude simulation.
pub struct Fleet {
    /// If a Client own this fleet or the server.
    owner: Option<ClientID>,
    /// If this fleet is participating in a Battlescape.
    pub battlescape: Option<BattlescapeID>,
    pub wish_position: Vec2,
    pub velocity: Vec2,

    /// The collider that make this fleet detected.
    detection_collider_id: ColliderId,
    /// The collider that detect other fleet.
    detector_collider_id: ColliderId,
    // /// Client controlled fleet will cause a more polished simulation in this radius.
    // /// Like spawning server owned fleets when inside FactionActivity.
    // reality_bubble_handle: Option<ColliderHandle>,

    // TODO: Goal: What this fleet want to do? (so that it does not just chase a client forever.)
    // TODO: Add factions
    // pub faction: FactionID,
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
    /// The maximum distance to the center.
    pub bound: f32,

    pub intersection_pipeline: IntersectionPipeline,
    // pub intersection_events_receiver: Receiver<IntersectionEvent>,
    pub connection_manager: ConnectionsManager,
    /// Connection that will be disconnected next update.
    pub disconnect_queue: Vec<ClientID>,
    /// Connected clients.
    pub clients: IndexMap<ClientID, Client>,

    pub fleets: IndexMap<FleetID, Fleet>,
    // pub active_battlescapes: IndexMap<ColliderHandle, ActiveBattlescape>,
    pub systems: IndexMap<ColliderId, System>,
}
impl Metascape {
    /// How long between each Battlescape/Metascape tick.
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(50);

    /// Create a new Metascape with default parameters.
    pub fn new(local: bool) -> tokio::io::Result<Self> {
        let connection_manager = ConnectionsManager::new(local)?;

        Ok(Self {
            tick: 0,
            bound: 1024.0,
            intersection_pipeline: IntersectionPipeline::new(),
            // intersection_events_receiver,
            connection_manager,
            disconnect_queue: Vec::new(),
            clients: IndexMap::new(),
            fleets: IndexMap::new(),
            systems: IndexMap::new(),
        })
    }

    fn get_new_connection(&mut self) {
        while let Ok(connection) = self.connection_manager.new_connection_receiver.try_recv() {
            // TODO: Add a new fleet untill load/save is implemented.
            let fleet_id = self.create_fleet(Some(connection.client_id), vec2(0.0, 0.0));

            let client_id = connection.client_id;

            // Create client.
            let client = Client {
                fleet_control: fleet_id,
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
                                if let Some(fleet) = self.fleets.get_mut(&client.fleet_control) {
                                    fleet.wish_position = wish_position;
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
            let mut new_pos = Vec2::ZERO;

            if let Some(detection_collider) = self.intersection_pipeline.get_collider_mut(fleet.detection_collider_id) {
                // Update velocity with fleet movement toward fleet's wish position.
                fleet.velocity += (fleet.wish_position - detection_collider.position).clamp_length_max(1.0);

                // Apply new position.
                detection_collider.position += fleet.velocity;
                new_pos = detection_collider.position;
            }

            // Apply new position to other collider of this fleet.
            if let Some(detector_collider) = self.intersection_pipeline.get_collider_mut(fleet.detector_collider_id) {
                detector_collider.position = new_pos;
            }
        }
    }

    /// TODO: Send unacknowledged commands.
    /// TODO: Just sending every fleets position for now.
    fn send_udp(&mut self) {
        let fleets_position: Vec<Vec2> = self
            .fleets
            .values()
            .map(|fleet| {
                self.intersection_pipeline
                    .get_collider(fleet.detection_collider_id)
                    .unwrap_or_default()
                    .position
            })
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

    /// Get a copy of every colliders. Useful for debug display.
    pub fn get_colliders(&self) -> Vec<Collider> {
        self.intersection_pipeline.get_colliders_copy()
    }

    pub fn get_system_rows_separation(&self) -> Vec<f32> {
        self.intersection_pipeline.get_rows_separation(Membership::System)
    }

    /// Add a new fleet to the metascape and return its id.
    fn create_fleet(&mut self, owner: Option<ClientID>, position: Vec2) -> FleetID {
        // Create colliders.
        let detection_collider = Collider {
            radius: 20.0,
            position: vec2(0.0, 0.0),
        };
        let detector_collider = Collider {
            radius: 30.0,
            position: vec2(0.0, 0.0),
        };

        // Add colliders.
        let detection_collider_id = self
            .intersection_pipeline
            .insert_collider(detection_collider, Membership::FleetDetection);
        let detector_collider_id = self
            .intersection_pipeline
            .insert_collider(detector_collider, Membership::FleetDetector);

        // Add new fleet.
        let fleet_id = FleetID { id: 100 };
        let new_fleet = Fleet {
            owner,
            battlescape: None,
            wish_position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            detection_collider_id,
            detector_collider_id,
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
}
