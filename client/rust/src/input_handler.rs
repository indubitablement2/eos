use crate::util::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

pub struct InputHandler {
    /// Mouse position relative to controlled fleet.
    pub relative_mouse_position: Vec2,
    pub primary: bool,
}
impl Default for InputHandler {
    fn default() -> Self {
        Self {
            relative_mouse_position: Vec2::ZERO,
            primary: false,
        }
    }
}
impl InputHandler {
    /// Reset to default values.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn update(&mut self, owner: &Node2D) {
        self.relative_mouse_position = godot_to_glam(owner.get_global_mouse_position());
    }

    pub fn handle_input(&mut self, event: TRef<InputEvent>) {
        if event.is_action_pressed("primary", false) {
            self.primary = true;
        } else if event.is_action_released("primary") {
            self.primary = false;
        }
    }
}