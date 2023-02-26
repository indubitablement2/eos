#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#![feature(map_try_insert)]
#![feature(is_some_and)]
#![feature(variant_count)]
#![feature(option_get_or_insert_default)]

mod battlescape;
mod client;
mod client_battlescape;
mod client_config;
mod data;
mod data_builder;
mod godot_logger;
mod metascape;
mod player_inputs;
mod time_manager;
mod util;

pub use data::*;
pub use metascape::{ClientId, FleetId};

use ahash::{AHashMap, AHashSet, RandomState};
use indexmap::IndexMap;
use rapier2d::na::{self, ComplexField, RealField};
use serde::{Deserialize, Serialize};
// use smallvec::{smallvec, SmallVec};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// 1 simulation unit = 128 godot unit.
pub const GODOT_SCALE: f32 = 128.0;

// TODO: Remove body/colliders when removing entity.

// TODO: Add client take control events. Use it on the render side to follow that entity. <---
// TODO: Handle battle over event.
// TODO: Split event handler into catching-up, normal, full.
// TODO: TimeManager should handle fast mode/catching up mode itself.

// TODO: Partition hull armor.
// TODO: Get damage texture and add to hull sprite.
// TODO: Add entity parent/child.
// TODO: Force spawn ship if fleet can not retread and timer is over.
// // TODO: Linvel/angvel

// TODO: Shared connection to the server. Dispatch packets to apropriate node

// TODO: When serializing/deserializing, check if anything is an EntityScript then convert it to its id.
// TODO: Add optional array of entity path to data which will be converted to EntityId and given to EntityScript.
// TODO: Render call should call a function on the sim node with the render node as an argument.

mod ext {
    use godot::prelude::*;

    struct Client {}
    #[gdextension]
    unsafe impl ExtensionLibrary for Client {}
}
