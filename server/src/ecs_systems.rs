use crate::data_manager::DataManager;
use crate::ecs_components::*;
use crate::ecs_events::*;
use crate::intersection::*;
use crate::res_clients::*;
use crate::res_fleets::*;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use common::collider::Collider;
use common::idx::*;
use common::packets::*;
use common::parameters::MetascapeParameters;
use common::res_time::TimeRes;
use glam::vec2;
use glam::Vec2;
use rand::Rng;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "first";
    schedule.add_stage(current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, increment_time.system());
    schedule.add_system_to_stage(current_stage, get_new_clients.system());
    schedule.add_system_to_stage(current_stage, ai_fleet_sensor.system());
    schedule.add_system_to_stage(current_stage, client_fleet_sensor.system());

    let previous_stage = current_stage;
    let current_stage = "pre_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, fleet_ai.system());

    let previous_stage = current_stage;
    let current_stage = "update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, movement.system());

    let previous_stage = current_stage;
    let current_stage = "post_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, apply_velocity.system());
    schedule.add_system_to_stage(current_stage, disconnect_client.system());
    schedule.add_system_to_stage(current_stage, spawn_ai_fleet.system());

    let previous_stage = current_stage;
    let current_stage = "last";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, send_detected_fleet.system());
    schedule.add_system_to_stage(current_stage, update_intersection_pipeline.system());
}

//* first

fn increment_time(mut time_res: ResMut<TimeRes>) {
    time_res.tick += 1;
}

/// Get new connection and insert client.
fn get_new_clients(
    mut commands: Commands,
    mut clients_res: ResMut<ClientsRes>,
    mut fleets_res: ResMut<FleetsRes>,
    data_manager: Res<DataManager>,
    client_connected: ResMut<EventRes<ClientConnected>>,
    intersection_pipeline: Res<IntersectionPipeline>,
) {
    while let Ok(connection) = clients_res.connection_manager.new_connection_receiver.try_recv() {
        let client_id = connection.client_id;
        let fleet_id = FleetId::from(client_id);

        // Create client.
        let client = Client { connection };

        // Insert client.
        match clients_res.connected_clients.insert(client_id, client) {
            Some(old_client) => {
                debug!("{:?} was disconnected as a new connection took this client.", client_id);
                // TODO: Send message to old client explaining why he got disconnected.
            }
            None => {
                // TODO: Load and insert client data.
                let client_data = data_manager.load_client(client_id);
            }
        }

        // Check if fleet is already spawned.
        if let Some(old_fleet_entity) = fleets_res.spawned_fleets.get(&fleet_id) {
            // TODO: Check old fleet components.
            error!("{:?} is already spawned.", fleet_id);
            for k in fleets_res.spawned_fleets.keys() {
                warn!("{:?}", k);
            }
            // todo!();
        } else {
            // TODO: Load or create client's fleet.
            let entity = commands
                .spawn_bundle(ClientFleetBundle {
                    client_id,
                    fleet_bundle: FleetBundle {
                        fleet_id,
                        position: Position(Vec2::ZERO),
                        wish_position: WishPosition(Vec2::ZERO),
                        velocity: Velocity(Vec2::ZERO),
                        acceleration: Acceleration(0.1),
                        fleet_ai: FleetAI {
                            goal: FleetGoal::Controlled,
                        },
                        reputation: Reputation(0),
                        detected_radius: DetectedRadius(10.0),
                        detector_radius: DetectorRadius(30.0),
                        entity_detected: EntityDetected(Vec::new()),
                    },
                })
                .id();

            // Insert fleet.
            let _ = fleets_res.spawned_fleets.insert(fleet_id, entity);
        }

        // Trigger event.
        client_connected.events.push(ClientConnected { client_id });
    }
}

/// Determine what each ai fleet can see.
fn ai_fleet_sensor(
    mut query: Query<(&Position, &FleetId, &DetectorRadius, &mut EntityDetected), Without<ClientId>>,
    intersection_pipeline: Res<IntersectionPipeline>,
    task_pool: Res<TaskPool>,
    time_res: Res<TimeRes>,
) {
    // We will only update 1/20 at a time.
    let num_turn = 20u64;
    let turn = time_res.tick % num_turn;

    query.par_for_each_mut(
        &task_pool,
        64 * num_turn as usize,
        |(pos, fleet_id, detector_radius, mut detected)| {
            if fleet_id.0 % num_turn == turn {
                detected.0.clear();

                let detector_collider = Collider {
                    radius: detector_radius.0,
                    position: pos.0,
                };

                intersection_pipeline.intersect_collider_into(detector_collider, &mut detected.0);
            }
        },
    );
}

