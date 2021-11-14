use glam::{uvec2, vec2, UVec2, Vec2};
use indexmap::IndexMap;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MembershipId {
    pub id: u32,
}

/// Higher bits are used as mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColliderId {
    pub id: u32,
}
impl ColliderId {
    pub const DYNAMIC_MASK: u32 = 0b10000000000000000000000000000000;
    pub const AABB_MASK: u32 = 0b00100000000000000000000000000000;
    pub const CIRCLE_MASK: u32 = 0b01000000000000000000000000000000;
    pub const LINE_MASK: u32 = 0b01100000000000000000000000000000;
    pub const ID_MASK: u32 = 0b00011111111111111111111111111111;

    pub fn get_collider_mask(collider: &Collider) -> u32 {
        let mut mask = match collider.dynamic {
            true => Self::DYNAMIC_MASK,
            false => 0,
        };

        mask |= match collider.shape {
            Shape::AABB(_) => Self::AABB_MASK,
            Shape::Circle(_) => Self::CIRCLE_MASK,
            Shape::Line(_) => Self::LINE_MASK,
        };

        mask
    }
}

struct ColliderIdDispenser {
    last: u32,
    available: Vec<u32>,
}
impl ColliderIdDispenser {
    pub fn new() -> Self {
        Self {
            last: 0,
            available: Vec::new(),
        }
    }

    pub fn new_collider_id(&mut self, mask: u32) -> ColliderId {
        match self.available.pop() {
            Some(available) => ColliderId { id: available + mask },
            None => {
                self.last += 1;
                ColliderId { id: self.last + mask }
            }
        }
    }

    pub fn recycle_collider_id(&mut self, collider_id: ColliderId) {
        self.available.push(collider_id.id | ColliderId::ID_MASK);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    extent: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    end: Vec2,
}

/// Comparaison between shapes only check discriminant ignoring actual data.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    AABB(AABB),
    Circle(Circle),
    Line(Line),
}

pub struct Collider {
    pub shape: Shape,
    pub position: Vec2,
    pub membership: MembershipId,
    /// This object is expected to move frequenly?
    pub dynamic: bool,
}

pub struct Grid {
    /// The botom right corner of the grid.
    maxs: Vec2,
    width: u32,
    height: u32,
    /// UVec2(width -1, height - 1). Anything more is outside of the grid.
    ucartesian_max: UVec2,
    collider_id_dispenser: ColliderIdDispenser,
    colliders: IndexMap<ColliderId, Collider>,
    data: Vec<Vec<ColliderId>>,
    /// Static collider don't need to be updated unless one war modified.
    full_update_required: bool,
}
impl Grid {
    pub const CELL_SIZE: f32 = 32.0;

    pub fn ucartesian_from_id(&self, id: u32) -> UVec2 {
        uvec2(id % self.width, id / self.width).min(self.ucartesian_max)
    }

    pub fn ucartesian_from_position(&self, position: Vec2) -> UVec2 {
        ((position + self.maxs) * 0.5 / Self::CELL_SIZE)
            .as_uvec2()
            .min(self.ucartesian_max)
    }

    pub fn id_from_ucartesian(&self, mut ucartesian: UVec2) -> u32 {
        ucartesian = ucartesian.min(self.ucartesian_max);
        ucartesian.y * self.width + ucartesian.x
    }

    pub fn id_from_position(&self, position: Vec2) -> u32 {
        let ucartesian = self.ucartesian_from_position(position);
        ucartesian.y * self.width + ucartesian.x
    }

    /// The returned position is the top left corner of the cell.
    pub fn position_from_ucartesian(&self, ucartesian: UVec2) -> Vec2 {
        ucartesian.as_vec2() * Grid::CELL_SIZE - self.maxs
    }

    pub fn position_from_id(&self, id: u32) -> Vec2 {
        let ucartesian = uvec2(id % self.width, id / self.width);
        self.position_from_ucartesian(ucartesian)
    }

