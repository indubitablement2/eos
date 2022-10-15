pub trait BoundingShape {
    /// By convention, down is positive.
    fn bot(&self) -> f32;
    /// By convention, up is negative.
    fn top(&self) -> f32;
    fn left(&self) -> f32;
    fn right(&self) -> f32;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn intersect(&self, other: &Self) -> bool;
    /// We already know that `other.bot >= self.top`
    /// Allow some optimization with some shape and acceleration structure.
    fn intersect_fast(&self, other: &Self) -> bool;
    fn intersect_point(&self, x: f32, y: f32) -> bool;
    fn from_point(x:f32, y: f32) -> Self;
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct CircleBoundingShape {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}
impl BoundingShape for CircleBoundingShape {
    fn bot(&self) -> f32 {
        self.y + self.r
    }

    fn top(&self) -> f32 {
        self.y - self.r
    }

    fn left(&self) -> f32 {
        self.x - self.r
    }

    fn right(&self) -> f32 {
        self.x + self.r
    }

    fn width(&self) -> f32 {
        self.r + self.r
    }

    fn height(&self) -> f32 {
        self.width()
    }

    fn intersect(&self, other: &Self) -> bool {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let r = self.r + other.r;
        x * x + y * y <= r * r
    }

    fn intersect_fast(&self, other: &Self) -> bool {
        self.intersect(other)
    }

    fn intersect_point(&self, x: f32, y: f32) -> bool {
        let x = self.x - x;
        let y = self.y - y;
        x * x + y * y <= self.r
    }

    fn from_point(x:f32, y: f32) -> Self {
        Self {
            x,
            y,
            r: 0.0,
        }
    }
}
impl rand::distributions::Distribution<CircleBoundingShape> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> CircleBoundingShape {
        CircleBoundingShape {
            x: rng.gen(),
            y: rng.gen(),
            r: rng.gen(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct AABB {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bot: f32,
}
impl BoundingShape for AABB {
    fn bot(&self) -> f32 {
        self.bot
    }

    fn top(&self) -> f32 {
        self.top
    }

    fn left(&self) -> f32 {
        self.left
    }

    fn right(&self) -> f32 {
        self.right
    }

    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn height(&self) -> f32 {
        self.bot - self.top
    }

    fn intersect(&self, other: &Self) -> bool {
        other.left <= self.right
            && other.bot >= self.top
            && other.top <= self.bot
            && other.right >= self.left
    }

    fn intersect_fast(&self, other: &Self) -> bool {
        other.left <= self.right && other.top <= self.bot && other.right >= self.left
    }

    fn intersect_point(&self, x: f32, y: f32) -> bool {
        x <= self.right
            && y >= self.top
            && y <= self.bot
            && x >= self.left
    }

    fn from_point(x:f32, y: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x,
            bot: y,
        }
    }
}
impl rand::distributions::Distribution<AABB> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> AABB {
        let left = rng.gen::<f32>();
        let top = rng.gen::<f32>();

        AABB {
            left,
            top,
            right: left + rng.gen::<f32>(),
            bot: top + rng.gen::<f32>(),
        }
    }
}
