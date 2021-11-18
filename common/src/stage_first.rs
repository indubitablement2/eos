use crate::{data_manager::DataManager, res_clients::ClientsRes, res_times::TimeRes};
use bevy_ecs::prelude::*;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "first";

    schedule.add_stage(current_stage, SystemStage::parallel());

    schedule.add_system_to_stage(current_stage, increment_time.system());
    schedule.add_system_to_stage(current_stage, get_new_clients.system());
}

fn increment_time(mut time_res: ResMut<TimeRes>) {
    time_res.tick += 1;
}

/// Get new connection and insert client.
fn get_new_clients(mut clients_res: ResMut<ClientsRes>, data_manager: Res<DataManager>) {
    // while let Ok(connection) = self.connection_manager.new_connection_receiver.try_recv() {
    //     // Load client data.
    //     let client_data = data_manager.load_client(connection.client_id);

    //     let client_id = connection.client_id;

    //     // Create client.
    //     let client = Client {
    //         connection,
    //         fleet_control: None,
    //         client_data,
    //         // input_battlescape: BattlescapeInput::default(),
    //         // unacknowledged_commands: IndexMap::new(),
    //     };

    //     // Add to Metascape.
    //     if self.connected_clients.insert(client_id, client).is_some() {
    //         info!("{:?} was disconnected as a new connection took this client.", client_id);
    //     }
    // }
}