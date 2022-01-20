use crate::data_manager::ClientData;
use crate::data_manager::DataManager;
use crate::ecs_components::*;
use crate::ecs_events::*;
use crate::res_clients::*;
use crate::res_fleets::*;
use crate::DetectedIntersectionPipeline;
use crate::SystemsAccelerationStructure;
use bevy_ecs::prelude::*;
use bevy_tasks::ComputeTaskPool;
use common::factions::Factions;
use common::idx::FleetId;
use common::idx::PlanetId;
use common::intersection::*;
use common::orbit::*;
use common::packets::*;
use common::parameters::Parameters;
use common::systems::Systems;
use common::time::Time;
use glam::Vec2;
use rand::thread_rng;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum Label {
    Movement,
    DetectedPipelineUpdate,
}

const DETECTED_UPDATE_INTERVAL: u64 = 5;
/// Minimum delay before a disconnected client's fleet get removed.
const DISCONNECT_REMOVE_FLEET_DELAY: u32 = 200;

pub fn add_systems(schedule: &mut Schedule) {
    schedule.add_stage("", SystemStage::parallel());

    schedule.add_system_to_stage("", get_new_clients);

    schedule.add_system_to_stage("", spawn_colonist);

    schedule.add_system_to_stage("", handle_orbit);
    schedule.add_system_to_stage("", remove_orbit);
    schedule.add_system_to_stage("", handle_idle);

    schedule.add_system_to_stage("", update_in_system);

    schedule.add_system_to_stage("", handle_client_inputs.before(Label::Movement));
    schedule.add_system_to_stage("", colony_fleet_ai.before(Label::Movement));
    schedule.add_system_to_stage("", colonist_ai.before(Label::Movement));

    schedule.add_system_to_stage("", apply_fleet_movement.label(Label::Movement));

    schedule.add_system_to_stage("", fleet_sensor.before(Label::DetectedPipelineUpdate));
    schedule.add_system_to_stage(
        "",
        update_detected_intersection_pipeline
            .label(Label::DetectedPipelineUpdate)
            .after(Label::Movement),
    );

    schedule.add_system_to_stage("", send_detected_entity.after(Label::Movement));

    schedule.add_system_to_stage("", disconnect_client);
    schedule.add_system_to_stage("", remove_fleet);
}

/// Get new connection and insert client.
fn get_new_clients(
    mut commands: Commands,
    mut query_client_fleet: Query<(&ClientIdComp, &mut KnowEntities)>,
    mut clients_res: ResMut<ClientsRes>,
    mut fleets_res: ResMut<FleetsRes>,
    mut data_manager: ResMut<DataManager>,
) {
    // TODO: Only do a few each tick.
    // TODO: Send notice to the rest of the wait queue.
    while let Ok(connection) = clients_res
        .connection_manager
        .new_connection_receiver
        .try_recv()
    {
        let client_id = connection.client_id;
        let fleet_id = client_id.to_fleet_id();

        // Insert client.
        if let Some(old_connection) = clients_res.connected_clients.insert(client_id, connection) {
            debug!(
                "{:?} was disconnected as a new connection took this client.",
                client_id
            );
            // Send message to old client explaining why he got disconnected.
            old_connection.send_packet_reliable(
                Packet::DisconnectedReason(DisconnectedReasonEnum::ConnectionFromOther).serialize(),
            );
            old_connection.flush_tcp_stream();
        }

        // Check if client has data.
        match data_manager
            .clients_data
            .try_insert(client_id, ClientData::default())
        {
            Ok(client_data) => {
                // This is this client's first login.
            }
            Err(client_data) => {
                // Old client.
            }
        }

        // Check if fleet is already spawned.
        if let Some(old_fleet_entity) = fleets_res.spawned_fleets.get(&fleet_id) {
            if let Ok((client_id_comp, mut know_entities)) =
                query_client_fleet.get_mut(*old_fleet_entity)
            {
                // Update old fleet components.
                let know_entities = &mut *know_entities;
                *know_entities = KnowEntities::default();
                commands.entity(*old_fleet_entity).remove::<QueueRemove>();

                if client_id_comp.0 != client_id {
                    error!(
                        "{:?} was asigned {:?}'s fleet. Fleets res and world do not match.",
                        client_id, client_id_comp.0
                    );
                } else {
                    debug!("{:?} has taken back control of his fleet.", client_id);
                }
            } else {
                fleets_res.spawned_fleets.remove(&fleet_id);
                error!(
                    "{:?}'s fleet is in fleets res, but is not found in world. Removing from spawned fleets...",
                    client_id
                );
            }
        } else {
            // Create default client fleet.
            let entity = commands
                .spawn_bundle(ClientFleetBundle {
                    client_id_comp: ClientIdComp(client_id),
                    know_entities: KnowEntities::default(),
                    fleet_bundle: FleetBundle {
                        name: Name(format!("{:?}", fleet_id)),
                        fleet_id_comp: FleetIdComp(fleet_id),
                        position: Position(Vec2::ZERO),
                        in_system: InSystem(None),
                        wish_position: WishPosition::default(),
                        velocity: Velocity::default(),
                        idle_counter: IdleCounter(0),
                        derived_fleet_stats: DerivedFleetStats { acceleration: 0.04 },
                        reputations: Reputations::default(),
                        detected_radius: DetectedRadius(10.0),
                        detector_radius: DetectorRadius(50.0),
                        entity_detected: EntityDetected(Vec::new()),
                    },
                })
                .id();

            // Insert fleet.
            let _ = fleets_res.spawned_fleets.insert(fleet_id, entity);

            debug!(
                "Created a new fleet for {:?} which he now control.",
                client_id
            );
        }
    }
}

