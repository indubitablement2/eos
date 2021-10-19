use crate::ecs_components::*;
use ahash::AHashMap;
use gdnative::{godot_print, godot_warn};
use indexmap::IndexMap;
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
    BundleReferenceId(BundleReferenceId),
    Name(Name),
    Faction(Faction),
}
impl EcsComponents {
    /// Used mostly for sort/dedup.
    pub fn to_u32(&self) -> u32 {
        match self {
            EcsComponents::BundleReferenceId(_) => 0,
            EcsComponents::Name(_) => 1,
            EcsComponents::Faction(_) => 2,
        }
    }
}

/// Conert YamlComponents to EcsComponents and the path to their sprite asset they.
/// Order need to be preserved in the sprite vector as components will only keep a usize to reference their sprite.
pub fn parse_yaml_components(
    yaml_components: Vec<Vec<Vec<YamlComponents>>>,
) -> Result<(Vec<Vec<EcsComponents>>, Vec<String>), crate::game_def::GameDefLoadError> {
    // TODO: If can't find sprite file, sprite_id = 0.
    // TODO: Auto check for rotated sprite t, tr, r, br, b, bl, l, tl

    // Get the number of bundle we have to parse.
    let num_bundle = yaml_components.iter().fold(0, |acc, yaml_group| {
        acc + yaml_group.iter().fold(0, |acc, yaml_bundle| acc + yaml_bundle.len())
    });

    let mut entity_bundles: IndexMap<String, Vec<EcsComponents>> = IndexMap::with_capacity(num_bundle);

    // The names of factions their id.
    let mut faction_names: AHashMap<String, usize> = AHashMap::with_capacity(32);

    for yaml_group in yaml_components.into_iter() {
        for yaml_bundle in yaml_group.into_iter() {
            // A new bundle where we will add parsed YamlComponents.
            let mut current_bundle: Vec<EcsComponents> = Vec::with_capacity(yaml_bundle.len());

            // This will be used to check if the YamlBundle has a ReferenceId.
            let mut got_reference_id: Option<String> = None;

            for yaml_component in yaml_bundle.into_iter() {
                // Convert YamlComponents to EcsComponents.
                match yaml_component {
                    YamlComponents::ReferenceId(v) => {
                        got_reference_id = Some(v);
                    }
                    YamlComponents::Name(v) => {
                        current_bundle.push(EcsComponents::Name(Name(v)));
                    }
                    YamlComponents::DefaultFaction(v) => {
                        match faction_names.get(&v) {
                            Some(id) => {
                                current_bundle.push(EcsComponents::Faction(Faction(*id)));
                            }
                            None => {
                                let new_id = faction_names.len();
                                faction_names.insert(v, new_id);
                                current_bundle.push(EcsComponents::Faction(Faction(new_id)));
                            }
                        }
                        // let faction_id = faction_names.
                        // entity_bundles[current_id].push(EcsComponents::Faction());
                    }
                    YamlComponents::DeathTrigger(_) => todo!(),
                    YamlComponents::GetHitTrigger(_) => todo!(),
                    YamlComponents::ActiveTrigger(_) => todo!(),
                }
            }
            // Add this new bundle to entity_bundles if it has a referenceId.
            if let Some(ref_id) = got_reference_id {
                // Sort bundle.
                current_bundle.sort_unstable_by_key(|k| k.to_u32());
                // Dedup bundle
                current_bundle.dedup_by(|a, b| a.to_u32().eq(&b.to_u32()));

                // We either overwrite an existing bundle or push our new bundle at the end.
                if let Some(old_bundle) = entity_bundles.get_full(&ref_id) {
                    current_bundle.insert(0, EcsComponents::BundleReferenceId(BundleReferenceId(old_bundle.0)));
                    godot_print!("Overwritten {}", &ref_id);
                } else {
                    current_bundle.insert(0, EcsComponents::BundleReferenceId(BundleReferenceId(entity_bundles.len())));
                }
                current_bundle.shrink_to_fit();
                entity_bundles.insert(ref_id, current_bundle);
            } else {
                godot_warn!("Ignored an EcsBundle without ReferenceId.");
            }
        }
    }

    entity_bundles.shrink_to_fit();

    let mut factions = faction_names.keys().map(|s| s.to_owned()).collect::<Vec<String>>();
    factions.sort_by(|a, b| {
        faction_names
            .get(a)
            .expect("This faction name should exist.")
            .cmp(faction_names.get(b).expect("This faction should exist."))
    });

    todo!()
}
