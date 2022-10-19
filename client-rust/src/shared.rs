use ahash::AHashMap;
use battlescape::commands::Replay;
use gdnative::prelude::{Node2D, Ref};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::godot_client_config::GodotClientConfig;

pub static SHARED: Lazy<RwLock<Shared>> = Lazy::new(|| RwLock::new(Default::default()));

#[derive(Debug, Default)]
pub struct Shared {
    pub client_config: GodotClientConfig,

    pub client_battlescape_data: AHashMap<u32, ClientBattlescapeData>,
}

#[derive(Debug, Default)]
pub struct ClientBattlescapeData {
    pub taken: Option<Ref<Node2D>>,
    pub replay: Replay,
}
