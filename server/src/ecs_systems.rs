use crate::data_manager::DataManager;
use crate::ecs_components::*;
use crate::ecs_events::*;
use crate::res_clients::*;
use crate::res_fleets::*;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use common::idx::*;
use common::intersection::*;
use common::orbit::Orbit;
use common::packets::*;
use common::parameters::MetascapeParameters;
use common::res_time::TimeRes;
use glam::Vec2;

const DETECTED_UPDATE_INTERVAL: u32 = 5;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "first";
    schedule.add_stage(current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, get_new_clients.system());
    schedule.add_system_to_stage(current_stage, fleet_sensor.system());
    schedule.add_system_to_stage(current_stage, handle_client_inputs.system());

    let previous_stage = current_stage;
    let current_stage = "pre_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, handle_orbit.system());
    schedule.add_system_to_stage(current_stage, remove_orbit.system());
    schedule.add_system_to_stage(current_stage, client_fleet_ai.system());
    schedule.add_system_to_stage(current_stage, colony_fleet_ai.system());

    let previous_stage = current_stage;
    let current_stage = "update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, apply_fleet_movement.system());
    schedule.add_system_to_stage(current_stage, disconnect_client.system());
    schedule.add_system_to_stage(current_stage, increment_time.system());

    let previous_stage = current_stage;
    let current_stage = "post_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, send_detected_entity.system());
    schedule.add_system_to_stage(current_stage, update_intersection_pipeline.system());
}

//* first

/// Get new connection and insert client.
fn get_new_clients(
    mut commands: Commands,
    mut clients_res: ResMut<ClientsRes>,
    mut fleets_res: ResMut<FleetsRes>,
    data_manager: Res<DataManager>,
) {
    while let Ok(connection) = clients_res.connection_manager.new_connection_receiver.try_recv() {
        let client_id = connection.client_id;
        let fleet_id = FleetId::from(client_id);

        // Insert client.
        if let Some(old_connection) = clients_res.connected_clients.insert(client_id, connection) {
            debug!("{:?} was disconnected as a new connection took this client.", client_id);
            // Send message to old client explaining why he got disconnected.
            old_connection.send_packet_reliable(
                Packet::DisconnectedReason(DisconnectedReasonEnum::ConnectionFromOther).serialize(),
            );
            old_connection.flush_tcp_stream();
        }

        // Check if it has data.
        if clients_res
            .clients_data
            .try_insert(client_id, ClientData::default())
            .is_ok()
        {
            // This is this client's first login.
        }

        // Check if fleet is already spawned.
        if let Some(old_fleet_entity) = fleets_res.spawned_fleets.get(&fleet_id) {
            // TODO: Check old fleet components.
            debug!("{:?} is already spawned. TODO: Check old fleet components.", fleet_id);
        } else {
            // Create default client fleet.
            let entity = commands
                .spawn_bundle(ClientFleetBundle {
                    client_id,
                    know_entities: KnowEntities::default(),
                    fleet_bundle: FleetBundle {
                        fleet_id,
                        position: Position(Vec2::ZERO),
                        wish_position: WishPosition::default(),
                        velocity: Velocity::default(),
                        derived_fleet_stats: DerivedFleetStats { acceleration: 0.1, max_speed: 2.0 },
                        reputation: Reputation(0),
                        detected_radius: DetectedRadius(10.0),
                        detector_radius: DetectorRadius(50.0),
                        entity_detected: EntityDetected(Vec::new()),
                    },
                })
                .id();

            // Insert fleet.
            let _ = fleets_res.spawned_fleets.insert(fleet_id, entity);
        }
    }
}

