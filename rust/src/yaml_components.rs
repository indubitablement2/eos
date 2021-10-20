use crate::ecs_components::*;
use gdnative::{godot_print, godot_warn};
use indexmap::{IndexMap, IndexSet};
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
    Sprite(String),
    Color(Vec<f32>),

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
    Sprite(Sprite),
}
impl EcsComponents {
    /// Used mostly for sort/dedup.
    pub fn to_u32(&self) -> u32 {
        match self {
            EcsComponents::BundleReferenceId(_) => 0,
            EcsComponents::Name(_) => 1,
            EcsComponents::Faction(_) => 2,
            EcsComponents::Sprite(_) => 3,
        }
    }
}

pub struct YamlParseResult {
    pub entity_bundles: IndexMap<String, Vec<EcsComponents>>,
    pub factions: Vec<String>,
    pub sprites: Vec<String>,
}
impl YamlParseResult {
    /// Conert YamlComponents to EcsComponents and the path to their sprite assets.
    /// Order need to be preserved in the resulting sprites vector as components will only keep a usize to reference their sprites.
    pub fn parse_yaml_components(yaml_components: Vec<Vec<Vec<YamlComponents>>>) -> Self {
        // TODO: Don't try to find sprite location. This is the job of the sprite packer.
        // TODO: If can't find sprite file, sprite_id = 0.
        // TODO: Auto check for rotated sprite t, tr, r, br, b, bl, l, tl

        // Get the number of bundle we have to parse.
        let num_bundle = yaml_components.iter().fold(0, |acc, yaml_group| {
            acc + yaml_group.iter().fold(0, |acc, yaml_bundle| acc + yaml_bundle.len())
        });

        // The name of a bundle with the bundle itself. Order is important.
        let mut entity_bundles: IndexMap<String, Vec<EcsComponents>> = IndexMap::with_capacity(num_bundle);

        // The names of factions their id.
        let mut faction_names: IndexSet<String> = IndexSet::with_capacity(32);

        // The path to sprites. Does not include auto rotation like t, tr, r, etc.
        let mut sprite_paths: IndexSet<String> = IndexSet::with_capacity(num_bundle * 6);
        // Sprite [0] is reserved for error.
        sprite_paths.insert("error".to_string());

        for yaml_group in yaml_components.into_iter() {
            for yaml_bundle in yaml_group.into_iter() {
                // A new bundle where we will add parsed YamlComponents.
                let mut current_bundle: Vec<EcsComponents> = Vec::with_capacity(yaml_bundle.len());

                // This will be used to check if the YamlBundle has a ReferenceId.
                let mut got_reference_id: Option<String> = None;

                // This will be used to combine YamlComponents Color and Sprite together.
                let mut sprite_component_pos = 0usize;

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
                            let (id, _) = faction_names.insert_full(v);
                            current_bundle.push(EcsComponents::Faction(Faction(id)));
                        }
                        YamlComponents::Sprite(v) => {
                            let (id, _) = sprite_paths.insert_full(v);

                            if let Some(comp) = current_bundle.get_mut(sprite_component_pos) {
                                if let EcsComponents::Sprite(ecs_sprite_comp) = comp {
                                    ecs_sprite_comp.sprite_id = id;
                                }
                            } else {
                                sprite_component_pos = current_bundle.len();
                                current_bundle.push(EcsComponents::Sprite(Sprite { sprite_id: id, color: [1.0; 4] }));
                            }
                        }
                        YamlComponents::Color(v) => {
                            // Parse color as we only know it is an array right now.
                            let mut new_col = [1.0; 4];
                            new_col.iter_mut().enumerate().for_each(|(i, c)| {
                                if let Some(new_c) = v.get(i) {
                                    *c = new_c.clamp(0.0, 1.0);
                                }
                            });

                            if let Some(comp) = current_bundle.get_mut(sprite_component_pos) {
                                if let EcsComponents::Sprite(ecs_sprite_comp) = comp {
                                    ecs_sprite_comp.color = new_col;
                                }
                            } else {
                                sprite_component_pos = current_bundle.len();
                                current_bundle.push(EcsComponents::Sprite(Sprite { sprite_id: 0, color: new_col }));
                            }
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
                    // Dedup bundle.
                    current_bundle.dedup_by(|a, b| a.to_u32().eq(&b.to_u32()));

                    // Add BundleReferenceId at start.
                    if let Some(old_bundle) = entity_bundles.get_full(&ref_id) {
                        current_bundle.insert(0, EcsComponents::BundleReferenceId(BundleReferenceId(old_bundle.0)));
                        godot_print!("Overwritten {}", &ref_id);
                    } else {
                        current_bundle.insert(0, EcsComponents::BundleReferenceId(BundleReferenceId(entity_bundles.len())));
                    }
                    current_bundle.shrink_to_fit();
                    // Insert maybe replacing a bundle with the same name.
                    entity_bundles.insert(ref_id, current_bundle);
                } else {
                    godot_warn!("Ignored an EcsBundle without ReferenceId.");
                }
            }
        }

        entity_bundles.shrink_to_fit();

        // Transform hashsets to vectors.
        let factions = faction_names.into_iter().collect::<Vec<String>>();
        let sprites = sprite_paths.into_iter().collect::<Vec<String>>();

        Self {
            entity_bundles,
            factions,
            sprites,
        }
    }
}
