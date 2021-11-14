use glam::Vec2;
use indexmap::IndexMap;
use num_enum::{FromPrimitive, IntoPrimitive};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::{
    cmp::Ordering,
    fmt::Debug,
    ops::{Index, IndexMut},
};

/// Higher bits are used as mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColliderId {
    id: u32,
}
impl ColliderId {
    pub const MEMBERSHIP_LEADING_ZEROS: u32 = (Membership::MAX as u32).leading_zeros();
    pub const MEMBERSHIP_MASK: u32 = u32::MAX << Self::MEMBERSHIP_LEADING_ZEROS;
    pub const ID_MASK: u32 = !Self::MEMBERSHIP_MASK;
}
impl From<Membership> for ColliderId {
    fn from(membership: Membership) -> Self {
        Self {
            id: u32::from(membership) << Self::MEMBERSHIP_LEADING_ZEROS,
        }
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

    pub fn new_collider_id(&mut self, membership: Membership) -> ColliderId {
        let mut collider_id = ColliderId::from(membership);

        match self.available.pop() {
            Some(available) => {
                collider_id.id += available;
            }
            None => {
                self.last += 1;
                collider_id.id += self.last;
            }
        }

        collider_id
    }

    pub fn recycle_collider_id(&mut self, collider_id: ColliderId) {
        self.available.push(collider_id.id & ColliderId::ID_MASK);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, FromPrimitive)]
#[repr(u32)]
pub enum Membership {
    #[num_enum(default)]
    FleetDetection,
    FleetDetector,
    System,
}
impl Membership {
    const MAX: usize = 3;

    pub fn get_min_row_size(&self) -> f32 {
        match self {
            Membership::FleetDetection => 32.0,
            Membership::FleetDetector => 32.0,
            Membership::System => 128.0,
        }
    }
}
impl From<ColliderId> for Membership {
    fn from(collider_id: ColliderId) -> Self {
        Self::from(collider_id.id >> ColliderId::MEMBERSHIP_LEADING_ZEROS)
    }
}
impl Index<Membership> for [AccelerationStructure; Membership::MAX] {
    type Output = AccelerationStructure;

    fn index(&self, index: Membership) -> &Self::Output {
        &self[index as usize]
    }
}
impl IndexMut<Membership> for [AccelerationStructure; Membership::MAX] {
    fn index_mut(&mut self, index: Membership) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub radius: f32,
    pub position: Vec2,
}
impl Collider {
    pub fn intersection_test(self, other: Collider) -> bool {
        self.position.distance_squared(other.position) <= (self.radius + other.radius).powi(2)
    }
}
impl Default for Collider {
    fn default() -> Self {
        Self {
            radius: Default::default(),
            position: Default::default(),
        }
    }
}

struct SAPRow {
    /// y position where this row ends and the next one (if there is one) ends.
    pub end: f32,
    /// y position where this row start and the previous one (if there is one) ends.
    pub start: f32,
    /// The biggest radius found in this row.
    pub biggest_radius: f32,
    /// An indice on the colliders IndexMap.
    pub data: Vec<u32>,
}
impl Default for SAPRow {
    fn default() -> Self {
        Self {
            end: 0.0,
            start: 0.0,
            biggest_radius: 0.0,
            data: Vec::with_capacity(AccelerationStructure::MIN_COLLIDER_PER_ROW),
        }
    }
}

struct AccelerationStructure {
    /// Sorted on the y axis.
    pub colliders: IndexMap<ColliderId, Collider>,
    /// The difference between each row's start and end can not be smaller than this.
    pub min_row_size: f32,
    /// Sorted on the x axis.
    pub rows: Vec<SAPRow>,
}
impl AccelerationStructure {
    const MIN_COLLIDER_PER_ROW: usize = 16;

    pub fn new(min_row_size: f32) -> Self {
        Self {
            colliders: IndexMap::new(),
            min_row_size,
            rows: Vec::new(),
        }
    }

