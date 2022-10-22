pub mod event;

use event::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InputMap {
    pub bc_fire_alt: Option<KeyCodeSerde>,
    pub bc_left: Option<KeyCodeSerde>,
    pub bc_right: Option<KeyCodeSerde>,
    pub bc_up: Option<KeyCodeSerde>,
    pub bc_down: Option<KeyCodeSerde>,
}
impl Default for InputMap {
    fn default() -> Self {
        Self {
            bc_fire_alt: None,
            bc_left: Some(KeyCodeSerde::Left),
            bc_right: Some(KeyCodeSerde::Right),
            bc_up: Some(KeyCodeSerde::Up),
            bc_down: Some(KeyCodeSerde::Down),
        }
    }
}

/// Map raw inputs to game actions.
#[derive(Debug, Clone)]
pub struct PlayerInputs {
    /// The strenght of the horizontal wish direction.
    /// This can be any values.
    /// It should be clamped to a unit vertor before use.
    pub wish_dir: Vec2,

    pub bc_fire: bool,
}
impl Default for PlayerInputs {
    fn default() -> Self {
        Self {
            wish_dir: Vec2::ZERO,
            bc_fire: false,
        }
    }
}
impl PlayerInputs {
    pub fn update(&mut self, map: &InputMap) {
        let x = is_key_down_option(map.bc_right) as i32 - is_key_down_option(map.bc_left) as i32;
        let y = is_key_down_option(map.bc_down) as i32 - is_key_down_option(map.bc_up) as i32;
        self.wish_dir = vec2(x as f32, y as f32);

        self.bc_fire =
            is_key_down_option(map.bc_fire_alt) | is_mouse_button_down(MouseButton::Left);
    }
}

fn is_key_down_option(k: Option<KeyCodeSerde>) -> bool {
    if let Some(k) = k {
        is_key_down(k.into())
    } else {
        false
    }
}
