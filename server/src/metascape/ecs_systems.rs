use super::battlescape_manager::BattlescapeManager;
use super::clients_manager::*;
use super::colony::Colonies;
use super::data_manager::ClientData;
use super::data_manager::DataManager;
use super::ecs_components::*;
use super::ecs_events::*;
use super::fleets_manager::*;
use super::interception_manager::InterceptionManager;
use super::DetectedIntersectionPipeline;
use super::SystemsAccelerationStructure;
use crate::server_configs::*;
use bevy_ecs::prelude::*;
use bevy_tasks::ComputeTaskPool;
use common::factions::*;
use common::idx::*;
use common::intersection::*;
use common::metascape_configs::MetascapeConfigs;
use common::net::packets::*;
use common::orbit::*;
use common::ships::Bases;
use common::systems::Systems;
use common::time::Time;
use common::WORLD_BOUND;
use glam::Vec2;
use rand::seq::index::sample;
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro128StarStar;
use std::f32::consts::TAU;

const DETECTED_UPDATE_INTERVAL: u64 = 5;
/// Minimum delay before a disconnected client's fleet get removed.
const DISCONNECT_REMOVE_FLEET_DELAY: u32 = 200;
const UPDATE_IN_SYSTEM_INTERVAL: u64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum Label {
    Movement,
    DetectedPipelineUpdate,
}

pub fn add_systems(schedule: &mut Schedule) {
    schedule.add_stage("", SystemStage::parallel());

    schedule.add_system_to_stage("", get_new_clients);

    schedule.add_system_to_stage("", spawn_colonist);

    schedule.add_system_to_stage("", handle_orbit);
    schedule.add_system_to_stage("", remove_orbit);

    schedule.add_system_to_stage("", update_in_system);

    schedule.add_system_to_stage("", handle_battlescape);

    schedule.add_system_to_stage("", handle_interceptions.before(Label::Movement));
    schedule.add_system_to_stage("", handle_client_inputs.before(Label::Movement));
    schedule.add_system_to_stage("", colony_guard_fleet_ai.before(Label::Movement));
    schedule.add_system_to_stage("", colonist_fleet_ai.before(Label::Movement));

    schedule.add_system_to_stage("", apply_fleet_movement.label(Label::Movement));

    schedule.add_system_to_stage("", fleet_sensor.before(Label::DetectedPipelineUpdate));
    schedule.add_system_to_stage(
        "",
        update_detected_intersection_pipeline
            .label(Label::DetectedPipelineUpdate)
            .after(Label::Movement),
    );

    schedule.add_system_to_stage("", send_detected_entity.after(Label::Movement));

    schedule.add_system_to_stage("", event_handler_client_disconnected);
    schedule.add_system_to_stage("", remove_fleet);

    schedule.add_system_to_stage("", event_handler_fleet_destroyed);
}

/// Get new connection and insert client.
fn get_new_clients(
    mut commands: Commands,
    mut query_client_fleet: Query<(&WrappedId<ClientId>, &mut KnowEntities)>,
    mut clients_manager: ResMut<ClientsManager>,
    mut fleets_manager: ResMut<FleetsManager>,
    mut data_manager: ResMut<DataManager>,
    connection_handler_configs: Res<ConnectionHandlerConfigs>,
) {
    clients_manager.handle_pending_connections();

    let mut num_new_connection = 0;

    // Connect a few clients.
    loop {
        let new_connection = match clients_manager.try_connect_one() {
            Ok(new_connection) => new_connection,
            Err(err) => match err {
                ConnectError::Empty => {
                    break;
                }
                ConnectError::AlreadyConnected => {
                    continue;
                }
            },
        };

        let client_id = new_connection.client_id();
        let fleet_id = client_id.to_fleet_id();

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
        if let Some(old_fleet_entity) = fleets_manager.get_spawned_fleet(fleet_id) {
            if let Ok((wrapped_client_id, mut know_entities)) =
                query_client_fleet.get_mut(old_fleet_entity)
            {
                // Update old fleet components.
                let know_entities = &mut *know_entities;
                *know_entities = KnowEntities::default();
                commands.entity(old_fleet_entity).remove::<QueueRemove>();

                if wrapped_client_id.id() != client_id {
                    error!(
                        "{:?} was asigned {:?}'s fleet. Fleets manager and world do not match.",
                        client_id,
                        wrapped_client_id.id()
                    );
                    debug_assert!(false, "Fleets manager and world out of sync.");
                }

                debug!("{:?} has taken back control of his fleet.", client_id);
            } else {
                error!(
                    "{:?}'s fleet is in fleets manager, but is not found in world. Removing from spawned fleets...",
                    client_id
                );
            }
        } else {
            fleets_manager.spawn_default_client_fleet(&mut commands, client_id);
        }

        num_new_connection += 1;
        if num_new_connection >= connection_handler_configs.max_new_connection_per_update {
            break;
        }
    }
}

