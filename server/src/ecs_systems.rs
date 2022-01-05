use crate::data_manager::DataManager;
use crate::ecs_components::*;
use crate::ecs_events::*;
use crate::res_clients::*;
use crate::res_fleets::*;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use common::fleet_movement::compute_fleet_movement;
use common::idx::*;
use common::intersection::*;
use common::packets::*;
use common::parameters::MetascapeParameters;
use common::position::*;
use common::res_time::TimeRes;
use glam::Vec2;

const DETECTED_UPDATE_INTERVAL: u32 = 5;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "first";
    schedule.add_stage(current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, get_new_clients.system());
    schedule.add_system_to_stage(current_stage, fleet_sensor.system());

    let previous_stage = current_stage;
    let current_stage = "pre_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
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
            let message = Packet::Message {
                origin: ClientId(0),
                content: "Someone else connected on the same account.".to_string(),
            };
            old_connection.send_packet(message.serialize(), true);
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
                        fleet_position: FleetPosition(Position::WorldPosition {
                            world_position: Vec2::ZERO,
                        }),
                        wish_position: WishPosition::default(),
                        velocity: Velocity::default(),
                        acceleration: Acceleration(0.1),
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
    mut query: Query<(&FleetId, &FleetPosition, &DetectorRadius, &mut EntityDetected)>,
    intersection_pipeline: Res<IntersectionPipeline>,
    task_pool: Res<TaskPool>,
    time_res: Res<TimeRes>,
) {
    // We will only update one part every tick.
    let turn = (time_res.tick % DETECTED_UPDATE_INTERVAL) as u64;
    let num_turn = DETECTED_UPDATE_INTERVAL as u64;

    let time = time_res.tick as f32;

    query.par_for_each_mut(
        &task_pool,
        64 * num_turn as usize,
        |(fleet_id, fleet_position, detector_radius, mut entity_detected)| {
            if fleet_id.0 % num_turn == turn {
                let detector_collider =
                    Collider::new_idless(detector_radius.0, fleet_position.0.to_world_position(time));

                entity_detected.0.clear();
                intersection_pipeline
                    .snapshot
                    .intersect_collider_into(detector_collider, &mut entity_detected.0);
            }
        },
    );
}

/// TODO: Apply the client's input to his fleet.
fn handle_client_inputs(
    mut query: Query<(&ClientId, &FleetPosition, &mut WishPosition), Without<ClientFleetAI>>,
    clients_res: Res<ClientsRes>,
    client_disconnected: ResMut<EventRes<ClientDisconnected>>,
    task_pool: Res<TaskPool>,
) {
    query.par_for_each_mut(&task_pool, 32, |(client_id, fleet_position, mut wish_position)| {
        if let Some(connection) = clients_res.connected_clients.get(&client_id) {
            loop {
                match connection.inbound_receiver.try_recv() {
                    Ok(payload) => match Packet::deserialize(&payload) {
                        Packet::Invalid => {
                            debug!("{:?} sent an invalid packet. Disconnecting...", client_id);
                            client_disconnected.events.push(ClientDisconnected {
                                client_id: *client_id,
                                send_packet: Some(Packet::DisconnectedReason(DisconnectedReasonEnum::InvalidPacket)),
                            });
                            break;
                        }
                        Packet::Message { origin, content } => todo!(),
                        Packet::MetascapeWishPos {
                            sequence_number,
                            wish_pos,
                        } => todo!(),
                        Packet::BattlescapeInput {
                            wish_input,
                            last_acknowledge_command,
                        } => todo!(),
                        _ => {}
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

/// Ai that control the client's fleet while he is not connected.
fn client_fleet_ai(
    mut query: Query<(&ClientFleetAI, &FleetPosition, &mut WishPosition), With<ClientId>>,
    task_pool: Res<TaskPool>,
) {
    query.par_for_each_mut(
        &task_pool,
        256,
        |(client_fleet_ai, fleet_position, mut wish_position)| match client_fleet_ai.goal {
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
    mut query: Query<(&mut ColonyFleetAI, &FleetPosition, &mut WishPosition)>,
    task_pool: Res<TaskPool>,
) {
    query.par_for_each_mut(
        &task_pool,
        256,
        |(mut colony_fleet_ai, fleet_position, mut wish_position)| match &mut colony_fleet_ai.goal {
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
    mut query: Query<(&mut FleetPosition, &mut WishPosition, &mut Velocity, &Acceleration)>,
    metascape_parameters: Res<MetascapeParameters>,
    time_res: Res<TimeRes>,
    task_pool: Res<TaskPool>,
) {
    let time = time_res.tick as f32;
    let metascape_parameters = &*metascape_parameters;

    query.par_for_each_mut(
        &task_pool,
        256,
        |(mut fleet_position, mut wish_position, mut velocity, acceleration)| {
            if let Some(world_position) = compute_fleet_movement(
                fleet_position.0,
                &mut velocity.0,
                wish_position.0,
                acceleration.0,
                time,
                metascape_parameters,
            ) {
                // Change fleet_position and trigger component change.
                fleet_position.0 = Position::WorldPosition { world_position };
            }
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
                connection.send_packet(packet.serialize(), true);
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
    query: Query<(Entity, &FleetPosition, &DetectedRadius)>,
    mut intersection_pipeline: ResMut<IntersectionPipeline>,
    time_res: Res<TimeRes>,
    mut last_update_delta: Local<u32>,
) {
    *last_update_delta += 1;

    if *last_update_delta > DETECTED_UPDATE_INTERVAL {
        // Take back the AccelerationStructure on the runner thread.
        match intersection_pipeline.update_result_receiver.try_recv() {
            Ok(mut runner) => {
                let time = time_res.tick as f32;

                // Update all colliders.
                intersection_pipeline.snapshot.colliders.clear();
                query.for_each(|(entity, fleet_position, detected_radius)| {
                    let new_collider =
                        Collider::new(entity.id(), detected_radius.0, fleet_position.0.to_world_position(time));
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
                if err == crossbeam_channel::TryRecvError::Disconnected {
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
        (Entity, &ClientId, &FleetPosition, &EntityDetected, &mut KnowEntities),
        Changed<EntityDetected>,
    >,
    query_entity: Query<(&FleetPosition, Option<&FleetId>), Changed<FleetPosition>>,
    time_res: Res<TimeRes>,
    clients_res: Res<ClientsRes>,
    task_pool: Res<TaskPool>,
) {
    query_client.par_for_each_mut(
        &task_pool,
        32,
        |(entity, client_id, fleet_position, entity_detected, know_entities)| {
            if let Some(connection) = clients_res.connected_clients.get(client_id) {

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