/// Determine what each client's fleet can see.
fn client_fleet_sensor(
    mut query: Query<(&Position, &ClientId, &FleetId, &DetectorRadius, &mut EntityDetected)>,
    intersection_pipeline: Res<IntersectionPipeline>,
    clients_res: Res<ClientsRes>,
    task_pool: Res<TaskPool>,
    time_res: Res<TimeRes>,
) {
    // We will only update 1/20 at a time.
    let num_turn = 20u64;
    let turn = time_res.tick % num_turn;

    query.par_for_each_mut(
        &task_pool,
        32 * num_turn as usize,
        |(pos, client_id, fleet_id, detector_radius, mut detected)| {
            if fleet_id.0 % num_turn == turn {
                let old_len = detected.0.len();
                let old_detected = std::mem::replace(&mut detected.0, Vec::with_capacity(old_len));

                let detector_collider = Collider {
                    radius: detector_radius.0,
                    position: pos.0,
                };

                intersection_pipeline.intersect_collider_into(detector_collider, &mut detected.0);

                if let Some(client) = clients_res.connected_clients.get(client_id) {
                    // Sort result.
                    detected.0.sort_unstable();

                    // If the entity list changed, sent it to the client.
                    if old_detected != detected.0 {
                        let _ = client.connection.tcp_sender.blocking_send(TcpServer::EntityList {
                            tick: time_res.tick,
                            list: detected
                                .0
                                .iter()
                                .map(|collider_id| ServerEntity(collider_id.0))
                                .collect(),
                        });
                    }
                }
            }
        },
    );
}

//* pre_update

fn fleet_ai(
    mut query: Query<(&FleetId, &mut FleetAI, &Position, &mut WishPosition)>,
    task_pool: Res<TaskPool>,
    clients_res: Res<ClientsRes>,
    client_disconnected: ResMut<EventRes<ClientDisconnected>>,
) {
    query.par_for_each_mut(&task_pool, 256, |(fleet_id, mut fleet_ai, pos, mut wish_pos)| {
        let mut rng = rand::thread_rng();

        match &mut fleet_ai.goal {
            // Get and process clients udp packets.
            FleetGoal::Controlled => {
                let client_id = ClientId::from(*fleet_id);

                if client_id.is_valid() {
                    if let Some(client) = clients_res.connected_clients.get(&client_id) {
                        // Apply the udp packets of this client on this fleet.
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
                                            wish_pos.0 = wish_position;
                                        }
                                    }
                                }
                                Err(err) => {
                                    if err == crossbeam_channel::TryRecvError::Disconnected {
                                        // Cliend disconnected.
                                        client_disconnected.events.push(ClientDisconnected { client_id });
                                    }
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    warn!("Fleet {:?} should not be controlled. Changing to idle...", fleet_id);
                    fleet_ai.goal = FleetGoal::Idle { duration: 0 };
                }
            }
            FleetGoal::Trade { from, to } => todo!(),
            FleetGoal::Guard { who, radius, duration } => todo!(),
            FleetGoal::Wandering { new_pos_timer } => {
                *new_pos_timer -= 1;
                if *new_pos_timer <= 0 {
                    wish_pos.0 = rng.gen::<Vec2>() * 100.0 - 50.0 + pos.0;
                    *new_pos_timer = rng.gen_range(5..30);
                }
            }
            FleetGoal::Idle { duration } => todo!(),
        }
    });
}

//* update

/// Add velocity based on wish position and acceleration.
/// TODO: Fleets engaged in the same Battlescape should aggregate.
fn movement(query: Query<(&Position, &WishPosition, &mut Velocity, &Acceleration)>) {
    query.for_each_mut(|(pos, wish_pos, mut vel, acceleration)| {
        // TODO: Stop threshold.
        if pos.0.distance_squared(wish_pos.0) < 10.0 {
            // Try to stop.
            let new_vel = -vel.0.clamp_length_max(acceleration.0);
            vel.0 += new_vel;
        } else {
            // Add velocity toward fleet's wish position at full speed.
            vel.0 += (wish_pos.0 - pos.0).clamp_length_max(acceleration.0);
        }
    });
}

//* post_update

