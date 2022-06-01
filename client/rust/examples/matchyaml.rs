// #![feature(test)]
// extern crate test;
// use std::mem::size_of;

// use bevy_ecs::prelude::Component;
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, Clone)]
// enum Comp {
//     Name(String),
//     Hp(i32),
//     Damage(i32),
//     Speed(f32),
//     Animation(i32),
//     Team(i32),
//     Size(f32),
//     Value(i32),
//     Tag1,
//     Tag2,
//     Arr([u8; 2]),
//     LargeVec(Vec<i32>),
//     Opt(Option<i32>),
// }

// #[bench]
// fn bench_pow(b: &mut test::Bencher) {
//     let data_og = vec![
//         Comp::Name("hello".to_string()),
//         Comp::Hp(22),
//         Comp::Damage(5),
//         Comp::Speed(50.5),
//         Comp::Animation(22),
//         Comp::Team(4),
//         Comp::Size(123.123),
//         Comp::Value(78),
//         Comp::Tag2,
//         Comp::Arr([5, 200]),
//         Comp::LargeVec(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]),
//         Comp::Opt(Some(11)),
//         Comp::Opt(None),
//     ];

//     let file_og = vec![data_og.clone(), data_og.clone(), data_og.clone()];

//     let data = serde_yaml::to_string(&file_og).unwrap();
//     println!("{}\n", &data);

//     let data_parsed: Vec<Vec<Comp>> = serde_yaml::from_str(&data).unwrap();
//     println!("{:?}\n", &data_parsed);

//     let mut valuei = 0i32;
//     let mut valuef = 0f32;
//     data_parsed.iter().for_each(|entity| {
//         entity.iter().for_each(|comp| match comp {
//             Comp::Name(_) => {
//                 valuei += 5i32;
//                 valuef += 5.0f32
//             }
//             Comp::Hp(v) => valuei += v,
//             Comp::Damage(v) => valuei += v,
//             Comp::Speed(v) => valuef += v,
//             Comp::Animation(v) => valuei += v,
//             Comp::Team(v) => valuei += v,
//             Comp::Size(v) => valuef += v,
//             Comp::Value(v) => valuei += v,
//             Comp::Tag2 => {
//                 valuei += 2;
//                 valuef += 2.0
//             }
//             Comp::Tag1 => {
//                 valuei += 1;
//                 valuef += 1.0
//             }
//             Comp::Arr(v) => valuei += i32::from(v[0] + v[1]),
//             Comp::LargeVec(v) => {
//                 for i in v.iter() {
//                     valuei += *i;
//                 }
//             }
//             Comp::Opt(v) => {
//                 if let Some(o) = v {
//                     valuei += *o;
//                 }
//             }
//         });
//     });

//     b.iter(|| {
//         data_parsed = serde_yaml::from_str(&data).unwrap();
//     });
// }

// #[derive(Debug, Serialize, Deserialize)]
// enum EnumOfComponents {
//     ComponentA(ComponentA),
//     ComponentB(ComponentB),
//     ComponentC(ComponentC),
// }

// #[derive(Debug, Serialize, Deserialize, Clone, Copy, Component)]
// struct ComponentA {
//     f0: i32,
//     f1: u8,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, Component)]
// struct ComponentB {
//     f0: String,
//     f1: [u8; 2],
// }

// #[derive(Debug, Serialize, Deserialize, Clone, Copy, Component)]
// struct ComponentC(f32);

// fn main() {
//     let data = vec![
//         EnumOfComponents::ComponentA(ComponentA { f0: 10, f1: 255 }),
//         EnumOfComponents::ComponentB(ComponentB {
//             f0: "Hello bevy!".to_string(),
//             f1: [5, 8],
//         }),
//         EnumOfComponents::ComponentC(ComponentC(123.123)),
//     ];

//     println!("{}", serde_yaml::to_string(&data).unwrap());

//     let mut world = bevy_ecs::prelude::World::new();
//     let mut entity = world.spawn();

//     // Convert enum to struct
//     data.iter().for_each(|enum_of_components| match enum_of_components {
//         EnumOfComponents::ComponentA(v) => {
//             entity.insert(v.to_owned());
//         }
//         EnumOfComponents::ComponentB(v) => {
//             entity.insert(v.to_owned());
//         }
//         EnumOfComponents::ComponentC(v) => {
//             entity.insert(v.to_owned());
//         }
//     });

//     println!("{}\n", size_of::<EnumOfComponents>());

//     let ser = bincode::serialize(&data).unwrap();
//     println!("{:?}, {}\n", &ser, ser.len());
// }

fn main() {}