/// Determine what each fleet can see.
fn fleet_sensor(
    mut query: Query<(&FleetId, &Position, &DetectorRadius, &mut EntityDetected)>,
    intersection_pipeline: Res<IntersectionPipeline>,
    task_pool: Res<TaskPool>,
    time_res: Res<TimeRes>,
) {
    // We will only update one part every tick.
    let turn = (time_res.tick % DETECTED_UPDATE_INTERVAL) as u64;
    let num_turn = DETECTED_UPDATE_INTERVAL as u64;

    query.par_for_each_mut(
        &task_pool,
        64 * num_turn as usize,
        |(fleet_id, position, detector_radius, mut entity_detected)| {
            if fleet_id.0 % num_turn == turn {
                let detector_collider =
                    Collider::new_idless(detector_radius.0, position.0);

                entity_detected.0.clear();
                intersection_pipeline
                    .snapshot
                    .intersect_collider_into(detector_collider, &mut entity_detected.0);
            }
        },
    );
}

/// Consume and apply the client's packets.
fn handle_client_inputs(
    mut query: Query<(&ClientId, &mut WishPosition), Without<ClientFleetAI>>,
    clients_res: Res<ClientsRes>,
    client_disconnected: ResMut<EventRes<ClientDisconnected>>,
    task_pool: Res<TaskPool>,
) {
    query.par_for_each_mut(&task_pool, 32, |(client_id, mut wish_position)| {
        if let Some(connection) = clients_res.connected_clients.get(&client_id) {
            loop {
                match connection.inbound_receiver.try_recv() {
                    Ok(payload) => match Packet::deserialize(&payload) {
                        Packet::Message { origin, content } => {
                            // TODO: Broadcast the message.
                        }
                        Packet::MetascapeWishPos { wish_pos } => {
                            wish_position.0 = Some(wish_pos);
                        }
                        Packet::BattlescapeInput {
                            wish_input,
                            last_acknowledge_command,
                        } => {
                            // TODO: Handle battlescape inputs.
                        }
                        _ => {
                            debug!("{:?} sent an invalid packet. Disconnecting...", client_id);
                            client_disconnected.events.push(ClientDisconnected {
                                client_id: *client_id,
                                send_packet: Some(Packet::DisconnectedReason(DisconnectedReasonEnum::InvalidPacket)),
                            });
                            break;
                        }
                    },
                    Err(err) => {
                        if err.is_disconnected() {
                            client_disconnected.events.push(ClientDisconnected {
                                client_id: *client_id,
                                send_packet: None,
                            });
                        }
                        break;
                    }
                }
            }
        }
    });
}

//* pre_update

/// Change the position of entities that have an orbit.
fn handle_orbit(
    mut query: Query<(&Orbit, &mut Position)>,
    time_res: Res<TimeRes>,
    task_pool: Res<TaskPool>,
) {
    let time = time_res.tick as f32;

    query.par_for_each_mut(&task_pool, 256, |(orbit, mut position)| {
        position.0 = orbit.to_position(time);
    })
}

/// Remove the orbit component from entities with velocity.
fn remove_orbit(
    mut commands: Commands,
    query: Query<(Entity, &Velocity), (Changed<Velocity>, With<Orbit>)>,
) {
    query.for_each(|(entity, velocity)| {
        if velocity.0.x != 0.0 || velocity.0.x != 0.0 {
            // Remove orbit as this entity has velocity.
            commands.entity(entity).remove::<Orbit>();
        }
    });
}

/// Ai that control the client's fleet while he is not connected.
fn client_fleet_ai(
    mut query: Query<(&ClientFleetAI, &Position, &mut WishPosition), With<ClientId>>,
    task_pool: Res<TaskPool>,
) {
    query.par_for_each_mut(
        &task_pool,
        256,
        |(client_fleet_ai, position, mut wish_position)| match client_fleet_ai.goal {
            ClientFleetAIGoal::Idle => {
                if wish_position.0.is_some() {
                    wish_position.0 = None;
                }
            }
        },
    )
}

/// TODO: Ai that control fleet owned by a colony.
fn colony_fleet_ai(
    mut query: Query<(&mut ColonyFleetAI, &Position, &mut WishPosition)>,
    task_pool: Res<TaskPool>,
) {
    query.par_for_each_mut(
        &task_pool,
        256,
        |(mut colony_fleet_ai, position, mut wish_position)| match &mut colony_fleet_ai.goal {
            ColonyFleetAIGoal::Trade { colony } => todo!(),
            ColonyFleetAIGoal::Guard { duration } => todo!(),
        },
    );
}

