use super::ecs_components::*;
use super::fleets_manager::FleetsManager;
use bevy_ecs::prelude::*;
use common::idx::*;

pub fn spawn_default_client_fleet(
    fleets_manager: &mut FleetsManager,
    commands: &mut Commands,
    client_fleet_bundle: ClientFleetBundle,
) -> Entity {
    let client_id = client_fleet_bundle.client_id();
    let fleet_id = client_fleet_bundle.fleet_bundle().fleet_id();

    // Add to ecs.
    let new_entity = commands.spawn_bundle(client_fleet_bundle).id();

    // Add to fleet manager.
    fleets_manager.add_spawned_fleet(fleet_id, new_entity);

    // TODO: Add to faction manager.

    log::debug!("Creating a new default fleet for {:?}.", client_id);
    new_entity
}

// pub fn spawn_colonist_ai_fleet(
//     &mut self,
//     commands: &mut Commands,
//     target: Option<PlanetId>,
//     travel_until: u32,
//     position: Vec2,
//     faction: Option<FactionId>,
// ) {
//     let fleet_id = self.get_new_fleet_id();

//     let entity = commands
//         .spawn()
//         .insert_bundle(ColonistAIFleetBundle::new(
//             target,
//             travel_until,
//             fleet_id,
//             position,
//             faction,
//             vec![ShipInfo {
//                 ship: ShipBaseId::from_raw(0),
//                 weapons: Vec::new(),
//             }],
//         ))
//         .id();

//     self.spawned_fleets.insert(fleet_id, entity);
// }

// TODO: Spawn loot and what not.
// TODO: Clean up faction as well.
pub fn destroy_fleet(
    fleets_manager: &mut FleetsManager,
    commands: &mut Commands,
    fleet_id: FleetId,
) {
    if let Some(client_id) = fleet_id.to_client_id() {
        // TODO: Handle client's fleet destruction.
        log::error!("Fleet destruction not implement for client...");
    } else if let Some(entity) = fleets_manager.remove_spawned_fleet(fleet_id) {
        commands.entity(entity).despawn();
    } else {
        log::warn!(
            "Remove spawned fleet was called for an unexisting fleet ({:?}). Ignoring...",
            fleet_id
        );
    }

    todo!()
}