    fn update(&mut self) {
        if self.colliders.is_empty() {
            return;
        }

        // Sort on y axis.
        self.colliders
            .sort_by(|_k1, v1, _k2, v2| v1.radius.partial_cmp(&v2.radius).unwrap_or(Ordering::Equal));

        // Recycle old rows.
        let num_old_row = self.rows.len();
        let mut old_row = std::mem::replace(&mut self.rows, Vec::with_capacity(num_old_row + 8));
        for row in &mut old_row {
            row.data.clear();
        }

        // Prepare first row.
        let mut current_row = old_row.pop().unwrap_or_default();
        current_row.start = match self.colliders.first() {
            Some((_, collider)) => collider.position.y,
            None => 0.0,
        };
        // Add colliders to rows.
        for (i, collider) in self.colliders.values().enumerate() {
            // Add this collider to current row.
            current_row.data.push(i as u32);
            current_row.end = collider.position.y;

            if current_row.data.len() > Self::MIN_COLLIDER_PER_ROW && current_row.end - current_row.start > self.min_row_size {
                // This row is full.
                self.rows.push(current_row);
                // Create a new row.
                current_row = old_row.pop().unwrap_or_default();
                current_row.start = collider.position.y;
            }
        }
        if current_row.data.len() > 1 {
            self.rows.push(current_row);
        }

        // Add colliders overlapping multiple rows.
        unsafe {
            for i in 1..self.rows.len() {
                let (p, c) = self.rows.split_at_mut_unchecked(i);
                let current = c.first_mut().unwrap_unchecked();
                let previous = p.last_mut().unwrap_unchecked();

                for i in &previous.data {
                    let collider = &self.colliders[*i as usize];
                    if collider.position.y + collider.radius >= current.start {
                        // This collider from the previous row overlap the current row.
                        current.data.push(*i);
                    }
                }
            }
        }

        // Sort each row.
        for row in &mut self.rows {
            row.data.sort_unstable_by(|a, b| {
                self.colliders[*a as usize]
                    .position
                    .x
                    .partial_cmp(&self.colliders[*b as usize].position.x)
                    .unwrap_or(Ordering::Equal)
            });
        }

        // Find biggest radius for each row.
        for row in &mut self.rows {
            row.biggest_radius = row
                .data
                .iter()
                .fold(0.0, |acc, i| self.colliders[*i as usize].radius.max(acc));
        }
    }
}

/// TODO: Add intersection events collector.
pub struct IntersectionPipeline {
    collider_id_dispenser: ColliderIdDispenser,
    memberships: [AccelerationStructure; Membership::MAX],
}
impl IntersectionPipeline {
    pub fn new() -> Self {
        Self {
            collider_id_dispenser: ColliderIdDispenser::new(),
            memberships: [
                AccelerationStructure::new(Membership::FleetDetection.get_min_row_size()),
                AccelerationStructure::new(Membership::FleetDetector.get_min_row_size()),
                AccelerationStructure::new(Membership::System.get_min_row_size()),
            ],
        }
    }

    pub fn insert_collider(&mut self, collider: Collider, membership: Membership) -> ColliderId {
        let collider_id = self.collider_id_dispenser.new_collider_id(membership);

        self.memberships[membership].colliders.insert(collider_id, collider);

        collider_id
    }

    pub fn remove_collider(&mut self, collider_id: ColliderId) -> Option<Collider> {
        let maybe_collider = self.memberships[Membership::from(collider_id)].colliders.remove(&collider_id);

        if maybe_collider.is_some() {
            self.collider_id_dispenser.recycle_collider_id(collider_id);
        }

        maybe_collider
    }

    /// Get a copy of a Collider.
    pub fn get_collider(&self, collider_id: ColliderId) -> Option<Collider> {
        if let Some(collider) = self.memberships[Membership::from(collider_id)].colliders.get(&collider_id) {
            Some(*collider)
        } else {
            None
        }
    }

    /// Get a mutable reference to a Collider.
    pub fn get_collider_mut(&mut self, collider_id: ColliderId) -> Option<&mut Collider> {
        self.memberships[Membership::from(collider_id)]
            .colliders
            .get_mut(&collider_id)
    }

    /// Get a copy of every colliders.
    pub fn get_colliders_copy(&self) -> Vec<Collider> {
        let num_collider = self
            .memberships
            .iter()
            .fold(0usize, |acc, acceleration_struct| acc + acceleration_struct.colliders.len());

        let mut v = Vec::with_capacity(num_collider);

        for acceleration_struct in &self.memberships {
            for collider in acceleration_struct.colliders.values() {
                v.push(*collider);
            }
        }

        v
    }

    /// TODO: Implementation on the AccelerationStructure side.
    pub fn intersect_collider(&self, collider: Collider, filter: Membership) -> bool {
        let acceleration_struct = &self.memberships[filter];

        for row in &acceleration_struct.rows {
            // Check if Collider overlap this row.
            if collider.position.y + collider.radius < row.start || collider.position.y - collider.radius > row.end {
                continue;
            }

            // Test with Collider in this row.
            for i in &row.data {
                if collider.intersection_test(acceleration_struct.colliders[*i as usize]) {
                    return true;
                }
            }
        }

        false
    }

    pub fn update(&mut self) {
        self.memberships.par_iter_mut().for_each(|acceleration_structure| {
            acceleration_structure.update();
        });
    }
}