//* update

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity and friction.
///
/// TODO: Fleets engaged in the same Battlescape should aggregate.
fn apply_fleet_movement(
    mut query: Query<(&mut Position, &mut WishPosition, &mut Velocity, &DerivedFleetStats), Without<Orbit>>,
    metascape_parameters: Res<MetascapeParameters>,
    task_pool: Res<TaskPool>,
) {
    let bound_squared = metascape_parameters.bound.powi(2);

    query.par_for_each_mut(
        &task_pool,
        256,
        |(mut position, mut wish_position, mut velocity, derived_fleet_stats)| {
            if let Some(target) = wish_position.0 {
                // A vector equal to our current velocity toward our target.
                let wish_vel = target - position.0 - velocity.0;

                // Seek target.
                velocity.0 += wish_vel.clamp_length_max(derived_fleet_stats.acceleration);

                // Stop if we are near the target.
                if wish_vel.length_squared() < 0.5 {
                    wish_position.0 = None;
                }
            } else if velocity.0.x != 0.0 || velocity.0.y != 0.0 {
                // Go against current velocity.
                let vel_change = -velocity.0.clamp_length_max(derived_fleet_stats.acceleration);
                velocity.0 += vel_change;

                // Set velocity to zero if we have nearly no velocity.
                if velocity.0.x.abs() < 0.001 {
                    velocity.0.x = 0.0;
                }
                if velocity.0.y.abs() < 0.001 {
                    velocity.0.y = 0.0;
                }
            }

            // Entities are pushed away from the world's bound.
            if position.0.length_squared() > bound_squared {
                velocity.0 -= position.0.normalize();
            }

            // Apply friction.
            velocity.0 *= metascape_parameters.friction;

            // Apply velocity.
            position.0 += velocity.0;
        },
    );
}

/// Remove a client from connected client map and change its components.
fn disconnect_client(
    mut commands: Commands,
    mut clients_res: ResMut<ClientsRes>,
    fleets_res: Res<FleetsRes>,
    client_disconnected: Res<EventRes<ClientDisconnected>>,
) {
    while let Some(client_disconnected) = client_disconnected.events.pop() {
        let client_id = client_disconnected.client_id;

        // Remove connection.
        if let Some(connection) = clients_res.connected_clients.remove(&client_id) {
            if let Some(packet) = client_disconnected.send_packet {
                // Send last packet.
                connection.send_packet_reliable(packet.serialize());
                connection.flush_tcp_stream();
            }
            debug!("{:?} disconneced.", client_id);
        } else {
            warn!(
                "Got ClientDisconnected event, but {:?} can not be found. Ignoring...",
                client_id
            );
        }

        // Add fleet ai and remove components that are not needed for unconnected client.
        if let Some(client_data) = clients_res.clients_data.get(&client_id) {
            if let Some(entity) = fleets_res.spawned_fleets.get(&FleetId::from(client_id)) {
                commands
                    .entity(*entity)
                    .insert(client_data.client_fleet_ai)
                    .remove::<KnowEntities>();
            }
        } else {
            warn!(
                "Got ClientDisconnected event, but {:?}'s data can not be found. Ignoring...",
                client_id
            );
        }
    }
}

/// Are we in the past or the future now?
fn increment_time(mut time_res: ResMut<TimeRes>) {
    time_res.tick += 1;
}

//* post_update

