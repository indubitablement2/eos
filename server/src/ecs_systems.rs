use crate::DetectedIntersectionPipeline;
use crate::SystemsAccelerationStructure;
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
use common::system::Systems;
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
    schedule.add_system_to_stage(current_stage, add_orbit.system());
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
    schedule.add_system_to_stage(current_stage, update_in_system.system());
    schedule.add_system_to_stage(current_stage, update_detected_intersection_pipeline.system());
    schedule.add_system_to_stage(current_stage, send_detected_entity.system());
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
            // Update old fleet components.
            commands
                .entity(*old_fleet_entity)
                .insert(KnowEntities::default())
                .insert(EntityDetected::default())
                .remove::<ClientFleetAI>();

            debug!("{:?} has taken back control of his fleet.", client_id);
        } else {
            // Create default client fleet.
            let entity = commands
                .spawn_bundle(ClientFleetBundle {
                    client_id,
                    know_entities: KnowEntities::default(),
                    fleet_bundle: FleetBundle {
                        name: Name(format!("{:?}", fleet_id)),
                        fleet_id,
                        position: Position(Vec2::ZERO),
                        in_system: InSystem(None),
                        wish_position: WishPosition::default(),
                        velocity: Velocity::default(),
                        idle_counter: IdleCounter(0),
                        derived_fleet_stats: DerivedFleetStats {
                            acceleration: 0.1,
                            max_speed: 2.0,
                        },
                        reputation: Reputation(0),
                        detected_radius: DetectedRadius(10.0),
                        detector_radius: DetectorRadius(50.0),
                        entity_detected: EntityDetected(Vec::new()),
                    },
                })
                .id();

            // Insert fleet.
            let _ = fleets_res.spawned_fleets.insert(fleet_id, entity);

            debug!("Created a new fleet for {:?} which he now control.", client_id);
        }
    }
}

/// Determine what each fleet can see.
fn fleet_sensor(
    mut query: Query<(&FleetId, &Position, &DetectorRadius, &mut EntityDetected)>,
    detected_intersection_pipeline: Res<DetectedIntersectionPipeline>,
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
                let detector_collider = Collider::new_idless(detector_radius.0, position.0);

                entity_detected.0.clear();
                detected_intersection_pipeline.0
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
    client_disconnected: Res<EventRes<ClientDisconnected>>,
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
fn handle_orbit(mut query: Query<(&Orbit, &mut Position)>, time_res: Res<TimeRes>, task_pool: Res<TaskPool>) {
    let time = time_res.tick as f32;

    query.par_for_each_mut(&task_pool, 256, |(orbit, mut position)| {
        position.0 = orbit.to_position(time);
    })
}

/// Remove the orbit component from entities with velocity.
fn remove_orbit(mut commands: Commands, query: Query<(Entity, &Velocity), (Changed<Velocity>, With<Orbit>)>) {
    query.for_each(|(entity, velocity)| {
        if velocity.0.x != 0.0 || velocity.0.x != 0.0 {
            // Remove orbit as this entity has velocity.
            commands.entity(entity).remove::<Orbit>();
        }
    });
}

/// Add orbit to idle entity within a system.
fn add_orbit(
    mut commands: Commands,
    query: Query<(&Position, &InSystem), Without<Orbit>>,
    systems: Res<Systems>,
    fleet_idle: Res<EventRes<FleetIdle>>,
) {
    while let Some(event) = fleet_idle.events.pop() {
        if let Ok((position, in_system)) = query.get(event.entity) {
            if let Some(system_id) = in_system.0 {
                let system = &systems.systems[system_id];

                let relative_pos = position.0 - system.position;
                let orbit_radius = relative_pos.length();
                
                let mut orbit_time = Orbit::DEFAULT_ORBIT_TIME;
                // Check if there is a body nearby we should copy its orbit time.
                system.bodies.iter().fold(999.0f32, |closest, body| {
                    if (body.orbit.orbit_radius - orbit_radius).abs() < closest {
                        orbit_time = body.orbit.orbit_time;
                        body.orbit.orbit_radius
                    } else {
                        closest
                    }
                });

                // Add orbit as this entity has no velocity.
                commands.entity(event.entity).insert(Orbit {
                    origin: system.position,
                    orbit_radius,
                    orbit_start_angle: relative_pos.y.atan2(relative_pos.x),
                    orbit_time,
                });
            }
        }
    }
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
            ClientFleetAIGoal::Flee => todo!(),
        },
    )
}

/// TODO: Ai that control fleet owned by a colony.
fn colony_fleet_ai(mut query: Query<(&mut ColonyFleetAI, &Position, &mut WishPosition)>, task_pool: Res<TaskPool>) {
    query.par_for_each_mut(&task_pool, 256, |(mut colony_fleet_ai, position, mut wish_position)| {
        match &mut colony_fleet_ai.goal {
            ColonyFleetAIGoal::Trade { colony } => todo!(),
            ColonyFleetAIGoal::Guard { duration } => todo!(),
        }
    });
}