fn apply_velocity(query: Query<(&mut Position, &mut Velocity)>, params: Res<MetascapeParameters>) {
    query.for_each_mut(|(mut pos, mut vel)| {
        // Apply velocity.
        pos.0 += vel.0;

        // Apply friction.
        vel.0 *= params.movement_friction;
    });
}

/// TODO: Prepare client's fleets to be removed.
fn disconnect_client(mut clients_res: ResMut<ClientsRes>, client_disconnected: Res<EventRes<ClientDisconnected>>) {
    while let Some(client_disconnected) = client_disconnected.events.pop() {
        if let Some(client) = clients_res.connected_clients.remove(&client_disconnected.client_id) {
            // TODO: Save his stuff.
            debug!("{:?} disconneced.", &client.connection.client_id);
        }
    }
}

/// TODO: This just spawn ai fleet every seconds for testing.
fn spawn_ai_fleet(time_res: Res<TimeRes>, mut commands: Commands, mut fleets_res: ResMut<FleetsRes>) {
    if time_res.tick == 5 {
        for _ in 0..500 {
            let fleet_id = fleets_res.get_new_fleet_id();

            let entity = commands
                .spawn_bundle(FleetBundle {
                    fleet_id,
                    position: Position(Vec2::ZERO),
                    wish_position: WishPosition(Vec2::ZERO),
                    velocity: Velocity(Vec2::ZERO),
                    acceleration: Acceleration(0.1),
                    fleet_ai: FleetAI {
                        goal: FleetGoal::Wandering { new_pos_timer: 0 },
                    },
                    reputation: Reputation(0),
                    detected_radius: DetectedRadius(10.0),
                    detector_radius: DetectorRadius(30.0),
                    entity_detected: EntityDetected(Vec::new()),
                })
                .id();

            // Insert fleet.
            let _ = fleets_res.spawned_fleets.insert(fleet_id, entity);
        }
    }
}

//* last

fn update_intersection_pipeline(
    query: Query<(Entity, &Position, &DetectedRadius)>,
    mut intersection_pipeline: ResMut<IntersectionPipeline>,
    time_res: Res<TimeRes>,
) {
    if time_res.tick % 20 == 0 {
        // Update all collider.
        query.for_each(|(entity, pos, detected_radius)| {
            let new_collider = Collider {
                radius: detected_radius.0,
                position: pos.0,
            };
            intersection_pipeline.modify_collider(ColliderId::new(entity.id()), new_collider);
        });

        intersection_pipeline.update();
    }
}

/// Send detected fleet to clients over udp.
fn send_detected_fleet(
    query_client: Query<(&ClientId, &Position, &EntityDetected)>,
    query_entity: Query<&Position>,
    time_res: Res<TimeRes>,
    clients_res: Res<ClientsRes>,
) {
    query_client.for_each(|(client_id, pos, entity_detected)| {
        if let Some(client) = clients_res.connected_clients.get(client_id) {
            let mut part = 0u8;
            let mut entities_position =
                Vec::with_capacity(entity_detected.0.len().min(UdpServer::NUM_ENTITIES_POSITION_MAX));

            for detected_entity in entity_detected.0.iter().map(|collider_id| Entity::new(collider_id.0)) {
                if let Ok(detected_pos) = query_entity.get(detected_entity) {
                    entities_position.push(detected_pos.0);

                    if entities_position.len() >= UdpServer::NUM_ENTITIES_POSITION_MAX {
                        // Send this part.
                        let packet = UdpServer::MetascapeEntityPosition {
                            metascape_tick: time_res.tick,
                            part,
                            entities_position,
                        };
                        let _ = client.connection.udp_sender.blocking_send(packet);

                        // Prepare next part.
                        part += 1;
                        entities_position = Vec::with_capacity(
                            (entity_detected
                                .0
                                .len()
                                .saturating_sub(usize::from(part) * UdpServer::NUM_ENTITIES_POSITION_MAX))
                            .min(UdpServer::NUM_ENTITIES_POSITION_MAX),
                        );
                    }
                } else {
                    entities_position.push(vec2(0.0, 1000.0));
                    debug!("Could not find a detected entity. Sending made up position...");
                }
            }

            if !entities_position.is_empty() {
                let packet = UdpServer::MetascapeEntityPosition {
                    metascape_tick: time_res.tick,
                    part,
                    entities_position,
                };
                let _ = client.connection.udp_sender.blocking_send(packet);
            }
        }
    });
}
