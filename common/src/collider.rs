use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub radius: f32,
    pub position: Vec2,
}
impl Collider {
    pub fn top(self) -> f32 {
        self.position.y - self.radius
    }

    pub fn bot(self) -> f32 {
        self.position.y + self.radius
    }

    pub fn right(self) -> f32 {
        self.position.x + self.radius
    }

    pub fn left(self) -> f32 {
        self.position.x - self.radius
    }

    /// Return true if these Colliders intersect.
    pub fn intersection_test(self, other: Collider) -> bool {
        self.position.distance_squared(other.position) <= (self.radius + other.radius).powi(2)
    }

    pub fn intersection_test_point(self, point: Vec2) -> bool {
        self.position.distance_squared(point) <= self.radius.powi(2)
    }
}
