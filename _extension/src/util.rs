use super::*;
use godot::prelude::*;

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

pub trait AproxZero {
    fn aprox_zero(self) -> bool;
}
impl AproxZero for f32 {
    fn aprox_zero(self) -> bool {
        ComplexField::abs(self) < 0.001
    }
}
impl AproxZero for glam::Vec2 {
    fn aprox_zero(self) -> bool {
        self.x.abs() + self.y.abs() < 0.001
    }
}

pub fn add_child<A, B>(parent: &Gd<A>, child: &Gd<B>)
where
    A: Inherits<Node> + godot::prelude::GodotClass,
    B: Inherits<Node> + godot::prelude::GodotClass,
{
    parent.share().upcast().add_child(
        child.share().upcast(),
        false,
        godot::engine::node::InternalMode::INTERNAL_MODE_DISABLED,
    );
}

pub fn add_child_node<B>(parent: &mut Gd<Node>, child: &Gd<B>)
where
    B: Inherits<Node> + godot::prelude::GodotClass,
{
    parent.add_child(
        child.share().upcast(),
        false,
        godot::engine::node::InternalMode::INTERNAL_MODE_DISABLED,
    );
}

pub fn add_child_node_node(parent: &mut Gd<Node>, child: Gd<Node>) {
    parent.add_child(
        child,
        false,
        godot::engine::node::InternalMode::INTERNAL_MODE_DISABLED,
    );
}

pub trait ToNalgebra {
    fn to_na(self) -> na::Vector2<f32>;

    /// Convert to a nalgebra vector with `GODOT_SCALE` scale removed.
    fn to_na_descaled(self) -> na::Vector2<f32>
    where
        Self: Sized,
    {
        self.to_na() / GODOT_SCALE
    }
}
impl ToNalgebra for Vector2 {
    fn to_na(self) -> na::Vector2<f32> {
        na::Vector2::new(self.x, self.y)
    }
}
impl ToNalgebra for glam::Vec2 {
    fn to_na(self) -> na::Vector2<f32> {
        na::Vector2::new(self.x, self.y)
    }
}

pub trait ToGlam {
    fn to_glam(self) -> glam::Vec2;

    /// Convert to a glam vector with `GODOT_SCALE` scale removed.
    fn to_glam_descaled(self) -> glam::Vec2
    where
        Self: Sized,
    {
        self.to_glam() / GODOT_SCALE
    }
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
        glam::vec2(self.x, self.y)
    }
}

pub trait ToGodot {
    fn to_godot(self) -> Vector2;

    /// Convert to a godot vector with `GODOT_SCALE` scale added.
    fn to_godot_scaled(self) -> Vector2;
}
impl ToGodot for na::Vector2<f32> {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(self.x * GODOT_SCALE, self.y * GODOT_SCALE)
    }
}
impl ToGodot for na::Translation2<f32> {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(self.x * GODOT_SCALE, self.y * GODOT_SCALE)
    }
}
impl ToGodot for glam::Vec2 {
    fn to_godot(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    fn to_godot_scaled(self) -> Vector2 {
        Vector2::new(self.x * GODOT_SCALE, self.y * GODOT_SCALE)
    }
}
