use gdnative::core_types::Vector2;

pub struct EcsInput {
    pub tick: u64,
    pub player_inputs: Vec<PlayerInput>,
}

/// Input done by a player.
pub struct PlayerInput {
    pub player_id: u32,
    pub wish_dir: Vector2,
}