use ahash::AHashMap;
use bevy_ecs::prelude::*;
use common::{idx::*, orbit::Orbit, reputation::Reputation, world_data::{WorldData, Faction}};
use glam::Vec2;
use std::{
    collections::VecDeque,
};

//* bundle

#[derive(Bundle)]
pub struct ClientFleetBundle {
    pub client_id_comp: ClientIdComp,
    pub know_entities: KnowEntities,
    #[bundle]
    pub fleet_bundle: FleetBundle,
}

#[derive(Bundle)]
pub struct FleetBundle {
    pub name: Name,
    pub fleet_id_comp: FleetIdComp,
    pub position: Position,
    pub in_system: InSystem,
    pub wish_position: WishPosition,
    pub velocity: Velocity,
    pub idle_counter: IdleCounter,
    pub derived_fleet_stats: DerivedFleetStats,
    pub reputations: Reputations,
    pub detected_radius: DetectedRadius,
    pub detector_radius: DetectorRadius,
    pub entity_detected: EntityDetected,
}

//* Client

/// Entity we have sent informations to the client.
///
/// Instead of sending the whole entity id, we identify entities with a temporary 8bits id.
#[derive(Debug, Default, Component)]
pub struct KnowEntities {
    pub free_idx: VecDeque<u8>,
    pub known: AHashMap<Entity, u8>,
}

// * Generic

/// A standard position relative to the world origin.
#[derive(Debug, Clone, Copy, Component)]
pub struct Position(pub Vec2);

#[derive(Debug, Clone, Copy, Component)]
pub struct OrbitComp(pub Orbit);

/// An entity's display name.
#[derive(Debug, Clone, Component)]
pub struct Name(pub String);

/// If this entity is within a system.
#[derive(Debug, Clone, Copy, Component)]
pub struct InSystem(pub Option<SystemId>);

#[derive(Debug, Clone, Component, Default)]
pub struct Reputations {
    pub faction: Option<FactionId>,
    pub faction_reputation: AHashMap<FactionId, Reputation>,
    pub common_reputation: Reputation,
}
impl Reputations {
    /// Return the relative reputation between two reputations.
    pub fn get_relative_reputation(&self, other: &Reputations, factions: &[Faction]) -> Reputation {
        if let Some(fac_self) = self.faction {
            if let Some(fac_orther) = other.faction {
                if fac_self > fac_orther {
                    factions[fac_self].faction_relation[usize::from(fac_orther.0)]
                } else if fac_orther > fac_self {
                    factions[fac_orther].faction_relation[usize::from(fac_self.0)]
                } else {
                    self.common_reputation.min(other.common_reputation)
                }
            } else {
                other.faction_reputation.get(&fac_self).copied().unwrap_or_else( ||
                    factions[fac_self].default_common_reputation
                )
            }
        } else {
            if let Some(fac_other) = other.faction {
                self.faction_reputation.get(&fac_other).copied().unwrap_or_else(|| factions[fac_other].default_common_reputation)
            } else {
                self.common_reputation.min(other.common_reputation)
            }
        }
    }
}

//* Fleet

/// How long this entity has been without velocity.
#[derive(Debug, Clone, Copy, Component)]
pub struct IdleCounter(pub u32);
impl IdleCounter {
    /// Delay before a fleet without velocity is considered idle in tick.
    pub const IDLE_DELAY: u32 = 200;

    pub fn is_idle(self) -> bool {
        self.0 >= Self::IDLE_DELAY
    }

    /// Will return true only when the entity start idling.
    pub fn just_stated_idling(self) -> bool {
        self.0 == Self::IDLE_DELAY
    }
}

/// Where the fleet wish to move.
#[derive(Debug, Clone, Copy, Default, Component)]
pub struct WishPosition(pub Option<Vec2>);

/// The current velocity of the entity.
#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Velocity(pub Vec2);

/// Fleet statistics that are derived from fleet composition.
#[derive(Debug, Clone, Copy, Component)]
pub struct DerivedFleetStats {
    /// How much velocity this entity can gain each update.
    pub acceleration: f32,
}

//* AI

#[derive(Debug, Clone, Copy)]
pub enum ColonyFleetAIGoal {
    Trade { colony: Entity },
    Guard { duration: i32 },
}
/// Ai for fleet that are owned by a colony.
#[derive(Debug, Clone, Copy, Component)]
pub struct ColonyFleetAI {
    pub goal: ColonyFleetAIGoal,
    /// The colony that own this fleet.
    pub colony: Entity,
}

//* Detection

/// Used to make an entity detectable.
#[derive(Debug, Clone, Copy, Component)]
pub struct DetectedRadius(pub f32);

/// Used to detect entity that have a DetectedRadius.
#[derive(Debug, Clone, Copy, Component)]
pub struct DetectorRadius(pub f32);

/// Entity id that are detected by this entity.
#[derive(Debug, Clone, Default, Component)]
pub struct EntityDetected(pub Vec<u32>);

//* Idx

#[derive(Debug, Clone, Copy, Component)]
pub struct ClientIdComp(pub ClientId);

#[derive(Debug, Clone, Copy, Component)]
pub struct FleetIdComp(pub FleetId);
