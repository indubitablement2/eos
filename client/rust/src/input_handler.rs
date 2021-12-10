use gdnative::api::*;
use glam::Vec2;

pub struct InputHandler {
    /// Mouse position relative to controlled fleet or ship.
    pub relative_mouse_position: Vec2,

}
impl Default for InputHandler {
    fn default() -> Self {
        Self {
            relative_mouse_position: Vec2::ZERO,
        }
    }
}
impl InputHandler {
    pub fn handle_input(&mut self, event: &InputEvent) {

    }
}