/// Spawn colonist fleet to reach faction's target colony.
fn spawn_colonist(
    mut commands: Commands,
    factions: Res<Factions>,
    mut fleets_res: ResMut<FleetsRes>,
    time: Res<Time>,
) {
    // TODO: Use run criteria.
    if time.tick % 10 != 0 {
        return;
    }

    for (faction_id, faction) in factions.factions.iter() {
        if faction.colonies.len() < faction.target_colonies {
            let faction_id = *faction_id;
            let fleet_id = fleets_res.get_new_fleet_id();
            let entity = commands.spawn().insert_bundle(ColonistAIFleetBundle::new(None, time.tick + 3000, fleet_id, Vec2::ZERO, Some(faction_id))).id();
            fleets_res.spawned_fleets.insert(fleet_id, entity);

        }
    }
}

/// Determine what each fleet can see.
fn fleet_sensor(
    mut query: Query<(
        Entity,
        &FleetIdComp,
        &Position,
        &DetectorRadius,
        &mut EntityDetected,
        &Reputations,
    )>,
    query_reputation: Query<&Reputations>,
    detected_intersection_pipeline: Res<DetectedIntersectionPipeline>,
    time: Res<Time>,
    factions: Res<Factions>,
) {
    // We will only update one part every tick.
    let turn = time.tick as u64 % DETECTED_UPDATE_INTERVAL;

    query.for_each_mut(
        |(entity, fleet_id_comp, position, detector_radius, mut entity_detected, reputations)| {
            if fleet_id_comp.0 .0 % DETECTED_UPDATE_INTERVAL == turn {
                let detector_collider = Collider::new(detector_radius.0, position.0);

                detected_intersection_pipeline
                    .0
                    .snapshot
                    .intersect_collider_into(detector_collider, &mut entity_detected.0);

                // Filter the result.
                if fleet_id_comp.0.is_client() {
                    // Client fleet filter out themself.
                    for i in 0..entity_detected.0.len() {
                        if entity_detected.0[i] == entity {
                            entity_detected.0.swap_remove(i);
                            break;
                        }
                    }
                } else {
                        // AI fleet filter out allied.
                        entity_detected.0.drain_filter(|id| {
                            if let Ok(other_rep) = query_reputation.get(*id) {
                                !reputations.get_relative_reputation(other_rep, &factions.factions)
                                    .is_enemy()
                            } else {
                                debug!("An entity has a Detector, but no Reputations. Ignoring...");
                                true
                            }
                        });
                }
            }
        },
    );
}