/// Spawn colonist fleet to reach faction's target colony.
fn spawn_colonist(
    mut commands: Commands,
    factions: Res<Factions>,
    colonies: Res<Colonies>,
    mut fleets_manager: ResMut<FleetsManager>,
    time: Res<Time>,
) {
    // TODO: Use run criteria.
    if time.tick % 10 != 0 {
        return;
    }

    for (faction, faction_id) in factions.factions.iter().zip(0u8..) {
        if faction.disabled {
            continue;
        }

        let faction_id = FactionId(faction_id);

        let faction_colonies = colonies.get_faction_colonies(faction_id);

        if faction_colonies.len() < faction.target_colonies {
            fleets_manager.spawn_colonist_ai_fleet(
                &mut commands,
                None,
                time.tick + ColonistFleetAI::DEFAULT_TRAVEL_DURATION,
                Vec2::ZERO,
                Some(faction_id),
            )
        }
    }
}

/// Determine what each fleet can see.
fn fleet_sensor(
    mut query: Query<(
        Entity,
        &WrappedId<FleetId>,
        &Position,
        &DetectorRadius,
        &mut EntityDetected,
        &Reputations,
    )>,
    query_reputation: Query<(&WrappedId<FleetId>, &Reputations)>,
    detected_intersection_pipeline: Res<DetectedIntersectionPipeline>,
    time: Res<Time>,
    factions: Res<Factions>,
) {
    // We will only update one part every tick.
    let turn = time.tick as u64 % DETECTED_UPDATE_INTERVAL;

    query.for_each_mut(
        |(
            entity,
            wrapped_fleet_id,
            position,
            detector_radius,
            mut entity_detected,
            reputations,
        )| {
            if wrapped_fleet_id.id().0 % DETECTED_UPDATE_INTERVAL == turn {
                let detector_collider = Collider::new(detector_radius.0, position.0);

                // Filter the result.
                if wrapped_fleet_id.id().is_client() {
                    detected_intersection_pipeline
                        .0
                        .snapshot
                        .intersect_collider_into(detector_collider, &mut entity_detected.0);

                    // Client fleet filter out themself.
                    for i in 0..entity_detected.0.len() {
                        if entity_detected.0[i] == entity {
                            entity_detected.0.swap_remove(i);
                            break;
                        }
                    }
                } else {
                    // AI fleet filter out non enemy faction fleet.
                    let filter = if let Some(faction_id) = reputations.faction {
                        faction_id.to_bit_flag()
                    } else {
                        u32::MAX
                    };
                    detected_intersection_pipeline
                        .0
                        .snapshot
                        .intersect_collider_into_filtered(
                            detector_collider,
                            &mut entity_detected.0,
                            filter,
                        );

                    // AI fleet filter out the remaining non enemy factionless fleet.
                    entity_detected.0.drain_filter(|id| {
                        if let Ok((other_wrapped_fleet_id, other_rep)) = query_reputation.get(*id) {
                            // TODO: Unroll and remove uneeded logic.
                            !reputations
                                .get_relative_reputation(
                                    other_rep,
                                    wrapped_fleet_id.id(),
                                    other_wrapped_fleet_id.id(),
                                    &factions.factions,
                                )
                                .is_enemy()
                        } else {
                            // This can happen if the entity was removed,
                            // but intersection pipeline was not updated yet.
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
    mut query: Query<(
        Entity,
        &WrappedId<ClientId>,
        &mut WishPosition,
        Option<&Intercepted>,
    )>,
    clients_manager: Res<ClientsManager>,
    mut event_client_disconnected: ResMut<EventRes<ClientDisconnected>>,
) {
    query.for_each_mut(
        |(entity, wrapped_client_id, mut wish_position, intercepted)| {
            if let Some(connection) = clients_manager.get_connection(wrapped_client_id.id()) {
                loop {
                    match connection.try_recv() {
                        Ok(payload) => match Packet::deserialize(&payload) {
                            Packet::Message { origin, content } => {
                                // TODO: Broadcast the message.
                            }
                            Packet::MetascapeWishPos { wish_pos, movement_multiplier } => {
                                if intercepted.is_none() {
                                    wish_position.set_wish_position(wish_pos, movement_multiplier.clamp(0.1, 1.0));
                                }
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
                                    wrapped_client_id.id()
                                );
                                event_client_disconnected.push(ClientDisconnected {
                                    client_id: wrapped_client_id.id(),
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
                                event_client_disconnected.push(ClientDisconnected {
                                    client_id: wrapped_client_id.id(),
                                    fleet_entity: entity,
                                    send_packet: None,
                                });
                            }
                            break;
                        }
                    }
                }
            }
        },
    );
}

/// Change the position of entities that have an orbit.
fn handle_orbit(mut query: Query<(&OrbitComp, &mut Position)>, time: Res<Time>) {
    let timef = time.as_timef();

    query.for_each_mut(|(orbit_comp, mut position)| {
        position.0 = orbit_comp.0.to_position(timef);
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

fn handle_interceptions(
    mut interception_manager: ResMut<InterceptionManager>,
    query: Query<(&Size, Option<&mut WishPosition>)>,
) {
    interception_manager.update(query);
}

fn handle_battlescape(
    mut commands: Commands,
    mut query_fleet: Query<(&mut FleetComposition, &mut FleetState)>,
    mut battlescape_manager: ResMut<BattlescapeManager>,
    mut event_fleet_destroyed: ResMut<EventRes<FleetDestroyed>>,
    bases: Res<Bases>,
    time: Res<Time>,
) {
    let bases = &*bases;
    let mut rng = Xoshiro128StarStar::seed_from_u64(time.total_tick.wrapping_mul(5342679));

    let mut queue_terminated = Vec::new();

    for (battlescape_id, battlescape) in battlescape_manager.active_battlescape.iter_mut() {
        battlescape.time += 1;

        if battlescape.teams.len() < 2 {
            if battlescape.time > 5 {
                // Queue the battlescape to be terminated.
                queue_terminated.push(*battlescape_id);
            }
            continue;
        }

        // Simulate the battlescape.
        if battlescape.time % 10 == 0 {
            while battlescape.teams.len() > 1 {
                // Chose an attacker and a defender team.
                let idx = sample(&mut rng, battlescape.teams.len(), 2);
                let attacker_id = idx.index(0);
                let defender_id = idx.index(1);
                let attacker_team = battlescape.teams[attacker_id].clone();
                let defender_team = battlescape.teams[defender_id].clone();

                let mut queue_remove_fleet = Vec::new();

                // Look for how much we will attack for.
                let attack = attacker_team
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, entity)| {
                        if let Ok((fleet_composition, fleet_state)) = query_fleet.get(entity) {
                            Some(fleet_composition.compute_auto_combat_strenght(fleet_state, bases))
                        } else {
                            // Queue fleet to be removed from the battlescape.
                            queue_remove_fleet.push(i);
                            None
                        }
                    })
                    .reduce(|acc, x| acc + x);
                let mut attack = if let Some(attack) = attack {
                    attack
                } else {
                    // Attacker team is invalid. We have to try again.
                    battlescape.teams.swap_remove(attacker_id);
                    continue;
                };

                // Remove invalid fleets from the attacker team.
                debug_assert!(queue_remove_fleet.is_sorted());
                let team = &mut battlescape.teams[attacker_id];
                for i in queue_remove_fleet.drain(..).rev() {
                    team.swap_remove(i);
                }

                // Apply attack to the defender team.
                attack /= defender_team.len() as f32;
                defender_team
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, entity)| {
                        if let Ok((mut fleet_composition, mut fleet_state)) =
                            query_fleet.get_mut(entity)
                        {
                            if fleet_composition.attack_fleet(
                                &mut fleet_state,
                                attack,
                                &mut rng,
                                time.tick,
                            ) {
                                // Fleet is destroyed.
                                queue_remove_fleet.push(i);
                                event_fleet_destroyed.push(FleetDestroyed { entity });
                            }
                        } else {
                            // Can not find fleet. Queue it to be removed from the battlescape.
                            queue_remove_fleet.push(i);
                        }
                    });

                // Remove defender destroyed fleets.
                debug_assert!(queue_remove_fleet.is_sorted());
                let team = &mut battlescape.teams[defender_id];
                for i in queue_remove_fleet.drain(..).rev() {
                    team.swap_remove(i);
                }

                break;
            }
        }
    }

    // Terminate battlescape.
    for battlescape_id in queue_terminated.into_iter() {
        if let Some(battlescape) = battlescape_manager
            .active_battlescape
            .remove(&battlescape_id)
        {
            for team in battlescape.teams.into_iter() {
                for entity in team.into_iter() {
                    commands.entity(entity).remove::<Intercepted>();
                }
            }
        }
    }
}

/// Ai that control fleet guarding a colony.
fn colony_guard_fleet_ai(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut ColonyGuardFleetAI,
            &Position,
            &mut WishPosition,
            &EntityDetected,
        ),
        Without<Intercepted>,
    >,
    query_other: Query<&Position, Without<Intercepted>>,
    mut interception_manager: ResMut<InterceptionManager>,
    mut battlescape_manager: ResMut<BattlescapeManager>,
    systems: Res<Systems>,
    time: Res<Time>,
) {
    let timef = time.as_timef();

    query.for_each_mut(
        |(entity, mut colony_guard_fleet_ai, position, mut wish_position, entity_detected)| {
            if let Some(target_entity) = colony_guard_fleet_ai.target {
                if let Ok(target_position) = query_other.get(target_entity) {
                    // TODO: Check that atarget is still detected.

                    // Go toward target position.
                    wish_position.set_wish_position(target_position.0, 1.0);

                    // Intercept and start a battlescape if target is close enough.
                    if target_position.0.distance_squared(position.0) < 1.0 {
                        // Create the battlescape.
                        let battlescape_id = battlescape_manager
                            .start_new_battlescape(vec![vec![entity], vec![target_entity]]);

                        // Create the intersection.
                        let entities = vec![entity, target_entity];
                        let center = (position.0 + target_position.0) * 0.5;
                        let interception_id =
                            interception_manager.create_interception(entities, center);

                        commands.entity(entity).insert(Intercepted {
                            interception_id,
                            reason: InterceptedReason::Battle(battlescape_id),
                        });
                        commands.entity(target_entity).insert(Intercepted {
                            interception_id,
                            reason: InterceptedReason::Battle(battlescape_id),
                        });
                    }
                } else {
                    // Can not find target.
                    colony_guard_fleet_ai.target = None;
                }
            } else if !entity_detected.0.is_empty() {
                // Chase the closest target.
                for other_entity in entity_detected.0.iter() {
                    if let Ok(other_position) = query_other.get(*other_entity) {
                        if let Some(previous_target) = wish_position.target() {
                            if previous_target.distance_squared(position.0)
                                > other_position.0.distance_squared(position.0)
                            {
                                colony_guard_fleet_ai.target = Some(*other_entity);
                                wish_position.set_wish_position(other_position.0, 1.0);
                            }
                        } else {
                            colony_guard_fleet_ai.target = Some(*other_entity);
                            wish_position.set_wish_position(other_position.0, 1.0);
                        }
                    }
                }
            } else if wish_position.target().is_none() {
                // TODO: Wander around colony.
                let (system, planet) = systems.get_system_and_planet(colony_guard_fleet_ai.colony);
                let planet_position = planet.relative_orbit.to_position(timef, system.position);

                wish_position.set_wish_position(planet_position, 0.8);
            }
        },
    );
}

fn colonist_fleet_ai(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Position,
            &InSystem,
            &mut ColonistFleetAI,
            &mut WishPosition,
            &Reputations,
        ),
        Without<Intercepted>,
    >,
    systems: Res<Systems>,
    mut colonies: ResMut<Colonies>,
    time: Res<Time>,
) {
    let timef = time.as_timef();
    let mut rng = Xoshiro128StarStar::seed_from_u64(time.total_tick.wrapping_mul(43627));

    query.for_each_mut(
        |(entity, position, in_system, mut colonist_fleet_ai, mut wish_position, reputations)| {
            if let Some(planet_id) = colonist_fleet_ai.target_planet() {
                // Go torward target planet.
                let (system, planet) = systems.get_system_and_planet(planet_id);
                let planet_position = planet.relative_orbit.to_position(timef, system.position);
                wish_position.set_wish_position(
                    planet_position,
                    ColonistFleetAI::MOVEMENT_MULTIPLIER_COLONIZING,
                );

                if colonies.get_colony_faction(planet_id).is_some() {
                    // Planet has already been colonized.
                    colonist_fleet_ai.reset_target_planet();
                } else {
                    // Are we close enough to the planet to take it?
                    if planet_position.distance_squared(position.0) < 1.0 {
                        // Faction take control of the planet.
                        // TODO: This should take time.
                        colonies.give_colony_to_faction(planet_id, reputations.faction);

                        // Change AI to guard the planet.
                        commands.entity(entity).remove::<ColonistFleetAI>().insert(
                            ColonyGuardFleetAI {
                                target: None,
                                colony: planet_id,
                            },
                        );

                        wish_position.stop();

                        debug!("Took planet.");
                    }
                }
            } else if wish_position.target().is_none() {
                // Go to a random system.
                // We aim for the system's bound instead of center where there can be a star or worse.

                // Get a random system.
                let system = &systems.systems[rng.gen::<usize>() % systems.systems.len()];

                // Randomly compute the system bound direction.
                let rot = rng.gen::<f32>() * TAU;
                let random_system_bound =
                    system.position + Vec2::new(rot.cos(), rot.sin()) * system.bound * 0.7;

                wish_position.set_wish_position(
                    random_system_bound,
                    ColonistFleetAI::MOVEMENT_MULTIPLIER_TRAVELLING,
                );
            } else if colonist_fleet_ai.is_done_travelling(time.tick) {
                // We are done travelling. Start searching for a planet.
                if let Some(system_id) = in_system.0 {
                    let system = &systems.systems[system_id];

                    // Randomly chose a planet to colonize in this system.
                    let planets_offset = (rng.gen::<u32>() % system.planets.len() as u32) as u8;
                    let planet_id = PlanetId {
                        system_id,
                        planets_offset,
                    };

                    if colonies.get_colony_faction(planet_id).is_none() {
                        colonist_fleet_ai.set_target_planet(planet_id);
                    } else {
                        // Planet is already a colony.
                        // Start travelling again.
                        colonist_fleet_ai
                            .set_travel_until(ColonistFleetAI::DEFAULT_TRAVEL_DURATION, time.tick);
                        wish_position.stop();
                    }
                }
            }
        },
    );
}

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity and friction.
///
/// Intercepted entity aggregate.
fn apply_fleet_movement(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Position,
        &mut WishPosition,
        &mut Velocity,
        &DerivedFleetStats,
        &InSystem,
        &mut IdleCounter,
    )>,
    metascape_configs: Res<MetascapeConfigs>,
    systems: Res<Systems>,
    time: Res<Time>,
) {
    let bound_squared = WORLD_BOUND.powi(2);
    let timef = time.as_timef();

    query.for_each_mut(
        |(
            entity,
            mut position,
            mut wish_position,
            mut velocity,
            derived_fleet_stats,
            in_system,
            mut idle_counter,
        )| {
            if let Some(target) = wish_position.target() {
                // A vector equal to our current velocity toward our target.
                let wish_vel = target - position.0 - velocity.0;

                // Seek target.
                velocity.0 += wish_vel.clamp_length_max(
                    derived_fleet_stats.acceleration * wish_position.movement_multiplier(),
                );

                // Stop if we are near the target.
                // TODO: Stop threshold should depend on fleet acceleration and current speed.
                if wish_vel.length_squared() < 1.0 {
                    wish_position.stop();
                }

                idle_counter.set_non_idle();
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

                idle_counter.set_non_idle();
            } else {
                idle_counter.increment();
                if idle_counter.just_stated_idling() {
                    // Add orbit as this entity has no velocity.
                    let orbit = if let Some(system_id) = in_system.0 {
                        let system = &systems.systems[system_id];
        
                        let relative_position = position.0 - system.position;
                        let distance = relative_position.length();
        
                        let mut orbit_speed = 0.0;
                        // Check if there is a body nearby we should copy its orbit speed.
                        system.planets.iter().fold(999.0f32, |closest, planet| {
                            let dif = (planet.relative_orbit.distance - distance).abs();
                            if dif < closest {
                                orbit_speed = planet.relative_orbit.orbit_speed;
                                dif
                            } else {
                                closest
                            }
                        });
        
                        Orbit::from_relative_position(
                                relative_position,
                                timef,
                                system.position,
                                distance,
                                orbit_speed,
                            )
                    } else {
                        // Add a stationary orbit.
                        Orbit::stationary(position.0)
                    };

                    commands.entity(entity).insert(OrbitComp(orbit));
                }
            }

            // Entities are pushed away from the world's bound.
            if position.0.length_squared() > bound_squared {
                velocity.0 -= position.0.normalize() * 10.0;
            }

            // Apply friction.
            velocity.0 *= metascape_configs.friction;

            // Apply velocity.
            position.0 += velocity.0;
        },
    );
}

/// Remove a client from connected client map and queue his fleet to be removed.
fn event_handler_client_disconnected(
    mut commands: Commands,
    mut clients_manager: ResMut<ClientsManager>,
    mut event_client_disconnected: ResMut<EventRes<ClientDisconnected>>,
    time: Res<Time>,
) {
    while let Some(client_disconnected) = event_client_disconnected.pop() {
        let client_id = client_disconnected.client_id;

        // Remove connection.
        if let Some(connection) = clients_manager.remove_connection(client_id) {
            if let Some(packet) = client_disconnected.send_packet {
                // Send last packet.
                connection.send_packet_reliable(packet.serialize());
                connection.flush_tcp_stream();
            }
            debug!("{:?} disconnected.", client_id);
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
    query: Query<(&WrappedId<FleetId>, &QueueRemove)>,
    mut fleets_manager: ResMut<FleetsManager>,
    time: Res<Time>,
) {
    query.for_each(|(wrapped_fleet_id, queue_remove)| {
        if queue_remove.when <= time.tick {
            fleets_manager.remove_spawned_fleet(&mut commands, wrapped_fleet_id.id());
        }
    });
}

/// Update the system each entity is currently in.
fn update_in_system(
    mut query: Query<(&WrappedId<FleetId>, &Position, &mut InSystem)>,
    systems: Res<Systems>,
    systems_acceleration_structure: Res<SystemsAccelerationStructure>,
    time: Res<Time>,
) {
    let turn = time.total_tick % UPDATE_IN_SYSTEM_INTERVAL;

    query.for_each_mut(|(wrapped_fleet_id, position, mut in_system)| {
        if wrapped_fleet_id.id().0 % UPDATE_IN_SYSTEM_INTERVAL == turn {
            match in_system.0 {
                Some(system_id) => {
                    let system = &systems.systems[system_id];
                    if system.position.distance_squared(position.0) > system.bound.powi(2) {
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
    query: Query<(Entity, &Position, &DetectedRadius, &Reputations)>,
    mut factions: ResMut<Factions>,
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
            // Update enemy masks.
            factions.update_factions_enemy_mask();

            // Update all colliders.
            old_snapshot.clear();
            old_snapshot.extend(query.iter().map(
                |(entity, position, detected_radius, reputations)| {
                    if let Some(faction_id) = reputations.faction {
                        (
                            Collider::new(detected_radius.0, position.0),
                            entity,
                            factions.enemy_masks[faction_id.0 as usize],
                        )
                    } else {
                        (
                            Collider::new(detected_radius.0, position.0),
                            entity,
                            u32::MAX,
                        )
                    }
                },
            ));

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
        &WrappedId<ClientId>,
        &Position,
        &EntityDetected,
        &mut KnowEntities,
    )>,
    query_changed: Query<
        (Entity, &FleetComposition, Option<&OrbitComp>),
        Or<(Changed<OrbitComp>, Changed<FleetComposition>)>,
    >,
    query_fleet_info: Query<(
        &WrappedId<FleetId>,
        &FleetComposition,
        &Name,
        Option<&OrbitComp>,
    )>,
    query_state: Query<&Position, Without<OrbitComp>>,
    clients_manager: Res<ClientsManager>,
    time: Res<Time>,
    task_pool: Res<ComputeTaskPool>,
) {
    query_client.par_for_each_mut(
        &task_pool,
        1024,
        |(entity, wrapped_client_id, position, entity_detected, mut know_entities)| {
            if let Some(connection) = clients_manager.get_connection(wrapped_client_id.id()) {
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
                            if query_changed.get(*detected_entity).is_ok() {
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
                        if let Ok((wrapped_fleet_id, fleet_composition, name, orbit_comp)) = query_fleet_info.get(*entity) {
                            infos.push((
                                temp_id,
                                EntityInfo {
                                    info_type: EntityInfoType::Fleet(FleetInfo {
                                        fleet_id: wrapped_fleet_id.id(),
                                        composition: fleet_composition.ships().iter().map(|ship| ship.ship).collect(),
                                    }),
                                    name: name.0.clone(),
                                    orbit: orbit_comp.map(|orbit_comp| orbit_comp.0),
                                },
                            ));
                        } else {
                            // TODO: Try to query other type of entity.
                            debug!("Unknow entity type. Ignoring...");
                        }
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
                let client_info = if know_entities.force_update_client_info || query_changed.get(entity).is_ok()
                {
                    know_entities.force_update_client_info = false;

                    if let Ok((wrapped_fleet_id, fleet_composition, name, orbit_comp)) = query_fleet_info.get(entity) {
                        Some(EntityInfo {
                            info_type: EntityInfoType::Fleet(FleetInfo {
                                fleet_id: wrapped_fleet_id.id(),
                                composition: fleet_composition.ships().iter().map(|ship| ship.ship).collect(),
                            }),
                            name: name.0.clone(),
                            orbit: orbit_comp.map(|orbit_comp| orbit_comp.0),
                        })
                    } else {
                        warn!(
                            "{:?} does not return a result when queried for fleet info. Ignoring...",
                            wrapped_client_id.id()
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
                            if let Ok(entity_position) = query_state.get(*entity) {
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

fn event_handler_fleet_destroyed(
    mut commands: Commands,
    query: Query<&WrappedId<FleetId>>,
    mut fleets_manager: ResMut<FleetsManager>,
    mut event_fleet_destroyed: ResMut<EventRes<FleetDestroyed>>,
    mut event_client_disconnected: ResMut<EventRes<ClientDisconnected>>,
) {
    while let Some(event) = event_fleet_destroyed.pop() {
        if let Ok(wrapped_fleet_id) = query.get(event.entity) {
            if let Some(client_id) = wrapped_fleet_id.id().to_client_id() {
                // TODO: Client's fleet should not be completely destroyable.
                warn!("{:?}'s has been completely destroyed. This should not happen. Disconnecting...", client_id);
                event_client_disconnected.push(ClientDisconnected {
                    client_id,
                    fleet_entity: event.entity,
                    send_packet: Some(Packet::DisconnectedReason(
                        DisconnectedReasonEnum::ServerError,
                    )),
                });
            } else {
                fleets_manager.remove_spawned_fleet(&mut commands, wrapped_fleet_id.id());
            }
        }
    }
}
