use crate::collision::*;
use crate::packets;
use crate::packets::*;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use indexmap::IndexMap;
use laminar::Packet;
use laminar::Socket;
use laminar::SocketEvent;
use rapier2d::na::{vector, Vector2};
use rapier2d::prelude::*;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::ops::Add;
use std::thread::spawn;
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

    pub udp_address: SocketAddr,
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
    /// TODO: Wrap these with a set_position() so they stick together. Also one should not exist without the other anyway.
    pub detection_handle: ColliderHandle,
    /// The collider that detect other fleet. Follow detection_collider.
    pub detector_handle: ColliderHandle,
    // TODO: Goal: What this fleet want to do? (so that it does not just chase a client forever.)
    // TODO: Add factions
    // pub faction: FactionID,
}
impl Fleet {
    pub const DETECTION_COLLISION_MEMBERSHIP: u32 = 2 ^ 1;
    pub const DETECTOR_COLLISION_MEMBERSHIP: u32 = 2 ^ 2;
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

    pub udp_packet_sender: Sender<Packet>,
    pub udp_packet_receiver: Receiver<SocketEvent>,

    /// Used when receiving udp packets.
    /// Don't send to these addresses as they may be slightly outdated.
    /// Send to the address in client instead.
    pub laminar_addresses: IndexMap<SocketAddr, ClientID>,
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
    pub fn new() -> Result<Self, ()> {
        // Create the rapier intersection channel.
        let (collision_events_bundle, channel_event_collector) = CollisionEventsBundle::new();

        match Socket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)) {
            Ok(mut udp_socket) => {
                // Get the channels from the Socket.
                let udp_packet_sender = udp_socket.get_packet_sender();
                let udp_packet_receiver = udp_socket.get_event_receiver();

                // Start polling on a separate thread.
                // TODO: This can not be stopped. Maybe add an Arc<bool> in there?
                spawn(move || udp_socket.start_polling_with_duration(Some(Duration::from_millis(5))));

                return Ok(Self {
                    tick: 0,
                    bound: AABB::from_half_extents(point![0.0, 0.0], vector![2048.0, 2048.0]),
                    collision_pipeline_bundle: CollisionPipelineBundle::new(channel_event_collector),
                    query_pipeline_bundle: QueryPipelineBundle::new(),
                    collider_set: ColliderSet::new(),
                    collision_events_bundle,
                    udp_packet_sender,
                    udp_packet_receiver,
                    laminar_addresses: IndexMap::new(),
                    clients: IndexMap::new(),
                    fleets: IndexMap::new(),
                    reality_bubbles: IndexMap::new(),
                    systems: IndexMap::new(),
                });
            }
            Err(err) => {
                error!("Error while binding Metascape socket: {:?}.", err);
                Err(())
            }
        }
    }

    /// Make sure that we don't have partially connected client.
    fn check_clients(&mut self) {
        // Disconnect any client without a laminar address.
        self.clients
            .iter()
            .filter_map(|(client_id, client)| {
                if self.laminar_addresses.get(&client.udp_address).is_none() {
                    debug!("{:?} does not have a laminar address. Disconnecting...", client_id);
                    Some(*client_id)
                } else {
                    None
                }
            })
            .collect::<Vec<ClientID>>()
            .into_iter()
            .for_each(|client_id| self.disconnect_client(client_id));

        let mut addr_without_client = Vec::new();

        // Disconnect any client with address not matching laminar address.
        self.laminar_addresses
            .iter()
            .filter_map(|(addr, client_id)| {
                if let Some(client) = self.clients.get(client_id) {
                    if client.udp_address != *addr {
                        Some(*client_id)
                    } else {
                        // All is fine.
                        None
                    }
                } else {
                    // Laminar address without client will be removed later.
                    addr_without_client.push(*addr);
                    None
                }
            })
            .collect::<Vec<ClientID>>()
            .into_iter()
            .for_each(|client_id| {
                self.disconnect_client(client_id);
            });

        // Remove any laminar address without a client.
        addr_without_client.into_iter().for_each(|addr| {
            self.laminar_addresses.remove(&addr);
        });
    }

    /// Get and process clients udp packets.
    fn get_client_udp_input(&mut self) {
        while let Ok(socket_event) = self.udp_packet_receiver.try_recv() {
            match socket_event {
                SocketEvent::Packet(packet) => {
                    // Check that client is properly connected.
                    if let Some(client_id) = self.laminar_addresses.get(&packet.addr()) {
                        if let Some(client) = self.clients.get_mut(client_id) {
                            // Deserialize packet.
                            if let Ok(udp_client) = UdpClient::deserialize(packet.payload()) {
                                trace!("Got {:?} from {}", &udp_client, packet.addr());
                                match udp_client {
                                    UdpClient::Battlescape {
                                        wish_input,
                                        acknowledge_command,
                                    } => {
                                        // Set as most recent Battlescape input.
                                        client.input_battlescape = wish_input;

                                        // Remove an acknowledged command from this client.
                                        client.unacknowledged_commands.remove(&acknowledge_command);
                                    }
                                    UdpClient::Metascape { wish_position } => {
                                        // Set wish location for the controlled fleet.
                                        if let Some(fleet_id) = client.fleet_control {
                                            if let Some(fleet) = self.fleets.get_mut(&fleet_id) {
                                                fleet.wish_position = wish_position;
                                            }
                                        }
                                    }
                                }
                            } else {
                                debug!(
                                    "Received a client packet from {:?} that could not be deserialized. Ignoring...",
                                    client_id
                                );
                            }
                        } else {
                            warn!(
                                "Received a packet from {:?} at {}, but can not find its client. Ignoring...",
                                client_id,
                                packet.addr()
                            );
                        }
                    } else {
                        debug!(
                            "Received a packet from a connected laminar socket at {}, but can not find its client. Ignoring...",
                            packet.addr()
                        );
                    }
                }
                SocketEvent::Connect(addr) => {
                    // Check that addr was added.
                    if self.laminar_addresses.get(&addr).is_none() {
                        error!("Laminar socket connected, but can not find client. {}", &addr);
                    }
                }
                SocketEvent::Timeout(addr) => {
                    info!("Laminar connection at {} timedout.", &addr);
                    if let Some(client_id) = self.laminar_addresses.remove(&addr) {
                        self.disconnect_client(client_id);
                    }
                }
                SocketEvent::Disconnect(addr) => {
                    info!("Laminar connection at {} disconnected.", &addr);
                    if let Some(client_id) = self.laminar_addresses.remove(&addr) {
                        self.disconnect_client(client_id);
                    }
                }
            }
        }
    }

    /// Calculate fleet velocity based on wish position.
    fn calc_fleet_velocity(&mut self) {
        for fleet in self.fleets.values_mut() {
            fleet.velocity += fleet.wish_position;
            fleet.velocity = fleet.velocity.cap_magnitude(10.0);
        }
    }

    /// Adds calculated velocity to fleets.
    /// TODO: Fleets engaged in the same Battlescape should aggregate.
    fn apply_fleet_velocity(&mut self) {
        for fleet in self.fleets.values() {
            let mut new_pos = vector![0.0f32, 0.0];
            // Set detection pos
            if let Some(detection_collider) = self.collider_set.get_mut(fleet.detection_handle) {
                new_pos = detection_collider.translation().add(fleet.velocity);
                detection_collider.set_translation(new_pos);
            }
            // Also set detector to the same position.
            if let Some(detector_collider) = self.collider_set.get_mut(fleet.detector_handle) {
                detector_collider.set_translation(new_pos);
            }
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

        let payload = UdpServer::Metascape { fleets_position }.serialize();

        for client in self.clients.values() {
            let _ = self
                .udp_packet_sender
                .send(Packet::unreliable(client.udp_address, payload.clone()));
        }
    }

    pub fn update(&mut self) {
        self.tick += 1;
        self.check_clients();
        self.get_client_udp_input();
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

    /// TODO: Only used for testing untill tcp is implemented.
    pub fn add_client_with_fleet(&mut self, client_id: ClientID, address: SocketAddr, translation: Vector2<f32>) {
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
            owner: Some(client_id),
            battlescape: None,
            wish_position: vector![0.0, 0.0],
            velocity: vector![0.0, 0.0],
            detection_handle,
            detector_handle,
        };
        self.fleets.insert(fleet_id, new_fleet);

        // Add laminar address entry.
        self.laminar_addresses.insert(address, client_id);

        // Send a packet to the address to initialize a laminar connection.
        let _ = self.udp_packet_sender.send(Packet::unreliable(address, vec![]));

        // Create Client.
        let new_client = Client {
            fleet_control: Some(fleet_id),
            input_battlescape: packets::BattlescapeInput {
                fire_toggle: false,
                wish_dir: 0.0,
                aim_dir: 0.0,
                wish_dir_force: 0.0,
            },
            unacknowledged_commands: IndexMap::new(),
            udp_address: address,
        };

        // Add Client.
        self.clients.insert(client_id, new_client);
    }

    /// Disconnect a client.
    /// TODO: Save his stuff and what not.
    pub fn disconnect_client(&mut self, client_id: ClientID) {
        // Remove client.
        if let Some(client) = self.clients.remove(&client_id) {
            // Remove address. This is redundant.
            self.laminar_addresses.remove(&client.udp_address);
        }

        info!("Disconnected {:?}.", client_id);
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