/// Consume and apply the client's packets.
fn handle_client_inputs(
    mut query: Query<(Entity, &ClientIdComp, &mut WishPosition)>,
    clients_res: Res<ClientsRes>,
    client_disconnected: Res<EventRes<ClientDisconnected>>,
) {
    query.for_each_mut(|(entity, client_id_comp, mut wish_position)| {
        if let Some(connection) = clients_res.connected_clients.get(&client_id_comp.0) {
            loop {
                match connection.inbound_receiver.try_recv() {
                    Ok(payload) => match Packet::deserialize(&payload) {
                        Packet::Message { origin, content } => {
                            // TODO: Broadcast the message.
                        }
                        Packet::MetascapeWishPos { wish_pos } => {
                            wish_position.to = Some(wish_pos);
                        }
                        Packet::BattlescapeInput {
                            wish_input,
                            last_acknowledge_command,
                        } => {
                            // TODO: Handle battlescape inputs.
                        }
                        _ => {
                            debug!(
                                "{:?} sent an invalid packet. Disconnecting...",
                                client_id_comp.0
                            );
                            client_disconnected.events.push(ClientDisconnected {
                                client_id: client_id_comp.0,
                                fleet_entity: entity,
                                send_packet: Some(Packet::DisconnectedReason(
                                    DisconnectedReasonEnum::InvalidPacket,
                                )),
                            });
                            break;
                        }
                    },
                    Err(err) => {
                        if err.is_disconnected() {
                            client_disconnected.events.push(ClientDisconnected {
                                client_id: client_id_comp.0,
                                fleet_entity: entity,
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

/// Change the position of entities that have an orbit.
fn handle_orbit(mut query: Query<(&OrbitComp, &mut Position)>, time: Res<Time>) {
    let time = time.as_time();

    query.for_each_mut(|(orbit_comp, mut position)| {
        position.0 = orbit_comp.0.to_position(time);
    })
}

/// Remove the orbit component from entities with velocity.
fn remove_orbit(
    mut commands: Commands,
    query: Query<(Entity, &Velocity), (Changed<Velocity>, With<OrbitComp>)>,
) {
    query.for_each(|(entity, velocity)| {
        if velocity.0.x != 0.0 || velocity.0.x != 0.0 {
            // Remove orbit as this entity has velocity.
            commands.entity(entity).remove::<OrbitComp>();
        }
    });
}

/// Add orbit to idle entity within a system..
fn handle_idle(
    mut commands: Commands,
    query: Query<(&Position, &InSystem)>,
    systems: Res<Systems>,
    fleet_idle: Res<EventRes<FleetIdle>>,
    time: Res<Time>,
) {
    let time = time.as_time();
    while let Some(event) = fleet_idle.events.pop() {
        if let Ok((position, in_system)) = query.get(event.entity) {
            if let Some(system_id) = in_system.0 {
                // Add orbit.
                let system = if let Some(system) = systems.systems.get(&system_id) {
                    system
                } else {
                    warn!("Can not find {:?}. Ignoring...", system_id);
                    continue;
                };

                let relative_position = position.0 - system.position;
                let distance = relative_position.length();

                let mut orbit_speed = 0.0;
                // Check if there is a body nearby we should copy its orbit time.
                system.planets.iter().fold(999.0f32, |closest, planet| {
                    let dif = (planet.relative_orbit.distance - distance).abs();
                    if dif < closest {
                        orbit_speed = planet.relative_orbit.orbit_speed;
                        dif
                    } else {
                        closest
                    }
                });

                // Add orbit as this entity has no velocity.
                commands
                    .entity(event.entity)
                    .insert(OrbitComp(Orbit::from_relative_position(
                        relative_position,
                        time,
                        system.position,
                        distance,
                        orbit_speed,
                    )));
            } else {
                // Add a stationary orbit.
                commands
                    .entity(event.entity)
                    .insert(OrbitComp(Orbit::stationary(position.0)));
            }
        }
    }
}

/// TODO: Ai that control fleet owned by a colony.
fn colony_fleet_ai(mut query: Query<(&mut ColonyFleetAI, &Position, &mut WishPosition)>) {
    query.for_each_mut(|(mut colony_fleet_ai, position, mut wish_position)| {
        match &mut colony_fleet_ai.goal {
            ColonyFleetAIGoal::Trade { colony } => todo!(),
            ColonyFleetAIGoal::Guard { duration } => todo!(),
        }
    });
}

fn colonist_ai(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Position,
        &InSystem,
        &mut ColonistAI,
        &mut WishPosition,
        &Reputations,
    )>,
    mut systems: ResMut<Systems>,
    mut factions: ResMut<Factions>,
    time: Res<Time>,
    parameters: Res<Parameters>,
) {
    let timef = time.as_time();
    let mut rng = thread_rng();

    query.for_each_mut(
        |(entity, position, in_system, mut colonist_ai, mut wish_position, reputations)| {
            if let Some(planet_id) = colonist_ai.target_planet {
                if let Some(system) = systems.systems.get_mut(&planet_id.system_id) {
                    if let Some(planet) = system.planets.get_mut(planet_id.planets_offset as usize)
                    {
                        // Check that planet has not been colonized already.
                        if planet.faction.is_some() {
                            colonist_ai.target_planet = None;
                        }

                        let planet_position =
                            planet.relative_orbit.to_position(timef, system.position);
                        wish_position.to = Some(planet_position);
                        wish_position.speed_multiplier = 0.3;

                        // Are we close enough to the planet to take it?
                        if planet_position.distance_squared(position.0) < 1.0 {
                            if let Some(faction_id) = reputations.faction {
                                if let Some(faction) = factions.factions.get_mut(&faction_id) {
                                    // Take control of the planet.
                                    // TODO: This should take time.
                                    planet.faction = Some(faction_id);
                                    faction.colonies.insert(planet_id);

                                    // TODO: Change AI to guard the planet.
                                } else {
                                    // Faction does not exist.
                                    commands.entity(entity).insert(QueueRemove { when: 0 });
                                }
                            } else {
                                // Fleet is factionless and can not colonize.
                                commands.entity(entity).insert(QueueRemove { when: 0 });
                            }
                        }
                    } else {
                        colonist_ai.target_planet = None;
                    }
                } else {
                    colonist_ai.target_planet = None;
                }
            } else if colonist_ai.travel_until < time.tick {
                if let Some(system_id) = in_system.0 {
                    if let Some(system) = systems.systems.get(&system_id) {
                        // Randomly chose a planet to colonize.
                        let planets_offset = (rng.gen::<u32>() % system.planets.len() as u32) as u8;
                        let planet_id = PlanetId {
                            system_id,
                            planets_offset,
                        };
                        let planet = &system.planets[planets_offset as usize];
                        if planet.faction.is_none() {
                            colonist_ai.target_planet = Some(planet_id);
                        } else {
                            // TODO: Go to a random system.
                            colonist_ai.travel_until = time.tick + 1200
                        }
                    }
                }
            } else {
                if wish_position.to.is_none() {
                    // TODO: Go to a random system.
                    // Go to a random position.
                    let v = (rng.gen::<Vec2>() * 2.0 - Vec2::ONE) * parameters.world_bound;
                    if v.length_squared() > parameters.world_bound * parameters.world_bound {
                        return;
                    }
                    wish_position.to = Some(v);
                    wish_position.speed_multiplier = 0.6;
                }
            }
        },
    );
}

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity and friction.
///
/// TODO: Fleets engaged in the same Battlescape should aggregate.
fn apply_fleet_movement(
    mut query: Query<(
        Entity,
        &mut Position,
        &mut WishPosition,
        &mut Velocity,
        &DerivedFleetStats,
        &mut IdleCounter,
    )>,
    parameters: Res<Parameters>,
    fleet_idle: Res<EventRes<FleetIdle>>,
) {
    let bound_squared = parameters.world_bound.powi(2);

    query.for_each_mut(
        |(
            entity,
            mut position,
            mut wish_position,
            mut velocity,
            derived_fleet_stats,
            mut idle_counter,
        )| {
            if let Some(target) = wish_position.to {
                // A vector equal to our current velocity toward our target.
                let wish_vel = target - position.0 - velocity.0;

                // Seek target.
                velocity.0 += wish_vel.clamp_length_max(
                    derived_fleet_stats.acceleration * wish_position.speed_multiplier,
                );

                // Stop if we are near the target.
                if wish_vel.length_squared() < 0.5 {
                    wish_position.to = None;
                }

                // Fleet is not idle.
                idle_counter.0 = 0;
            } else if velocity.0.x != 0.0 || velocity.0.y != 0.0 {
                // Go against current velocity.
                let vel_change = -velocity
                    .0
                    .clamp_length_max(derived_fleet_stats.acceleration);
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
            } else if !idle_counter.is_idle() {
                idle_counter.0 += 1;
                if idle_counter.just_stated_idling() {
                    // Fire idle event only once.
                    fleet_idle.events.push(FleetIdle { entity });
                }
            }

            // Entities are pushed away from the world's bound.
            if position.0.length_squared() > bound_squared {
                velocity.0 -= position.0.normalize();
            }

            // Apply friction.
            velocity.0 *= parameters.friction;

            // Apply velocity.
            position.0 += velocity.0;
        },
    );
}

/// Remove a client from connected client map and trigger idle event again if already idle to remove the fleet.
/// TODO: This is janky.
fn disconnect_client(
    mut commands: Commands,
    mut clients_res: ResMut<ClientsRes>,
    client_disconnected: Res<EventRes<ClientDisconnected>>,
    time: Res<Time>,
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
            debug!("{:?} disconnected.", client_id);
        } else {
            warn!(
                "Got ClientDisconnected event, but {:?} can not be found. Ignoring...",
                client_id
            );
        }

        // Queue his fleet to be removed after a delay.
        commands
            .entity(client_disconnected.fleet_entity)
            .insert(QueueRemove {
                when: time.tick + DISCONNECT_REMOVE_FLEET_DELAY,
            });
    }
}

/// Remove fleet that are queued for deletion.
fn remove_fleet(
    mut commands: Commands,
    query: Query<(Entity, &FleetIdComp, &QueueRemove)>,
    mut fleets_res: ResMut<FleetsRes>,
    mut data_manager: ResMut<DataManager>,
    time: Res<Time>,
) {
    query.for_each(|(entity, fleet_id_comp, queue_remove)| {
        if queue_remove.when <= time.tick {
            if let Some(client_id) = fleet_id_comp.0.to_client_id() {
                // TODO: Save client's fleet.
                data_manager.client_fleets.insert(client_id, ());

                debug!("Removed and saved {:?}'s fleet.", client_id);
            }

            fleets_res.spawned_fleets.remove(&fleet_id_comp.0);
            commands.entity(entity).despawn();
        }
    });
}

/// Update the system each entity is currently in.
fn update_in_system(
    mut query: Query<(&FleetIdComp, &Position, &mut InSystem)>,
    systems: Res<Systems>,
    systems_acceleration_structure: Res<SystemsAccelerationStructure>,
    time: Res<Time>,
) {
    let turn = time.total_tick % 20;

    query.for_each_mut(|(fleet_id_comp, position, mut in_system)| {
        if fleet_id_comp.0 .0 % 20 == turn {
            match in_system.0 {
                Some(system_id) => {
                    if let Some(system) = systems.systems.get(&system_id) {
                        if system.position.distance_squared(position.0) > system.bound.powi(2) {
                            in_system.0 = None;
                        }
                    } else {
                        in_system.0 = None;
                    }
                }
                None => {
                    if let Some(system_id) = systems_acceleration_structure
                        .0
                        .intersect_point_first(position.0)
                    {
                        in_system.0 = Some(system_id);
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

    if *last_update_delta > DETECTED_UPDATE_INTERVAL as u32 {
        if let Some(mut old_snapshot) = intersection_pipeline.outdated.take() {
            // Update all colliders.
            old_snapshot.clear();
            old_snapshot.extend(query.iter().map(|(entity, position, detected_radius)| {
                (Collider::new(detected_radius.0, position.0), entity)
            }));

            // Send snapshot to be updated.
            if let Err(err) = intersection_pipeline
                .update_request_sender
                .send(old_snapshot)
            {
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
    mut query_client: Query<(
        Entity,
        &ClientIdComp,
        &Position,
        &EntityDetected,
        &mut KnowEntities,
    )>,
    query_changed_entity: Query<Entity, Changed<OrbitComp>>,
    query_fleet_info: Query<(&FleetIdComp, &Name, Option<&OrbitComp>)>,
    query_entity_state: Query<&Position, Without<OrbitComp>>,
    time: Res<Time>,
    clients_res: Res<ClientsRes>,
    task_pool: Res<ComputeTaskPool>,
) {
    query_client.par_for_each_mut(
        &task_pool,
        512,
        |(entity, client_id_comp, position, entity_detected, mut know_entities)| {
            if let Some(connection) = clients_res.connected_clients.get(&client_id_comp.0) {
                let know_entities = &mut *know_entities;

                let mut updated = Vec::with_capacity(entity_detected.0.len());
                let mut infos = Vec::new();

                entity_detected
                    .0
                    .iter()
                    .filter_map(|detected_entity| {
                        if let Some(temp_id) = know_entities.known.remove(detected_entity) {
                            // Client already know about this entity.
                            updated.push((*detected_entity, temp_id));
                            // Check if the entity infos changed. Otherwise do nothing.
                            if query_changed_entity.get(*detected_entity).is_ok() {
                                Some((temp_id, detected_entity))
                            } else {
                                None
                            }
                        } else {
                            // This is a new entity for the client.
                            let temp_id = know_entities.get_new_id();
                            updated.push((*detected_entity, temp_id));
                            Some((temp_id, detected_entity))
                        }
                    })
                    .for_each(|(temp_id, entity)| {
                        // TODO: This should be a function that write directly into a buffer.
                        if let Ok((fleet_id_comp, name, orbit_comp)) = query_fleet_info.get(*entity) {
                            infos.push((
                                temp_id,
                                EntityInfo {
                                    info_type: EntityInfoType::Fleet(FleetInfo {
                                        fleet_id: fleet_id_comp.0,
                                        composition: Vec::new(),
                                    }),
                                    name: name.0.clone(),
                                    orbit: orbit_comp.map(|orbit_comp| orbit_comp.0),
                                },
                            ));
                        } else {
                            debug!("Unknow entity type. Ignoring...");
                        }
                        // TODO: Try to query other type of entity.
                    });

                // Recycle temp idx.
                let to_remove: Vec<u16> = know_entities.known.drain().map(|(_, temp_id)| temp_id).collect();
                for temp_id in to_remove.iter() {
                    know_entities.recycle_id(temp_id.to_owned());
                }
                let packet = Packet::EntitiesRemove(EntitiesRemove {
                    tick: time.tick,
                    to_remove,
                })
                .serialize();
                connection.send_packet_reliable(packet);

                // Update known map.
                know_entities.known.extend(updated.into_iter());

                // Check if we should update the client's fleet.
                let client_info = if know_entities.force_update_client_info || query_changed_entity.get(entity).is_ok()
                {
                    know_entities.force_update_client_info = false;

                    if let Ok((fleet_id_comp, name, orbit_comp)) = query_fleet_info.get(entity) {
                        Some(EntityInfo {
                            info_type: EntityInfoType::Fleet(FleetInfo {
                                fleet_id: fleet_id_comp.0,
                                composition: Vec::new(),
                            }),
                            name: name.0.clone(),
                            orbit: orbit_comp.map(|orbit_comp| orbit_comp.0),
                        })
                    } else {
                        warn!(
                            "{:?} does not return a result when queried for fleet info. Ignoring...",
                            client_id_comp.0
                        );
                        None
                    }
                } else {
                    None
                };

                if let Some(info) = &client_info {
                    debug!("{:?}", info);
                }

                // Send entities info.
                let packet = Packet::EntitiesInfo(EntitiesInfo {
                    tick: time.tick,
                    client_info,
                    infos,
                })
                .serialize();
                connection.send_packet_reliable(packet);

                // Send entities state.
                // TODO: Limit the number of entity to not go over packet size limit.
                let packet = Packet::EntitiesState(EntitiesState {
                    tick: time.tick,
                    client_entity_position: position.0,
                    relative_entities_position: know_entities
                        .known
                        .iter()
                        .filter_map(|(entity, temp_id)| {
                            if let Ok(entity_position) = query_entity_state.get(*entity) {
                                Some((*temp_id, entity_position.0 - position.0))
                            } else {
                                None
                            }
                        })
                        .collect(),
                })
                .serialize();
                // TODO: Reuse this buffer to write entities infos.
                connection.send_packet_unreliable(&packet);

                // Flush tcp buffer.
                connection.flush_tcp_stream();

                // Maybe recycle temp idx.
                if time.tick % 10 == 0 {
                    know_entities.recycle_pending_idx();
                }
            }
        },
    );
}
