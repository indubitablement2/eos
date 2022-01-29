use ahash::AHashMap;
use bevy_ecs::prelude::*;
use common::{factions::Faction, idx::*, orbit::Orbit, reputation::Reputation, WORLD_BOUND};
use glam::Vec2;

//* bundle

#[derive(Bundle)]
pub struct ClientFleetBundle {
    wrapped_client_id: WrappedId<ClientId>,
    know_entities: KnowEntities,
    #[bundle]
    fleet_bundle: FleetBundle,
}
impl ClientFleetBundle {
    pub fn new(
        client_id: ClientId,
        fleet_id: FleetId,
        position: Vec2,
        faction: Option<FactionId>,
    ) -> Self {
        debug_assert_eq!(client_id.to_fleet_id(), fleet_id);

        Self {
            wrapped_client_id: WrappedId::new(client_id),
            know_entities: Default::default(),
            fleet_bundle: FleetBundle::new(fleet_id, position, faction),
        }
    }
}

#[derive(Bundle)]
pub struct ColonistAIFleetBundle {
    pub colonist_fleet_ai: ColonistFleetAI,
    #[bundle]
    pub fleet_bundle: FleetBundle,
}
impl ColonistAIFleetBundle {
    pub fn new(
        target: Option<PlanetId>,
        travel_until: u32,
        fleet_id: FleetId,
        position: Vec2,
        faction: Option<FactionId>,
    ) -> Self {
        Self {
            colonist_fleet_ai: ColonistFleetAI {
                target_planet: target,
                travel_until,
            },
            fleet_bundle: FleetBundle::new(fleet_id, position, faction),
        }
    }
}

#[derive(Bundle)]
pub struct FleetBundle {
    pub name: Name,
    pub wrapped_fleet_id: WrappedId<FleetId>,
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
impl FleetBundle {
    pub fn new(fleet_id: FleetId, position: Vec2, faction: Option<FactionId>) -> Self {
        Self {
            name: Name(format!("{}", fleet_id.0)),
            wrapped_fleet_id: WrappedId::new(fleet_id),
            position: Position(position),
            in_system: InSystem::default(),
            wish_position: WishPosition::default(),
            velocity: Velocity::default(),
            idle_counter: IdleCounter::default(),
            derived_fleet_stats: DerivedFleetStats { acceleration: 0.04 },
            reputations: Reputations {
                faction,
                common_reputation: Default::default(),
            },
            detected_radius: DetectedRadius(10.0),
            detector_radius: DetectorRadius(10.0),
            entity_detected: EntityDetected::default(),
        }
    }
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
#[derive(Debug, Clone, Copy, Component, Default)]
pub struct InSystem(pub Option<SystemId>);

#[derive(Debug, Clone, Component, Default)]
pub struct Reputations {
    pub faction: Option<FactionId>,
    pub common_reputation: Reputation,
}
impl Reputations {
    /// Return the relative reputation between two reputations.
    pub fn get_relative_reputation(
        &self,
        other: &Reputations,
        self_fleet_id: FleetId,
        other_fleet_id: FleetId,
        factions: &[Faction; 32],
    ) -> Reputation {
        if let Some(fac_self) = self.faction {
            if let Some(fac_other) = other.faction {
                let (highest, lowest) = if fac_self > fac_other {
                    (fac_self, fac_other)
                } else if fac_other > fac_self {
                    (fac_other, fac_self)
                } else {
                    return Reputation::MAX;
                };

                factions[highest].relations[lowest.0 as usize]
            } else {
                let faction = &factions[fac_self];
                if let Some(reputation) = faction.reputations.get(&other_fleet_id) {
                    reputation.to_owned()
                } else {
                    faction.default_reputation
                }
            }
        } else {
            if let Some(fac_other) = other.faction {
                let faction = &factions[fac_other];
                if let Some(reputation) = faction.reputations.get(&self_fleet_id) {
                    reputation.to_owned()
                } else {
                    faction.default_reputation
                }
            } else {
                self.common_reputation.min(other.common_reputation)
            }
        }
    }
}

//* Fleet

/// How long this entity has been without velocity.
#[derive(Debug, Clone, Copy, Component, Default)]
pub struct IdleCounter {
    counter: u32,
}
impl IdleCounter {
    /// Delay before a fleet without velocity is considered idle in tick.
    const IDLE_DELAY: u32 = 60;

    pub fn increment(&mut self) {
        self.counter += 1;
    }

