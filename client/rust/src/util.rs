use gdnative::prelude::*;
use glam::Vec2;

/// Convert Vec2 to Vector2.
#[inline(always)]
pub fn glam_to_godot(glam: Vec2) -> Vector2 {
    Vector2::new(glam.x, glam.y)
}

/// Convert Vector2 to Vec2.
#[inline(always)]
pub fn godot_to_glam(godot: Vector2) -> Vec2 {
    Vec2::new(godot.x, godot.y)
}
