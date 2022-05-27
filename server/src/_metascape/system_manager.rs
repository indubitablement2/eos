use super::ecs_components::*;
use ahash::{AHashMap, AHashSet};
use bevy_ecs::prelude::*;
use common::idx::SystemId;
use common::intersection::*;

#[derive(Default)]
pub struct SystemInner {
    pub fleets: AHashSet<Entity>,
    pub fleet_detected_acc: AccelerationStructure<Entity, NoFilter>,
    last_update: u32,
}
impl SystemInner {
    pub fn update_acceleration_structure(
        &mut self,
        query: Query<(&Position, &DetectedRadius)>,
        tick: u32,
    ) {
        self.last_update = tick;
        self.fleet_detected_acc.clear();
        self.fleet_detected_acc
            .extend(self.fleets.iter().filter_map(|&entity| {
                if let Ok((position, detected_radius)) = query.get(entity) {
                    Some((
                        Collider::new(detected_radius.0, position.0),
                        entity,
                        NoFilter::default(),
                    ))
                } else {
                    None
                }
            }));
        self.fleet_detected_acc.update();
    }

    /// Get the last update tick.
    #[must_use]
    pub fn last_update(&self) -> u32 {
        self.last_update
    }
}

pub struct FleetInSystem {
    pub fleet_in_systems: AHashMap<SystemId, SystemInner>,
}
