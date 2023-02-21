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
use smallvec::{smallvec, SmallVec};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// 1 simulation unit = 128 godot unit.
pub const GODOT_SCALE: f32 = 128.0;

// TODO: add_ship should also add an ai!
// TODO: Remove body/colliders when removing entity.
// TODO: Make spawn point static.

// TODO: Add client take control events. Use it on the render side to follow that entity. <---
// TODO: Handle battle over event.
// TODO: Render call should call a function on the sim node with the render node as an argument.
// TODO: TimeManager should handle fast mode/catching up mode itself.

// TODO: Partition hull armor.
// TODO: Get damage texture and add to hull sprite.

// TODO: Shared connection to the server. Dispatch packets to apropriate node

// TODO: Rename bc to bs and mc to ms
// TODO: Split event handler into catching-up, normal, full.
// TODO: Do not create node when taking render events.

// TODO: When serializing/deserializing, check if anything is an EntityScript then convert it to its id.

// // TODO: Split render/sim data. Hull should have a sprite offset from its collider.
// TODO: Use these render offset.
// TODO: Make data public as a & only. Remove `helper`

mod ext {
    use godot::prelude::*;

    struct Client {}
    #[gdextension]
    unsafe impl ExtensionLibrary for Client {}
}
