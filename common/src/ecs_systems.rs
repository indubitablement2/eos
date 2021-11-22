use crate::data_manager::DataManager;
use crate::ecs_components::*;
use crate::ecs_events::*;
use crate::fleet_ai::*;
use crate::packets::*;
use crate::res_clients::*;
use crate::res_fleets::FleetId;
use crate::res_fleets::FleetsRes;
use crate::res_parameters::ParametersRes;
use crate::res_times::TimeRes;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "first";
    schedule.add_stage(current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, increment_time.system());
    schedule.add_system_to_stage(current_stage, get_new_clients.system());
    schedule.add_system_to_stage(current_stage, change_fleet_control.system());

    let previous_stage = current_stage;
    let current_stage = "pre_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, add_remove_fleet_ai.system());
    schedule.add_system_to_stage(current_stage, fleet_ai.system());
    schedule.add_system_to_stage(current_stage, process_client_udp.system());

    let previous_stage = current_stage;
    let current_stage = "update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, movement.system());

    let previous_stage = current_stage;
    let current_stage = "post_update";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, apply_velocity.system());

    let previous_stage = current_stage;
    let current_stage = "last";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, clear_events.system());
}

//* first

fn increment_time(mut time_res: ResMut<TimeRes>) {
    time_res.tick += 1;
}

/// Get new connection and insert client.
fn get_new_clients(
    mut clients_res: ResMut<ClientsRes>,
    data_manager: Res<DataManager>,
    mut client_connected: ResMut<EventRes<ClientConnected>>,
) {
    while let Ok(connection) = clients_res.connection_manager.new_connection_receiver.try_recv() {
        // Load client data.
        let client_data = data_manager.load_client(connection.client_id);

        let client_id = connection.client_id;

        // Create client.
        let client = Client {
            connection,
            fleet_control: None,
            client_data,
            // input_battlescape: BattlescapeInput::default(),
            // unacknowledged_commands: IndexMap::new(),
        };

        // Insert client.
        if clients_res.connected_clients.insert(client_id, client).is_some() {
            info!("{:?} was disconnected as a new connection took this client.", client_id);
        }

        // Trigger event.
        client_connected.trigger_event(ClientConnected { client_id });
    }
}

/// Add or remove Controlled from entities.
fn change_fleet_control(
    mut command: Commands,
    query: Query<(Entity, &FleetId, &Controlled)>,
    clients_res: Res<ClientsRes>,
    fleet_res: Res<FleetsRes>,
    mut just_controlled: ResMut<EventRes<JustControlled>>,
    mut just_stop_controlled: ResMut<EventRes<JustStopControlled>>,
) {
    // Remove Controlled from entity that are no longer being directly controlled.
    query.for_each(|(entity, fleet_id, controlled)| {
        // Check that the client controlling this entity is connected. 
        if let Some(client) = clients_res.connected_clients.get(&controlled.0) {
            // Check that the client is currently controlling this fleet.
            if client.fleet_control != Some(*fleet_id) {
                // The client is not controlling this entity.
                command.entity(entity).remove::<Controlled>();
                just_stop_controlled.trigger_event(JustStopControlled { entity, client_id: controlled.0 });
            }
        } else {
            // The client is not connected.
            command.entity(entity).remove::<Controlled>();
            just_stop_controlled.trigger_event(JustStopControlled { entity, client_id: controlled.0 });
        }
    });

    // Add Controlled to entity that are now directly controlled.
    for (client_id, client) in &clients_res.connected_clients {
        if let Some(fleet_id) = &client.fleet_control {
            if let Some(entity) = fleet_res.spawned_fleets.get(fleet_id) {
                if let Err(err) =  query.get(*entity) {
                    match err {
                        bevy_ecs::query::QueryEntityError::QueryDoesNotMatch => {
                            // Add Controlled to the entity.
                            command.entity(*entity).insert(Controlled(*client_id));
                            just_controlled.trigger_event(JustControlled {
                                entity: *entity,
                                client_id: *client_id,
                            });
                        }
                        bevy_ecs::query::QueryEntityError::NoSuchEntity => {
                            debug!("{:?} tried to control {:?}, but it is not loaded. Maybe it is destroyed?", client_id, fleet_id);
                        }
                    }
                }
            }
        }
    }
}

//* pre_update

/// Make sure entity with a FleetId have either FleetAI or Controlled.
fn add_remove_fleet_ai(mut command: Commands, query: Query<(Entity, &FleetId, Option<&Controlled>, Option<&FleetAI>)>) {
    query.for_each(|(entity, fleet_id, controlled, fleet_ai)| {
        if controlled.is_some() && fleet_ai.is_some() {
            // Controlled take precedence.
            command.entity(entity).remove::<FleetAI>();
        } else if controlled.is_none() && fleet_ai.is_none() {
            // Add FleetAI.
            command.entity(entity).insert(FleetAI::default());
        }
    });
}

fn fleet_ai(mut query: Query<(&mut FleetAI, &Position, &mut WishPosition)>, task_pool: Res<TaskPool>) {
    query.par_for_each_mut(&task_pool, 16, |(mut fleet_ai, pos, mut wish_pos)| {
        match fleet_ai.goal {
            FleetGoal::Trade { from, to } => todo!(),
            FleetGoal::Guard { who, radius, duration } => todo!(),
            FleetGoal::Wandering { to, pause } => todo!(),
            FleetGoal::Idle { duration } => todo!(),
        }
    });
}

/// Get and process clients udp packets.
fn process_client_udp(clients_res: Res<ClientsRes>, query: Query<(&Controlled, &mut WishPosition)>, mut client_disconnected: ResMut<EventRes<ClientDisconnected>>,) {
    query.for_each_mut(|(controlled, mut fleet_wish_pos)| {
        if let Some((client_id, client)) = clients_res.connected_clients.get_key_value(&controlled.0) {
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
                                fleet_wish_pos.0 = wish_position;
                            }
                        }
                    }
                    Err(err) => {
                        if err == crossbeam_channel::TryRecvError::Disconnected {
                            // Cliend disconnected.
                            client_disconnected.trigger_event(ClientDisconnected { client_id: *client_id });
                        }
                        break;
                    }
                }
            }
        }
    });
}

//* update

/// Add velocity based on wish position.
/// TODO: Fleets engaged in the same Battlescape should aggregate.
/// TODO: Use the speed of the fleet.
fn movement(query: Query<(&Position, &WishPosition, &mut Velocity)>) {
    query.for_each_mut(|(pos, wish_pos, mut vel)| {
        // TODO: Stop threshold.
        if pos.0.distance_squared(wish_pos.0) < 10.0 {
            // Try to stop.
            let new_vel = -vel.0.clamp_length_max(1.0);
            vel.0 += new_vel;
        } else {
            // Add velocity toward fleet's wish position at full speed.
            vel.0 += (wish_pos.0 - pos.0).clamp_length_max(1.0);
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

//* last

// /// TODO: Send unacknowledged commands.
// /// TODO: Just sending every fleets position for now.
// fn send_udp(&mut self) {
//     let fleets_position: Vec<Vec2> = self.fleets.values().map(|fleet| fleet.detector_collider.position).collect();

//     let packet = UdpServer::Metascape { fleets_position };

//     for (client_id, client) in &self.clients {
//         if client.connection.udp_sender.blocking_send(packet.clone()).is_err() {
//             self.disconnect_queue.push(*client_id);
//         }
//     }
// }
