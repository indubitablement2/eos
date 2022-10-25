use glam::{vec2, Vec2};
use std::f32::consts::{PI, TAU};

pub trait Lerp {
    fn lerp(self, to: Self, t: f32) -> Self;
    fn slerp(self, to: Self, t: f32) -> Self;
}
impl Lerp for f32 {
    fn lerp(self, to: Self, t: f32) -> Self {
        t.mul_add(to - self, self)
    }

    fn slerp(self, to: Self, t: f32) -> Self {
        let delta = ((to - self + TAU + PI) % TAU) - PI;
        (self + delta * t + TAU) % TAU
    }
}

pub trait ToNalgebra {
    fn to_na(self) -> na::Vector2<f32>;
}
impl ToNalgebra for Vec2 {
    fn to_na(self) -> na::Vector2<f32> {
        na::vector![self.x, self.y]
    }
}

pub trait ToGlam {
    fn to_glam(self) -> Vec2;
}
impl ToGlam for na::Translation2<f32> {
    fn to_glam(self) -> Vec2 {
        vec2(self.x, self.y)
    }
}
impl ToGlam for na::Vector2<f32> {
    fn to_glam(self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

// pub trait PhysicToGodotTransform {
//     /// Convert to a godot's `Transform2D` with `GAME_TO_GODOT_RATIO` scale applied.
//     fn to_godot_transform_scaled(self) -> Transform2D;
// }
// impl PhysicToGodotTransform for na::Isometry2<f32> {
//     fn to_godot_transform_scaled(self) -> Transform2D {
//         let cos = self.rotation.cos_angle() * GAME_TO_GODOT_RATIO;
//         let sin = self.rotation.sin_angle() * GAME_TO_GODOT_RATIO;

//         Transform2D {
//             a: Vector2::new(cos, sin),
//             b: Vector2::new(-sin, cos),
//             origin: Vector2::new(self.translation.x, self.translation.y) * GAME_TO_GODOT_RATIO,
//         }
//     }
// }
