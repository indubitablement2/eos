use crate::data_manager::DataManager;
use crate::ecs_components::*;
use crate::ecs_events::*;
use crate::intersection::*;
use crate::res_clients::*;
use crate::res_fleets::*;
use crate::res_parameters::ParametersRes;
use crate::res_times::TimeRes;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use common::packets::*;
use glam::Vec2;
use rand::Rng;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "first";
    schedule.add_stage(current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, increment_time.system());
    schedule.add_system_to_stage(current_stage, get_new_clients.system());
    schedule.add_system_to_stage(current_stage, fleet_sensor.system());

    let previous_stage = current_stage;
    let current_stage = "pre_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, fleet_ai.system());

    let previous_stage = current_stage;
    let current_stage = "update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, update_collider_position.system());
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
    fleet_intersection_pipeline: Res<FleetIntersectionPipeline>,
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
            todo!();
        } else {
            // TODO: Load or create client's fleet.
            let fleet_entity = commands
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
                        fleet_collider: FleetCollider(fleet_intersection_pipeline.insert_collider(
                            Collider {
                                radius: 10.0,
                                position: Vec2::ZERO,
                            },
                            fleet_id.0,
                        )),
                        reputation: Reputation(0),
                        detector_radius: DetectorRadius(30.0),
                        fleet_detected: FleetDetected(Vec::new()),
                    },
                })
                .id();

            // Insert fleet.
            let _ = fleets_res.spawned_fleets.insert(fleet_id, fleet_entity);
        }

        // Trigger event.
        client_connected.events.push(ClientConnected { client_id });
    }
}

/// Determine what each fleet can see.
fn fleet_sensor(
    mut query: Query<(&Position, &FleetId, &DetectorRadius, &mut FleetDetected)>,
    fleet_intersection_pipeline: Res<FleetIntersectionPipeline>,
    fleets_res: Res<FleetsRes>,
    task_pool: Res<TaskPool>,
    time_res: Res<TimeRes>,
) {
    // We will only update 1/5 at a time.
    let turn = time_res.tick % 5;

    query.par_for_each_mut(&task_pool, 512, |(pos, fleet_id, detector_radius, mut detected)| {
        if fleet_id.0 % 5 == turn {
            detected.0.clear();

            let detector_collider = Collider {
                radius: detector_radius.0,
                position: pos.0,
            };

            for collider_id in fleet_intersection_pipeline.intersect_collider(detector_collider) {
                if let Some(custom_data) = fleet_intersection_pipeline.get_collider_custom_data(collider_id) {
                    let detected_fleet_id = FleetId(custom_data);
                    if let Some(entity) = fleets_res.spawned_fleets.get(&detected_fleet_id) {
                        detected.0.push(*entity);
                    } else {
                        // It can happen that a fleet is removed from spawned fleet, but not from the intersection pipeline yet.
                        debug!(
                            "{:?} is detected, but is not spawned This should not happens often. Ignoring...",
                            detected_fleet_id
                        );
                    }
                } else {
                    warn!("Collider inside FleetIntersectionPipeline does not have custom data. Ignoring...");
                }
            }
        }
    });
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

fn update_collider_position(
    query: Query<(&Position, &FleetCollider)>,
    fleet_intersection_pipeline: Res<FleetIntersectionPipeline>
) {
    query.for_each(|(pos, fleet_collider)| {
        if let Some(old_collider) = fleet_intersection_pipeline.get_collider(fleet_collider.0) {
            let new_collider = Collider {
                radius: old_collider.radius,
                position: pos.0,
            };
            fleet_intersection_pipeline.modify_collider(fleet_collider.0, new_collider);
        }
    })
    
}

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

fn apply_velocity(query: Query<(&mut Position, &mut Velocity)>, params: Res<ParametersRes>) {
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
fn spawn_ai_fleet(
    time_res: Res<TimeRes>,
    mut commands: Commands,
    mut fleets_res: ResMut<FleetsRes>,
    fleet_intersection_pipeline: Res<FleetIntersectionPipeline>,
) {
    if time_res.tick % 10 != 0 {
        return;
    }

    let fleet_id = fleets_res.get_new_fleet_id();

    let fleet_entity = commands
        .spawn_bundle(FleetBundle {
            fleet_id,
            position: Position(Vec2::ZERO),
            wish_position: WishPosition(Vec2::ZERO),
            velocity: Velocity(Vec2::ZERO),
            acceleration: Acceleration(0.1),
            fleet_ai: FleetAI {
                goal: FleetGoal::Wandering { new_pos_timer: 0 },
            },
            fleet_collider: FleetCollider(fleet_intersection_pipeline.insert_collider(
                Collider {
                    radius: 10.0,
                    position: Vec2::ZERO,
                },
                fleet_id.0,
            )),
            reputation: Reputation(0),
            detector_radius: DetectorRadius(10.0),
            fleet_detected: FleetDetected(Vec::new()),
        })
        .id();

    // Insert fleet.
    let _ = fleets_res.spawned_fleets.insert(fleet_id, fleet_entity);

    info!("{} fleets spawned.", fleets_res.spawned_fleets.len());
}

//* last

fn update_intersection_pipeline(
    mut fleet_intersection_pipeline: ResMut<FleetIntersectionPipeline>,
    time_res: Res<TimeRes>,
) {
    if time_res.tick % 5 == 0 {
        fleet_intersection_pipeline.update();
    }
}

// Send detected fleet to clients over udp.
fn send_detected_fleet(
    query_client: Query<(&ClientId, &Position, &FleetDetected)>,
    query_fleet: Query<&Position>,
    time_res: Res<TimeRes>,
    clients_res: Res<ClientsRes>,
) {
    query_client.for_each(|(client_id, pos, fleet_detected)| {
        if let Some(client) = clients_res.connected_clients.get(client_id) {
            let mut fleets_position = Vec::with_capacity(25);

            // Always send client's own fleet position.
            fleets_position.push(pos.0);

            // TODO: If too many fleet are detected, throttle which ones are sent.
            for detected_entity in fleet_detected.0.iter() {
                if let Ok(detected_fleet_pos) = query_fleet.get(*detected_entity) {
                    fleets_position.push(detected_fleet_pos.0);
                    if fleets_position.len() >= 24 {
                        debug!("Could not send all detected fleet position. Ignoring rest...");
                        break;
                    }
                }
            }

            let packet = UdpServer::Metascape {
                fleets_position,
                tick: time_res.tick,
            };

            let _ = client.connection.udp_sender.blocking_send(packet);
        }
    });
}

// /// TODO: Send unacknowledged commands.
// fn send_udp(query: Query<(&FleetId, &Position)>, time_res: Res<TimeRes>, clients_res: Res<ClientsRes>) {
// }
