use crate::constants::GAME_TO_GODOT_RATIO;
use gdnative::{core_types::Transform2D, prelude::Vector2};
use glam::Vec2;
use na::*;

pub trait ToGlam {
    fn to_glam(self) -> Vec2;

    /// Convert to a glam vector with `GAME_TO_GODOT_RATIO` scale removed.
    fn to_glam_descaled(self) -> Vec2;
}
impl ToGlam for Vector2 {
    fn to_glam(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    fn to_glam_descaled(self) -> Vec2 {
        Vec2::new(self.x, self.y) / GAME_TO_GODOT_RATIO
    }
}

pub trait ToNalgebra {
    fn to_na(self) -> na::Vector2<f32>;

    /// Convert to a nalgebra vector with `GAME_TO_GODOT_RATIO` scale removed.
    fn to_na_descaled(self) -> na::Vector2<f32>;
}
impl ToNalgebra for Vector2 {
    #[inline(always)]
    fn to_na(self) -> na::Vector2<f32> {
        vector![self.x, self.y]
    }

    #[inline(always)]
    fn to_na_descaled(self) -> na::Vector2<f32> {
        vector![self.x, self.y] / GAME_TO_GODOT_RATIO
    }
}

pub trait ToGodot {
    fn to_godot(self) -> Vector2;

    /// Convert to a godot vector with `GAME_TO_GODOT_RATIO` scale added.
    fn to_godot_scaled(self) -> Vector2;
}
impl ToGodot for Translation<f32, 2_usize> {
    #[inline(always)]
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(self.x, self.y) * GAME_TO_GODOT_RATIO
    }
}
impl ToGodot for na::Vector2<f32> {
    #[inline(always)]
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(self.x, self.y) * GAME_TO_GODOT_RATIO
    }
}
impl ToGodot for Vec2 {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(self.x, self.y) * GAME_TO_GODOT_RATIO
    }
}

pub trait PhysicToGodotTransform {
    /// Convert to a godot's `Transform2D` with `GAME_TO_GODOT_RATIO` scale applied.
    fn to_godot_render_transform_scaled(self) -> Transform2D;
}
impl PhysicToGodotTransform for Isometry<f32, Unit<Complex<f32>>, 2_usize> {
    #[inline(always)]
    fn to_godot_render_transform_scaled(self) -> Transform2D {
        let cos = self.rotation.cos_angle() * GAME_TO_GODOT_RATIO;
        let sin = self.rotation.sin_angle() * GAME_TO_GODOT_RATIO;

        Transform2D {
            a: Vector2::new(cos, sin),
            b: Vector2::new(-sin, cos),
            origin: Vector2::new(self.translation.x, self.translation.y) * GAME_TO_GODOT_RATIO,
        }
    }
}