//* update

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity and friction.
///
/// TODO: Fleets engaged in the same Battlescape should aggregate.
fn apply_fleet_movement(
    mut query: Query<(Entity, &mut Position, &mut WishPosition, &mut Velocity, &DerivedFleetStats, &mut IdleCounter), Without<Orbit>>,
    metascape_parameters: Res<MetascapeParameters>,
    fleet_idle: Res<EventRes<FleetIdle>>,
    task_pool: Res<TaskPool>,
) {
    let bound_squared = metascape_parameters.bound.powi(2);

    query.par_for_each_mut(
        &task_pool,
        256,
        |(entity, mut position, mut wish_position, mut velocity, derived_fleet_stats, mut idle_counter)| {
            if let Some(target) = wish_position.0 {
                // A vector equal to our current velocity toward our target.
                let wish_vel = target - position.0 - velocity.0;

                // Seek target.
                velocity.0 += wish_vel.clamp_length_max(derived_fleet_stats.acceleration);

                // Stop if we are near the target.
                if wish_vel.length_squared() < 0.5 {
                    wish_position.0 = None;
                }

                // Fleet is not idle.
                idle_counter.0 = 0;
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

                // Fleet is not idle.
                idle_counter.0 = 0;
            } else {
                // Fleet is idle.
                idle_counter.0 += 1;
                if idle_counter.0 > IdleCounter::IDLE_DELAY {
                    fleet_idle.events.push(FleetIdle { entity });
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
                let mut entity_cmd = commands.entity(*entity);
                entity_cmd.insert(client_data.client_fleet_ai)
                    .remove::<KnowEntities>();
                if let ClientFleetAIGoal::Idle = client_data.client_fleet_ai.goal {
                    entity_cmd.remove::<EntityDetected>();
                }
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

/// Update the system each entity is currently in.
fn update_in_system(
    mut query: Query<(&FleetId, &Position, &mut InSystem)>,
    systems: Res<Systems>,
    systems_acceleration_structure: Res<SystemsAccelerationStructure>,
    task_pool: Res<TaskPool>,
    mut turn: Local<u64>,
) {
    *turn = (*turn + 1) % 10;

    query.par_for_each_mut(&task_pool, 2048, |(fleet_id, position, mut in_system)| {
        if fleet_id.0 % 10 == *turn {
            match in_system.0 {
                Some(system_id) => {
                    let system = &systems.systems[system_id];
                    if system.position.distance_squared(position.0) > system.bound.powi(2) {
                        in_system.0 = None;
                    }
                }
                None => {
                    if let Some(id) = systems_acceleration_structure.0.intersect_point_single(position.0) {
                        in_system.0 = Some(SystemId(id as u16));
                    }
                }
            }
        }
    })
}

/// Take a snapshot of the AccelerationStructure from the last update and request a new update on the runner thread.
///
/// This effectively just swap the snapshots between the runner thread and this IntersectionPipeline.
fn update_detected_intersection_pipeline(
    query: Query<(Entity, &Position, &DetectedRadius)>,
    mut detected_intersection_pipeline: ResMut<DetectedIntersectionPipeline>,
    mut last_update_delta: Local<u32>,
) {
    *last_update_delta += 1;
    let intersection_pipeline = &mut detected_intersection_pipeline.0;

    if intersection_pipeline.outdated.is_none() {
        // Take back the AccelerationStructure on the runner thread.
        match intersection_pipeline.update_result_receiver.try_recv() {
            Ok(mut new_scapshot) => {
                std::mem::swap(&mut intersection_pipeline.snapshot, &mut new_scapshot);
                intersection_pipeline.outdated = Some(new_scapshot);
            }
            Err(err) => {
                if err.is_disconnected() {
                    warn!("Detected intersection pipeline update runner thread dropped. Creating a new runner...");
                    intersection_pipeline.start_new_runner_thread();
                }
            }
        }
    }

    if *last_update_delta > DETECTED_UPDATE_INTERVAL {
        if let Some(mut old_snapshot) = intersection_pipeline.outdated.take() {
            // Update all colliders.
            old_snapshot.colliders.clear();
            query.for_each(|(entity, position, detected_radius)| {
                let new_collider = Collider::new(entity.id(), detected_radius.0, position.0);
                old_snapshot.colliders.push(new_collider);
            });

            // Send snapshot to be updated.
            if let Err(err) = intersection_pipeline.update_request_sender.send(old_snapshot) {
                warn!("Detected intersection pipeline update runner thread dropped. Creating a new runner...");
                intersection_pipeline.outdated = Some(err.0);
                intersection_pipeline.start_new_runner_thread();
            }

            *last_update_delta = 0;
        } else {
            warn!("AccelerationStructure runner is taking longer than expected to update. Trying again latter...");
        }
    }
}

/// Send detected fleet to clients.
fn send_detected_entity(
    mut query_client: Query<(Entity, &ClientId, &Position, &EntityDetected, &mut KnowEntities)>,
    query_changed_entity: Query<Entity, Changed<Orbit>>,
    query_fleet_info: Query<(&FleetId, &Name, Option<&Orbit>)>,
    query_entity_state: Query<&Position, Without<Orbit>>,
    time_res: Res<TimeRes>,
    clients_res: Res<ClientsRes>,
    task_pool: Res<TaskPool>,
) {
    query_client.par_for_each_mut(
        &task_pool,
        64,
        |(entity, client_id, position, entity_detected, mut know_entities)| {
            if let Some(connection) = clients_res.connected_clients.get(client_id) {
                let mut updated = Vec::with_capacity(entity_detected.0.len());
                let mut entities_info = Vec::new();
                let know_entities = &mut *know_entities;

                for detected_entity in entity_detected.0.iter().map(|id| Entity::new(*id)) {
                    if let Some(temp_id) = know_entities.known.remove(&detected_entity) {
                        // Client already know about this entity.
                        // Check if the entity infos changed. Otherwise do nothing.
                        if query_changed_entity.get(detected_entity).is_ok() {
                            // TODO: This should be a function that write directly into a buffer. (1)
                            if let Ok((fleet_id, name, orbit)) = query_fleet_info.get(detected_entity) {
                                entities_info.push((
                                    temp_id,
                                    EntityInfo::Fleet(FleetInfo {
                                        name: name.0.clone(),
                                        fleet_id: fleet_id.to_owned(),
                                        composition: Vec::new(),
                                        orbit: orbit.cloned(),
                                    }),
                                ));
                            }
                            // TODO: Try to query other type of entity.
                        }

                        updated.push((detected_entity, temp_id));
                    } else {
                        // Send a new entity.
                        if let Some(temp_id) = know_entities.free_idx.pop_front() {
                            // TODO: This should be a function that write directly into a buffer. (2)
                            if let Ok((fleet_id, name, orbit)) = query_fleet_info.get(detected_entity) {
                                entities_info.push((
                                    temp_id,
                                    EntityInfo::Fleet(FleetInfo {
                                        name: name.0.clone(),
                                        fleet_id: fleet_id.to_owned(),
                                        composition: Vec::new(),
                                        orbit: orbit.cloned(),
                                    }),
                                ));
                            }

                            updated.push((detected_entity, temp_id));
                        }
                    }
                }

                // Ask the client to remove uneeded entities.
                for (_, temp_id) in know_entities.known.drain() {
                    entities_info.push((temp_id, EntityInfo::Remove));
                    know_entities.free_idx.push_back(temp_id);
                }

                // Update known map.
                know_entities.known.extend(updated.into_iter());

                // Check if we should update the client's fleet.
                let client_fleet_info = if query_changed_entity.get(entity).is_ok() {
                    // TODO: This should be a function that write directly into a buffer. (3)
                    if let Ok((fleet_id, name, orbit)) = query_fleet_info.get(entity) {
                        Some(FleetInfo {
                            name: name.0.clone(),
                            fleet_id: fleet_id.to_owned(),
                            composition: Vec::new(),
                            orbit: orbit.cloned(),
                        })
                    } else {
                        warn!(
                            "{:?} does not return a result when queried for fleet info. Ignoring...",
                            client_id
                        );
                        None
                    }
                } else {
                    None
                };

                // Send new entities info.
                let packet = Packet::EntitiesInfo {
                    tick: time_res.tick,
                    client_fleet_info,
                    infos: entities_info,
                }
                .serialize();
                connection.send_packet_reliable(packet);

                // Send entities state.
                let packet = if know_entities.known.len() > 32 {
                    // Large state.
                    let mut bitfield = [0u8; 32];
                    let relative_entities_position = know_entities
                        .known
                        .iter()
                        .filter_map(|(entity, temp_id)| {
                            if let Ok(entity_position) = query_entity_state.get(*entity) {
                                // Set updated bit.
                                let byte = temp_id / 8;
                                let bit = temp_id % 8 + 1;
                                bitfield[byte as usize] |= bit;

                                Some(entity_position.0)
                            } else {
                                None
                            }
                        })
                        .collect();

                    Packet::EntitiesStateLarge {
                        tick: time_res.tick,
                        client_entity_position: position.0,
                        bitfield,
                        relative_entities_position,
                    }
                    .serialize()
                } else {
                    // Small state.
                    let relative_entities_position = know_entities
                        .known
                        .iter()
                        .filter_map(|(entity, temp_id)| {
                            if let Ok(entity_position) = query_entity_state.get(*entity) {
                                Some((*temp_id, entity_position.0))
                            } else {
                                None
                            }
                        })
                        .collect();

                    Packet::EntitiesStateSmall {
                        tick: time_res.tick,
                        client_entity_position: position.0,
                        relative_entities_position,
                    }
                    .serialize()
                };

                // TODO: Reuse this buffer to write entities infos.
                connection.send_packet_unreliable(&packet);

                // Flush tcp buffer.
                connection.flush_tcp_stream();
            }
        },
    );
}