    /// Will not take effect until grid is updated.
    pub fn remove_collider(&mut self, collider_id: ColliderId) -> Option<Collider> {
        match self.colliders.remove(&collider_id) {
            Some(collider) => {
                self.collider_id_dispenser.recycle_collider_id(collider_id);
                self.full_update_required &= collider.dynamic;
                Some(collider)
            }
            None => None,
        }
    }

    /// Will not take effect until grid is updated.
    pub fn add_collider(&mut self, collider: Collider) -> ColliderId {
        self.full_update_required &= collider.dynamic;
        let collider_id = self
            .collider_id_dispenser
            .new_collider_id(ColliderId::get_collider_mask(&collider));
        self.colliders.insert(collider_id, collider);
        collider_id
    }

    /// Return an iterator over all overlapping cell id for this AABB.
    pub fn overlapping_cell_aabb(&self, aabb: AABB, position: Vec2) -> impl Iterator<Item = usize> + '_ {
        let start = self.ucartesian_from_position(position);
        let end = self.ucartesian_from_position(position + aabb.extent);

        (start.y as usize..end.y as usize)
            .flat_map(move |y| (start.x as usize..end.x as usize).map(move |x| y * self.width as usize + x))
    }

    /// TODO: Use binary search instead of linear.
    /// Get the index of the first dynamic collider.
    fn first_dynamic_collider(&self) -> usize {
        for (i, collider) in self.colliders.values().enumerate() {
            if collider.dynamic {
                return i;
            }
        }
        0
    }

    pub fn update(&mut self) {
        if self.full_update_required {
            self.update_full();
        } else {
            self.update_partial();
        }
        self.full_update_required = false;
    }

    fn update_partial(&mut self) {
        // Clear dynamic colliders.
        for cell in &mut self.data {
            cell.truncate(cell.partition_point(|x| x.id < ColliderId::DYNAMIC_MASK).saturating_sub(1));
        }

        // Add dynamic colliders.
        for (collider_id, collider) in self.colliders.iter().skip(self.first_dynamic_collider()) {
            match collider.shape {
                Shape::AABB(aabb) => {
                    let start = self.ucartesian_from_position(collider.position);
                    let end = self.ucartesian_from_position(collider.position + aabb.extent);

                    for y in start.y as usize..end.y as usize {
                        for x in start.x as usize..end.x as usize {
                            let i = y * self.width as usize + x;
                            self.data[i].push(*collider_id);
                        }
                    }
                }
                Shape::Circle(circle) => {
                    let start = self.ucartesian_from_position(collider.position - circle.radius);
                    let end = self.ucartesian_from_position(collider.position + circle.radius);

                    for y in start.y as usize..end.y as usize {
                        for x in start.x as usize..end.x as usize {
                            let i = y * self.width as usize + x;
                            self.data[i].push(*collider_id);
                        }
                    }
                }
                Shape::Line(line) => {}
            }
        }
    }

    fn update_full(&mut self) {
        // Clear all colliders.
        for cell in &mut self.data {
            cell.clear();
        }

        // Sort colliders so statics ones are first then sort by shape type.
        self.colliders.sort_keys();

        // Add all colliders.
        for (collider_id, collider) in &self.colliders {
            match collider.shape {
                Shape::AABB(aabb) => {
                    let start = self.ucartesian_from_position(collider.position);
                    let end = self.ucartesian_from_position(collider.position + aabb.extent);

                    for y in start.y as usize..end.y as usize {
                        for x in start.x as usize..end.x as usize {
                            let i = y * self.width as usize + x;
                            self.data[i].push(*collider_id);
                        }
                    }
                }
                Shape::Circle(circle) => {
                    let start = self.ucartesian_from_position(collider.position - circle.radius);
                    let end = self.ucartesian_from_position(collider.position + circle.radius);

                    for y in start.y as usize..end.y as usize {
                        for x in start.x as usize..end.x as usize {
                            let i = y * self.width as usize + x;
                            self.data[i].push(*collider_id);
                        }
                    }
                }
                Shape::Line(line) => {}
            }
        }
    }
}
