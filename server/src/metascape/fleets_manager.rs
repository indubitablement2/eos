use super::ecs_components::ClientFleetBundle;
use super::ecs_components::ColonistAIFleetBundle;
use ahash::AHashMap;
use bevy_ecs::prelude::*;
use common::idx::*;
use glam::Vec2;

pub struct FleetsManager {
    spawned_fleets: AHashMap<FleetId, Entity>,
    last_used_id: u64,

    /// Saved client fleets.
    client_fleets: AHashMap<ClientId, ()>,
}
impl FleetsManager {
    /// Get a new unique/never recycled ai fleet id.
    fn get_new_fleet_id(&mut self) -> FleetId {
        self.last_used_id += 1;
        FleetId(self.last_used_id)
    }

    pub fn spawn_default_client_fleet(&mut self, commands: &mut Commands, client_id: ClientId) {
        let fleet_id = client_id.to_fleet_id();

        // Create a default client fleet.
        let new_entity = commands
            .spawn_bundle(ClientFleetBundle::new(
                client_id,
                fleet_id,
                Vec2::ZERO,
                None,
            ))
            .id();

        debug!("Created a new default fleet for {:?}.", client_id);

        if self.spawned_fleets.insert(fleet_id, new_entity).is_some() {
            error!(
                "{:?}'s fleet was overwritten. World and fleets manager are out of sync.",
                client_id
            );
        }
    }

    pub fn spawn_colonist_ai_fleet(
        &mut self,
        commands: &mut Commands,
        target: Option<PlanetId>,
        travel_until: u32,
        position: Vec2,
        faction: Option<FactionId>,
    ) {
        let fleet_id = self.get_new_fleet_id();

        let entity = commands
            .spawn()
            .insert_bundle(ColonistAIFleetBundle::new(
                target,
                travel_until,
                fleet_id,
                position,
                faction,
            ))
            .id();

        self.spawned_fleets.insert(fleet_id, entity);
    }

    /// Return the entity of an existing fleet.
    pub fn get_spawned_fleet(&self, fleet_id: FleetId) -> Option<Entity> {
        self.spawned_fleets.get(&fleet_id).copied()
    }

    /// Remove the fleet and save it, if it is owned by client.
    pub fn remove_spawned_fleet(&mut self, commands: &mut Commands, fleet_id: FleetId) {
        if let Some(entity) = self.spawned_fleets.remove(&fleet_id) {
            commands.entity(entity).despawn();

            if let Some(client_id) = fleet_id.to_client_id() {
                // TODO: Save client's fleet.
                self.client_fleets.insert(client_id, ());

                debug!("Removed and saved {:?}'s fleet.", client_id);
            }
        } else {
            debug!(
                "Remove spawned fleet was called for an unexisting fleet ({:?}). Ignoring...",
                fleet_id
            );
        }
    }
}
impl Default for FleetsManager {
    fn default() -> Self {
        Self {
            spawned_fleets: Default::default(),
            last_used_id: u32::MAX as u64,
            client_fleets: Default::default(),
        }
    }
}
