use gdnative::prelude::*;
use glam::Vec2;

/// Convert a Vec2 to a Vector2.
#[inline(always)]
pub fn glam_to_godot(glam: Vec2) -> Vector2 {
    Vector2::new(glam.x, glam.y)
}