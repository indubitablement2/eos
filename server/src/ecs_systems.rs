use crate::data_manager::DataManager;
use crate::ecs_components::*;
use crate::ecs_events::*;
use crate::fleet_ai::*;
use crate::res_clients::*;
use crate::res_fleets::FleetsRes;
use crate::res_fleets::*;
use crate::res_parameters::ParametersRes;
use crate::res_times::TimeRes;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use common::packets::*;
use glam::Vec2;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "first";
    schedule.add_stage(current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, increment_time.system());
    schedule.add_system_to_stage(current_stage, get_new_clients.system());

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

    let previous_stage = current_stage;
    let current_stage = "last";
    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());
    schedule.add_system_to_stage(current_stage, send_udp.system());
}

//* first

fn increment_time(mut time_res: ResMut<TimeRes>) {
    time_res.tick += 1;
}

/// Get new connection and insert client.
fn get_new_clients(
    mut command: Commands,
    mut clients_res: ResMut<ClientsRes>,
    mut fleets_res: ResMut<FleetsRes>,
    data_manager: Res<DataManager>,
    client_connected: ResMut<EventRes<ClientConnected>>,
) {
    while let Ok(connection) = clients_res.connection_manager.new_connection_receiver.try_recv() {
        // Load client data.
        let client_data = data_manager.load_client(connection.client_id);

        let client_id = connection.client_id;
        let fleet_id = FleetId::from(client_id);

        // Create client.
        let client = Client {
            connection,
            client_data,
            // input_battlescape: BattlescapeInput::default(),
            // unacknowledged_commands: IndexMap::new(),
        };

        // Insert client.
        if clients_res.connected_clients.insert(client_id, client).is_some() {
            info!("{:?} was disconnected as a new connection took this client.", client_id);
        }

        // Check if fleet is already spawned.
        if let Some(old_fleet_entity) = fleets_res.spawned_fleets.get(&fleet_id) {
            // TODO: Check components.
        } else {
            // TODO: Create client's fleet.
            let fleet_entity = command
                .spawn_bundle((
                    fleet_id,
                    client_id,
                    Position(Vec2::ZERO),
                    WishPosition(Vec2::ZERO),
                    Velocity(Vec2::ZERO),
                    FleetAI {
                        goal: FleetGoal::Controlled,
                    },
                ))
                .id();

            // Insert fleet.
            let _ = fleets_res.spawned_fleets.insert(fleet_id, fleet_entity);
        }

        // Trigger event.
        client_connected.events.push(ClientConnected { client_id });
    }
}

//* pre_update

fn fleet_ai(
    mut query: Query<(&FleetId, &mut FleetAI, &Position, &mut WishPosition)>,
    task_pool: Res<TaskPool>,
    clients_res: Res<ClientsRes>,
    client_disconnected: ResMut<EventRes<ClientDisconnected>>,
) {
    query.par_for_each_mut(&task_pool, 16, |(fleet_id, mut fleet_ai, pos, mut wish_pos)| {
        match fleet_ai.goal {
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
            FleetGoal::Wandering { to, pause } => todo!(),
            FleetGoal::Idle { duration } => todo!(),
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

/// TODO: Prepare client's fleets to be removed.
fn disconnect_client(mut clients_res: ResMut<ClientsRes>, client_disconnected: Res<EventRes<ClientDisconnected>>) {
    while let Some(client_disconnected) = client_disconnected.events.pop() {
        if let Some(client) = clients_res.connected_clients.remove(&client_disconnected.client_id) {
            // TODO: Save his stuff.
            debug!("{:?} disconneced.", &client.connection.client_id);
        }
    }
}

//* last

/// TODO: Send unacknowledged commands.
/// TODO: Just sending every fleets position for now.
fn send_udp(query: Query<(&FleetId, &Position)>, clients_res: Res<ClientsRes>) {
    let fleets_position: Vec<Vec2> = query.iter().map(|(_fleet_id, position)| position.0).collect();

    let packet = UdpServer::Metascape { fleets_position };

    for client in clients_res.connected_clients.values() {
        let _ = client.connection.udp_sender.blocking_send(packet.clone());
    }
}
