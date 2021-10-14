use serde::{Deserialize, Serialize};
use std::{convert::TryInto, ops::Range};

fn main() {
    let m = Monster {
        sprite: 123,
        hp: 101,
        damage: 10,
        speed: 60.0,
        death_drop: vec![
            Drop {
                item: "flesh".to_string(),
                amount: 1..4,
                chance: (1, 4),
            },
            Drop {
                item: "nail".to_string(),
                amount: 10..30,
                chance: (1, 8),
            },
        ],
    };

    println!("{:?}", &m);

    println!("{:?}", bincode::serialize(&m).unwrap());

    let yaml = serde_yaml::to_string(&m).unwrap();
    println!("{}", &yaml);

    let deser = serde_yaml::from_str::<Monster>(&yaml).unwrap();
    println!("{:?}", &deser);

    let deser_generic: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
    let sp: usize = deser_generic.get("sprite").unwrap().as_u64().unwrap().try_into().unwrap();

    assert_eq!(sp, m.sprite);
    assert_eq!(m, deser);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Monster {
    sprite: usize,
    hp: i32,
    damage: i32,
    speed: f32,
    death_drop: Vec<Drop>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Drop {
    item: String,
    amount: Range<u32>,
    chance: (u32, u32),
}
