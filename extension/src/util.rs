use super::*;
use godot::prelude::*;

/// 1 battlescape unit == 32 godot unit
const BATTLESCAPE_TO_GODOT_SCALE: f32 = 32.0;

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
        t.mul_add(delta, self + TAU) % TAU
    }
}

pub trait ToNalgebra {
    fn to_na(self) -> na::Vector2<f32>;

    /// Convert to a nalgebra vector with `BATTLESCAPE_TO_GODOT_SCALE` scale removed.
    fn to_na_descaled(self) -> na::Vector2<f32>
    where
        Self: Sized,
    {
        self.to_na() / BATTLESCAPE_TO_GODOT_SCALE
    }
}
impl ToNalgebra for Vector2 {
    fn to_na(self) -> na::Vector2<f32> {
        na::Vector2::new(self.inner().x, self.inner().y)
    }
}
// impl ToNalgebra for glam::Vec2 {
//     fn to_na(self) -> na::Vector2<f32> {
//         na::vector![self.x, self.y]
//     }
// }

pub trait ToGlam {
    fn to_glam(self) -> glam::Vec2;
}
impl ToGlam for na::Translation2<f32> {
    fn to_glam(self) -> glam::Vec2 {
        glam::vec2(self.x, self.y)
    }
}
impl ToGlam for na::Vector2<f32> {
    fn to_glam(self) -> glam::Vec2 {
        glam::vec2(self.x, self.y)
    }
}
impl ToGlam for Vector2 {
    fn to_glam(self) -> glam::Vec2 {
        self.inner()
        // glam::vec2(self.x, self.y)
    }
}

pub trait ToGodot {
    fn to_godot(self) -> Vector2;

    /// Convert to a godot vector with `BATTLESCAPE_TO_GODOT_SCALE` scale added.
    fn to_godot_scaled(self) -> Vector2;
}
impl ToGodot for na::Vector2<f32> {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(
            self.x * BATTLESCAPE_TO_GODOT_SCALE,
            self.y * BATTLESCAPE_TO_GODOT_SCALE,
        )
    }
}
impl ToGodot for na::Translation2<f32> {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(
            self.x * BATTLESCAPE_TO_GODOT_SCALE,
            self.y * BATTLESCAPE_TO_GODOT_SCALE,
        )
    }
}
impl ToGodot for glam::Vec2 {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(
            self.x * BATTLESCAPE_TO_GODOT_SCALE,
            self.y * BATTLESCAPE_TO_GODOT_SCALE,
        )
    }
}
