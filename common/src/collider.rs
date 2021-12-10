use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub radius: f32,
    pub position: Vec2,
}
impl Collider {
    /// Return true if these Colliders intersect.
    pub fn intersection_test(self, other: Collider) -> bool {
        self.position.distance_squared(other.position) <= (self.radius + other.radius).powi(2)
    }

    pub fn intersection_test_point(self, point: Vec2) -> bool {
        self.position.distance_squared(point) <= self.radius.powi(2)
    }
}
