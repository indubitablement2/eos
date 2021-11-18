use crate::ecs_events::clear_events;
use bevy_ecs::prelude::*;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "last";
    let previous_stage = "post_update";

    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());

    schedule.add_system_to_stage(current_stage, clear_events.system());
}

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