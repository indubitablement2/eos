use super::ecs_components::*;
use ahash::{AHashMap, AHashSet};
use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use common::idx::{BattlescapeId, InterceptionId};
use glam::Vec2;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub enum InterceptedReason {
    Battle(BattlescapeId),
}

#[derive(Debug)]
pub struct Interception {
    pub center: Vec2,
    pub entities: Vec<Entity>,
    pub reason: InterceptedReason,
}

#[derive(Debug, Default)]
pub struct InterceptionManager {
    next_id: u32,
    interceptions: AHashMap<InterceptionId, Interception>,
    to_update: AHashSet<InterceptionId>,
}
impl InterceptionManager {
    /// Join or create an interception with the other entity.
    /// # Panic
    /// Both entities should be valid.
    // pub fn join_interception(
    //     &mut self,
    //     commands: &mut Commands,
    //     query_intercepted: Query<&WrappedId<InterceptionId>>,
    //     initiator: Entity,
    //     other: Entity,
    //     center: Vec2,
    //     reason: InterceptedReason
    // ) -> InterceptionId {
    //     debug_assert!(query_intercepted.get(initiator).is_err(), "Tryied to initiate an interception when already intercepted.");

    //     if let Ok(wrapped_interception_id) = query_intercepted.get(other) {
    //         if let Some(interception) = self.get_interception_mut(wrapped_interception_id.id()) {
    //             match interception.reason {
    //                 InterceptedReason::Battle(battlescape_id) => {
    //                     // Join this battlescape.
    //                 }
    //             }
    //         }
    //     } else {
    //         // Create a new interception.
    //     }

    //     let id = InterceptionId::from_raw(self.next_id);
    //     self.next_id = self.next_id.wrapping_add(1);

    //     self.interceptions
    //         .insert(id, Interception { center, entities, reason });

    //     self.to_update.insert(id);

    //     id
    // }

    pub fn get_interception_mut(
        &mut self,
        interception_id: InterceptionId,
    ) -> Option<&mut Interception> {
        let result = self.interceptions.get_mut(&interception_id);

        if result.is_some() {
            self.to_update.insert(interception_id);
        }

        result
    }

    pub fn update(
        &mut self,
        mut query: bevy_ecs::system::Query<(&Size, Option<&mut WishPosition>)>,
    ) {
        for interception_id in self.to_update.drain() {
            let remove = if let Some(interception) = self.interceptions.get_mut(&interception_id) {
                let mut queue_remove = Vec::new();

                // Compute the circonference and the radius of the interception.
                let mut circonference =
                    interception
                        .entities
                        .iter()
                        .enumerate()
                        .fold(0.0, |acc, (i, &entity)| {
                            if let Ok((size, wish_position)) = query.get(entity) {
                                if wish_position.is_some() {
                                    acc + size.radius
                                } else {
                                    acc
                                }
                            } else {
                                // This entity does not exist.
                                queue_remove.push(i);
                                acc
                            }
                        });
                circonference *= 2.0;
                let r = circonference / TAU;

                // Remove invalid entities.
                debug_assert!(queue_remove.is_sorted());
                for i in queue_remove.into_iter().rev() {
                    interception.entities.swap_remove(i);
                }

                let mut iter = interception.entities.iter();

                // Handle the first one separately.
                let mut used = iter
                    .find_map(|&entity| {
                        if let Ok((size, wish_position)) = query.get_mut(entity) {
                            if let Some(mut wish_position) = wish_position {
                                wish_position.set_wish_position(
                                    Vec2::new(0.0, r) + interception.center,
                                    1.0,
                                );

                                Some(size.radius)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                // Handle the rest.
                for &entity in iter.into_iter() {
                    if let Ok((size, wish_position)) = query.get_mut(entity) {
                        if let Some(mut wish_position) = wish_position {
                            used += size.radius;

                            let angle = used / TAU;
                            wish_position.set_wish_position(
                                Vec2::new(angle.cos(), angle.sin()) * r + interception.center,
                                2.0,
                            );

                            used += size.radius;
                        }
                    }
                }

                interception.entities.is_empty()
            } else {
                false
            };

            // The interception is empty and can be removed.
            if remove {
                let result = self.interceptions.remove(&&interception_id);
                debug_assert!(result.is_some());
            }
        }
    }
}
