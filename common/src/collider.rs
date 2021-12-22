use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    /// Set by the user.
    pub id: u32,
    pub radius: f32,
    pub position: Vec2,
}
impl Collider {
    pub fn new(id: u32, radius: f32, position: Vec2) -> Self {
        Self { id, radius, position }
    }

    /// Create a new collider with an id of 0.
    pub fn new_idless(radius: f32, position: Vec2) -> Self {
        Self {
            id: 0,
            radius,
            position,
        }
    }

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

    /// Return true if these colliders intersect.
    pub fn intersection_test(self, other: Collider) -> bool {
        self.position.distance_squared(other.position) <= (self.radius + other.radius).powi(2)
    }

    /// Return true if this collider intersect a point.
    pub fn intersection_test_point(self, point: Vec2) -> bool {
        self.position.distance_squared(point) <= self.radius.powi(2)
    }

    /// Return half of the biggest horizontal lenght within two (possibly) intersecting horizontal lines.
    /// 
    /// This will return the collider radius if the collider's vertical position is within these lines.
    /// Both lines needs to be either above or bellow the coillider's vertical position for it not to.
    /// 
    /// This is often used as a threshold for when we should stop looking for new possible colliders
    /// to test in the intersection engine. 
    pub fn biggest_slice_within_row(self, top: f32, bot: f32) -> f32 {
        if self.position.y > top && self.position.y < bot {
            self.radius
        } else {
            // The distance to the top or bot. Whichever is closest.
            let distance = (self.position.y - top)
                .abs()
                .min((self.position.y - bot).abs());
            // This is used instead of the collider's radius as it is smaller.
            (self.radius.powi(2) - distance.powi(2)).sqrt()
        }
    }
}
