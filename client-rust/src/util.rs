use crate::constants::GAME_TO_GODOT_RATIO;
use gdnative::{
    core_types::{Color, Transform2D},
    prelude::Vector2,
};

pub trait ToNalgebra {
    fn to_na(self) -> na::Vector2<f32>;

    /// Convert to a nalgebra vector with `GAME_TO_GODOT_RATIO` scale removed.
    fn to_na_descaled(self) -> na::Vector2<f32>
    where
        Self: Sized,
    {
        self.to_na() / GAME_TO_GODOT_RATIO
    }
}
impl ToNalgebra for Vector2 {
    fn to_na(self) -> na::Vector2<f32> {
        na::vector![self.x, self.y]
    }
}
impl ToNalgebra for glam::Vec2 {
    fn to_na(self) -> na::Vector2<f32> {
        na::vector![self.x, self.y]
    }
}

pub trait ToGodot {
    fn to_godot(self) -> Vector2;

    /// Convert to a godot vector with `GAME_TO_GODOT_RATIO` scale added.
    fn to_godot_scaled(self) -> Vector2
    where
        Self: Sized,
    {
        self.to_godot() * GAME_TO_GODOT_RATIO
    }
}
impl ToGodot for na::Translation2<f32> {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
}
impl ToGodot for na::Vector2<f32> {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
}
impl ToGodot for glam::Vec2 {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
}

pub trait PhysicToGodotTransform {
    /// Convert to a godot's `Transform2D` with `GAME_TO_GODOT_RATIO` scale applied.
    fn to_godot_transform_scaled(self) -> Transform2D;
}
impl PhysicToGodotTransform for na::Isometry2<f32> {
    fn to_godot_transform_scaled(self) -> Transform2D {
        let cos = self.rotation.cos_angle() * GAME_TO_GODOT_RATIO;
        let sin = self.rotation.sin_angle() * GAME_TO_GODOT_RATIO;

        Transform2D {
            a: Vector2::new(cos, sin),
            b: Vector2::new(-sin, cos),
            origin: Vector2::new(self.translation.x, self.translation.y) * GAME_TO_GODOT_RATIO,
        }
    }
}

pub trait SetAlpha {
    /// Return the same color, but with the provided alpha.
    fn with_alpha(self, a: f32) -> Self;
}
impl SetAlpha for Color {
    fn with_alpha(self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
}
