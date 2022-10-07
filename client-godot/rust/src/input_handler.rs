use crate::util::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

#[derive(Debug, Clone)]
pub struct PlayerInputs {
    /// The global position of the mouse in game unit.
    pub global_mouse_position: Vec2,
    /// The strenght of the horizontal wish direction.
    /// This can be any values.
    /// It will be clamped to a unit vertor before use.
    pub wish_dir_x: f32,
    /// The strenght of the vertical wish direction.
    /// This can be any values.
    /// It will be clamped to a unit vertor before use.
    pub wish_dir_y: f32,
    pub primary: bool,
}
impl Default for PlayerInputs {
    fn default() -> Self {
        Self {
            global_mouse_position: Vec2::ZERO,
            wish_dir_x: 0.0,
            wish_dir_y: 0.0,
            primary: false,
        }
    }
}
impl PlayerInputs {
    pub fn update(&mut self, owner: &Node2D) {
        let input = Input::godot_singleton();

        // Mouse
        self.global_mouse_position = owner.get_global_mouse_position().to_glam_descaled();

        // Direction
        self.wish_dir_x = input.get_action_strength("right", false) as f32
            - input.get_action_strength("left", false) as f32;
        self.wish_dir_y = input.get_action_strength("backward", false) as f32
            - input.get_action_strength("forward", false) as f32;
    }

    pub fn handle_input(&mut self, event: TRef<InputEvent>) {
        if event.is_action_pressed("primary", false, false) {
            self.primary = true;
        } else if event.is_action_released("primary", false) {
            self.primary = false;
        }
    }
}
