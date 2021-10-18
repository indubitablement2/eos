use crate::ecs_components::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PossibleTarget {
    /// The entity itself.
    Itself,

    /// The tile directly beneat the entity.
    GroundSelf,
    /// The closest tile in front of the moster that has no wall.
    GroundInFront,
    /// Any ground without wall within close distance.
    GroundNearby,

    /// If the trigger imply a target, this is it.
    TriggerTarget,

    NearestEnemyNearby,
    NearestAllyNearby,
    NearestEntityNearby,

    RandomEnemyNearby,
    RandomAllyNearby,
    RandomEntityNearby,

    /// Where the entity is moving.
    WishLocation,
    /// The target the entity chose to attack. This is not always the closest enemy.
    WishTarget,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Effects {
    DropItem { item: String, quantity: (u32, u32), force: f32 },
    CastSpell { spell: Vec<Effects> },
    MoveTo { to: PossibleTarget },
    ChangeFaction { new_faction: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EffectRequirement {
    TargetWithinRange(f32),
    HealthBellowPercent(f32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Effect {
    effect: Effects,
    target: PossibleTarget,
    requirement: Vec<EffectRequirement>,
    chance: (u32, u32),
    cooldown: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum YamlComponents {
    ReferenceId(String),
    Name(String),
    DefaultFaction(String),

    // TODO: Add these.
    DeathTrigger(Vec<Effect>),
    GetHitTrigger(Vec<Effect>),
    ActiveTrigger(Vec<Effect>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EcsComponents {
    Name(Name),
    DefaultFaction(DefaultFaction),
}

/// Conert YamlComponents to EcsComponents and the path to their sprite asset they.
/// Order need to be preserved in the sprite vector as components will only keep a usize to reference their sprite.
pub fn parse_yaml_components(
    yaml_components: &[Vec<YamlComponents>],
) -> Result<(Vec<Vec<EcsComponents>>, Vec<String>), crate::game_def::GameDefLoadError> {
    // TODO: If can't find sprite file, sprite_id = 0.
    // TODO: Auto check for rotated sprite t, tr, r, br, b, bl, l, tl
    todo!()
}
