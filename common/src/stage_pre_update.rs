use crate::{ecs_components::*, fleet_ai::*, packets::*, res_clients::*};
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "pre_update";
    let previous_stage = "first";

    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());

    schedule.add_system_to_stage(current_stage, fleet_ai.system());
}

fn fleet_ai(mut query: Query<(&mut FleetAI, &Position, &mut WishPosition)>, task_pool: Res<TaskPool>) {
    query.par_for_each_mut(&task_pool, 16, |(mut fleet_ai, pos, mut wish_pos)| {
        match fleet_ai.goal {
            FleetGoal::Trade { from, to } => todo!(),
            FleetGoal::Guard { who, radius, duration } => todo!(),
            FleetGoal::Wandering { to, pause } => todo!(),
        }
    });
}

// /// Get and process clients udp packets.
// fn process_client_udp(clients_res: Res<ClientsRes>, query: Query<Option<&mut FleetMetascapeAI>>) {
//     for client in clients_res.connected_clients.values() {
//         // Get the entity controlled by this client.
//         if let Some(current_entiy) = match client.fleet_control {
//             Some(entity) => {
//                 query.get_mut(entity).ok()
//             }
//             None => None,
//         } {
//             loop {
//                 // Receive all udp packets and apply them to the currently controlled entiy.
//                 match client.connection.udp_receiver.try_recv() {
//                     Ok(packet) => {
//                         match packet {
//                             UdpClient::Battlescape {
//                                 wish_input,
//                                 acknowledge_command,
//                             } => {
//                                 // TODO: Set next as battlescape input.
    
//                                 // TODO: Remove an acknowledged command.
    
//                                 todo!();
//                             }
//                             UdpClient::Metascape { wish_position } => {
//                                 // Get controlled fleet.
//                                 if let Some(fleet_metascape_ai) = &current_entiy {
//                                     fleet_metascape_ai = FleetMetascapeAI::GoToPosition(wish_position);
//                                 }
//                             }
//                         }
//                     }
//                     Err(err) => {
//                         if err == crossbeam_channel::TryRecvError::Disconnected {
//                             // TODO: Disconnect event.
//                             // self.disconnect_queue.push(*client_id);
//                         }
//                         break;
//                     }
//                 }
//             }
//         }
//     }
// }