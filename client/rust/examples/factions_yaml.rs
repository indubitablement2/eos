use ahash::{AHashMap, AHashSet};
use common::{
    factions::Factions,
    idx::{FactionId, PlanetId, SystemId},
    reputation::Reputation,
};

fn main() {
    let mut f = Factions::default();

    let colonies = AHashSet::from_iter(
        vec![
            PlanetId {
                system_id: SystemId(10),
                planets_offset: 1,
            },
            PlanetId {
                system_id: SystemId(0),
                planets_offset: 4,
            },
        ]
        .into_iter(),
    );
    let mut faction_relation = AHashMap::new();
    faction_relation.insert(FactionId(2), Reputation::ENEMY_THRESHOLD);

    f.factions.insert(
        common::idx::FactionId(3),
        common::factions::Faction {
            name: "a name".to_string(),
            colonies,
            faction_relation,
            default_reputation: Reputation::NEUTRAL,
        },
    );

    let yaml = serde_yaml::to_string(&f).unwrap();

    println!("{}", yaml);
    println!("{:#?}", serde_yaml::from_str::<Factions>(yaml.as_str()).unwrap());
}
