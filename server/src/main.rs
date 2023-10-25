use ahash::{AHashMap, AHashSet, RandomState};
use battlescape::entity::{EntityData, EntityDataId};
use indexmap::IndexMap;
use rand::prelude::*;
use rapier2d::na::{self, Vector2};
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

mod battlescape;
mod central_server;
mod logger;
mod metascape;

#[tokio::main]
async fn main() {
    logger::Logger::init();

    EntityData::set_data(vec![EntityData::default()]);

    central_server::CentralServer::start().await;

    let mut simulation = battlescape::Battlescape::new();
    simulation.spawn_entity(EntityDataId(0), Default::default());

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // simulation.step();
        // let Packet = serde_json::to_string(&Packet {
        //     time: simulation.tick,
        //     entities: simulation
        //         .entities
        //         .iter()
        //         .map(|(entity_id, entity)| {
        //             let position = simulation.physics.bodies[entity.rb].position();
        //             EntityPacket {
        //                 entity_id: entity_id.0,
        //                 entity_data_id: entity.entity_data_id.0,
        //                 translation: position.translation.vector.into(),
        //                 angle: position.rotation.angle(),
        //             }
        //         })
        //         .collect(),
        // })
        // .unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Packet {
    time: u64,
    entities: Vec<EntityPacket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityPacket {
    entity_id: u64,
    entity_data_id: u32,
    translation: [f32; 2],
    angle: f32,
}