    pub fn set_non_idle(&mut self) {
        self.counter = 0;
    }

    pub fn is_idle(self) -> bool {
        self.counter >= Self::IDLE_DELAY
    }

    /// Will return true only when the entity start idling.
    pub fn just_stated_idling(self) -> bool {
        self.counter == Self::IDLE_DELAY
    }
}

/// Where the fleet wish to move.
#[derive(Debug, Clone, Copy, Component)]
pub struct WishPosition {
    /// Where the fleet will try to move to.
    target: Option<Vec2>,
    /// Fleet will cap its movement speed.
    /// This will always be between 0 and 1.
    movement_multiplier: f32,
}
impl WishPosition {
    /// Reset the wish position's target to none.
    pub fn stop(&mut self) {
        // We don't need to reset movement multiplier as it will not be taken into account when stopping.
        self.target = None;
    }

    /// Set the wish position's target and movement multiplier.
    pub fn set_wish_position(&mut self, target: Vec2, movement_multiplier: f32) {
        debug_assert!(
            target.length_squared() < WORLD_BOUND.powi(2),
            "Wish position's target should be within the would bound."
        );
        self.target = Some(target);

        debug_assert!(
            movement_multiplier > 0.0,
            "Movement multiplier should be more than 0."
        );
        debug_assert!(
            movement_multiplier <= 1.0,
            "Movement multiplier should not be more than 1."
        );
        self.movement_multiplier = movement_multiplier;
    }

    /// Get a reference to the wish position's movement multiplier.
    pub fn movement_multiplier(&self) -> f32 {
        self.movement_multiplier
    }

    /// Get a reference to the wish position's target.
    pub fn target(&self) -> Option<Vec2> {
        self.target
    }
}
impl Default for WishPosition {
    fn default() -> Self {
        Self {
            target: None,
            movement_multiplier: 1.0,
        }
    }
}

/// The current velocity of the entity.
#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Velocity(pub Vec2);

/// Fleet statistics that are derived from fleet composition.
#[derive(Debug, Clone, Copy, Component)]
pub struct DerivedFleetStats {
    /// How much velocity this entity can gain each update.
    pub acceleration: f32,
}

/// Fleet that should be removed after a provided tick,
/// if they are not in a battle.
#[derive(Debug, Clone, Copy, Component)]
pub struct QueueRemove {
    pub when: u32,
}

//* AI

/// AI that wants to colonize a random factionless planet.
#[derive(Debug, Clone, Copy, Component)]
pub struct ColonistFleetAI {
    target_planet: Option<PlanetId>,
    /// Fleet will travel at least until this tick before attempting to find a planet.
    travel_until: u32,
}
impl ColonistFleetAI {
    pub const MOVEMENT_MULTIPLIER_TRAVELLING: f32 = 0.5;
    pub const MOVEMENT_MULTIPLIER_COLONIZING: f32 = 0.3;

    /// Return if this AI has completed its minimum travelling time.
    pub fn is_done_travelling(&self, time: u32) -> bool {
        self.travel_until < time
    }

    /// Get a reference to the colonist fleet ai's target planet.
    pub fn target_planet(&self) -> Option<PlanetId> {
        self.target_planet
    }

    /// Set the colonist fleet ai's target planet to none.
    pub fn reset_target_planet(&mut self) {
        self.target_planet = None;
    }

    /// Set the colonist fleet ai's target planet.
    pub fn set_target_planet(&mut self, target_planet: PlanetId) {
        self.target_planet = Some(target_planet);
    }

    /// Set the colonist fleet ai's travel duration.
    pub fn set_travel_until(&mut self, travel_duration: u32, time: u32) {
        self.travel_until = time + travel_duration;
    }
}
impl Default for ColonistFleetAI {
    fn default() -> Self {
        Self {
            target_planet: None,
            travel_until: 3000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColonyFleetAIGoal {
    Trade { colony: PlanetId },
    Guard,
}
/// Ai for fleet that are owned by a colony.
#[derive(Debug, Clone, Copy, Component)]
pub struct ColonyFleetAI {
    pub goal: ColonyFleetAIGoal,
    /// The colony that own this fleet.
    pub colony: PlanetId,
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
pub struct WrappedId<T> {
    id: T,
}
impl<T> WrappedId<T> {
    pub fn new(id: T) -> Self {
        Self { id }
    }

    /// Get the wrapped id's id.
    pub fn id(self) -> T {
        self.id
    }
}
