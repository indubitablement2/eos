use ahash::AHashMap;
use bevy_ecs::prelude::*;
use common::{idx::*, orbit::Orbit, reputation::Reputation, world_data::Faction};
use glam::Vec2;

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
#[derive(Debug, Component)]
pub struct KnowEntities {
    /// The next id we should create, if there are no id to reuse.
    pub next_new_id: u16,
    /// Idx that are safe to reuse.
    pub free_idx: Vec<u16>,
    /// Idx that are pending some duration before being recycled.
    pub pending_idx: (Vec<u16>, Vec<u16>),
    /// Entity that the client has info about and their id.
    pub known: AHashMap<Entity, u16>,
    pub force_update_client_info: bool,
}
impl KnowEntities {
    /// This will break if there are more than 65535 know entities,
    /// but that should not happen.
    pub fn get_new_id(&mut self) -> u16 {
        self.free_idx.pop().unwrap_or_else(|| {
            let id = self.next_new_id;
            self.next_new_id += 1;
            id
        })
    }

    pub fn recycle_pending_idx(&mut self) {
        self.free_idx.extend(self.pending_idx.0.drain(..));
        std::mem::swap(&mut self.pending_idx.0, &mut self.pending_idx.1);
    }

    pub fn recycle_id(&mut self, temp_id: u16) {
        self.pending_idx.1.push(temp_id);
    }
}
impl Default for KnowEntities {
    fn default() -> Self {
        Self {
            next_new_id: 0,
            free_idx: Vec::new(),
            pending_idx: (Vec::new(), Vec::new()),
            known: AHashMap::new(),
            force_update_client_info: true,
        }
    }
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
    pub fn get_relative_reputation(&self, other: &Reputations, factions: &AHashMap<FactionId, Faction>) -> Reputation {
        if let Some(fac_self) = self.faction {
            if let Some(fac_other) = other.faction {
                let (highest, lowest) = if fac_self > fac_other {
                    (fac_self, fac_other)
                } else if fac_other > fac_self {
                    (fac_other, fac_self)
                } else {
                    return Reputation::MAX;
                };

                if let Some(faction) = factions.get(&highest) {
                    if let Some(reputation) = faction.faction_relation.get(&lowest) {
                        reputation.to_owned()
                    } else {
                        debug!(
                            "Can not find reputation {:?} in {:?}. Returning neutral reputation...",
                            lowest, highest
                        );
                        Reputation::NEUTRAL
                    }
                } else {
                    debug!("Can not find {:?}. Returning neutral reputation...", highest);
                    Reputation::NEUTRAL
                }
            } else {
                if let Some(reputation) = other.faction_reputation.get(&fac_self) {
                    reputation.to_owned()
                } else {
                    if let Some(faction) = factions.get(&fac_self) {
                        faction.default_reputation
                    } else {
                        debug!("Can not find {:?}. Returning neutral reputation...", fac_self);
                        Reputation::NEUTRAL
                    }
                }
            }
        } else {
            if let Some(fac_other) = other.faction {
                if let Some(reputation) = self.faction_reputation.get(&fac_other) {
                    reputation.to_owned()
                } else {
                    if let Some(faction) = factions.get(&fac_other) {
                        faction.default_reputation
                    } else {
                        debug!("Can not find {:?}. Returning neutral reputation...", fac_other);
                        Reputation::NEUTRAL
                    }
                }
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
    pub const IDLE_DELAY: u32 = 50;

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
pub struct EntityDetected(pub Vec<Entity>);

//* Idx

#[derive(Debug, Clone, Copy, Component)]
pub struct ClientIdComp(pub ClientId);

#[derive(Debug, Clone, Copy, Component)]
pub struct FleetIdComp(pub FleetId);