/// Take a snapshot of the AccelerationStructure from the last update and request a new update on the runner thread.
///
/// This effectively just swap the snapshots between the runner thread and this IntersectionPipeline.
fn update_intersection_pipeline(
    query: Query<(Entity, &Position, &DetectedRadius)>,
    mut intersection_pipeline: ResMut<IntersectionPipeline>,
    mut last_update_delta: Local<u32>,
) {
    *last_update_delta += 1;

    if *last_update_delta > DETECTED_UPDATE_INTERVAL {
        // Take back the AccelerationStructure on the runner thread.
        match intersection_pipeline.update_result_receiver.try_recv() {
            Ok(mut runner) => {
                // Update all colliders.
                intersection_pipeline.snapshot.colliders.clear();
                query.for_each(|(entity, position, detected_radius)| {
                    let new_collider = Collider::new(
                        entity.id(),
                        detected_radius.0,
                        position.0,
                    );
                    intersection_pipeline.snapshot.colliders.push(new_collider);
                });

                // Swap snapshot.
                std::mem::swap(&mut intersection_pipeline.snapshot, &mut runner);

                // Return runner.
                if intersection_pipeline.update_request_sender.send(runner).is_err() {
                    warn!("Intersection pipeline update runner thread dropped. Creating a new runner...");
                    intersection_pipeline.start_new_runner_thread();
                }

                *last_update_delta = 0;
            }
            Err(err) => {
                if err == crossbeam::channel::TryRecvError::Disconnected {
                    warn!("Intersection pipeline update runner thread dropped. Creating a new runner...");
                    intersection_pipeline.start_new_runner_thread();
                }
                warn!("AccelerationStructure runner is taking longer than expected to update. Trying again latter...");
            }
        }
    }
}

/// Send detected fleet to clients.
fn send_detected_entity(
    mut query_client: Query<
        (Entity, &ClientId, &Position, &EntityDetected, &mut KnowEntities),
        Changed<EntityDetected>,
    >,
    query_changed_entity: Query<
        (&Position, &Velocity, &DerivedFleetStats, &WishPosition),
        Or<(Changed<WishPosition>, Changed<DerivedFleetStats>)>,
    >,
    time_res: Res<TimeRes>,
    clients_res: Res<ClientsRes>,
    task_pool: Res<TaskPool>,
    mut turn: Local<u8>,
) {
    *turn = turn.wrapping_add(1);

    query_client.par_for_each_mut(
        &task_pool,
        32,
        |(entity, client_id, position, entity_detected, know_entities)| {
            if let Some(connection) = clients_res.connected_clients.get(client_id) {
                for detected_entity in entity_detected.0.iter().map(|id| Entity::new(*id)) {}

                // let mut metascape_state_part = MetascapeStatePart {
                //     tick: time_res.tick,
                //     part: 0,
                //     entity_order_required: entity_order.id,
                //     relative_position: pos.0,
                //     entities_position: Vec::with_capacity(
                //         entity_order
                //             .current_entity_order
                //             .len()
                //             .min(MetascapeStatePart::NUM_ENTITIES_POSITION_MAX),
                //     ),
                // };

                // for detected_entity in entity_order
                //     .current_entity_order
                //     .iter()
                //     .map(|entity_id| Entity::new(*entity_id))
                // {
                //     // Add entity position.
                //     if let Ok(detected_pos) = query_entity.get(detected_entity) {
                //         metascape_state_part.entities_position.push(detected_pos.0 - pos.0);
                //     } else {
                //         metascape_state_part.entities_position.push(vec2(0.0, 1000.0));
                //         debug!("Could not find a detected entity. Sending made up position...");
                //     }

                //     if metascape_state_part.entities_position.len() >= MetascapeStatePart::NUM_ENTITIES_POSITION_MAX {
                //         // Send this part.
                //         let packet = UdpServer::MetascapeEntityPosition(metascape_state_part);
                //         let _ = client.connection.send_udp(packet.serialize());

                //         // Prepare next part.
                //         unsafe {
                //             if let UdpServer::MetascapeEntityPosition(p) = packet {
                //                 metascape_state_part = p;
                //             } else {
                //                 unreachable_unchecked();
                //             }
                //         }
                //         metascape_state_part.part += 1;
                //         metascape_state_part.entities_position.clear();
                //     }
                // }

                // if !metascape_state_part.entities_position.is_empty() {
                //     let packet = UdpServer::MetascapeEntityPosition(metascape_state_part);
                //     let _ = client.connection.send_udp(packet.serialize());
                // }
            }
        },
    );
}
