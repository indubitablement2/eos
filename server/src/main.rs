use ahash::{AHashMap, AHashSet, RandomState};
use indexmap::IndexMap;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use tokio::time;

mod simulation;

#[tokio::main]
async fn main() {
    let mut interval = time::interval(time::Duration::from_millis(50));

    // let mut simulation = simulation::Simulation::new();

    // // tokio_tungstenite::accept_async(stream)

    // loop {
    //     interval.tick().await;

    //     simulation.update(0.05);
    //     let packet = serde_json::to_string(&simulation.get_packet()).unwrap();
    // }
}
