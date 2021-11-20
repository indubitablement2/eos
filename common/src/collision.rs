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

    /// A ColliderId that is always invalid.
    pub fn new_invalid() -> Self {
        Self { id: 0 }
    }

    /// Return if this ColliderId is valid.
    pub fn is_valid(&self) -> bool {
        self.id & Self::ID_MASK != 0
    }
}
impl From<Membership> for ColliderId {
    fn from(membership: Membership) -> Self {
        Self {
            id: u32::from(membership) << Self::MEMBERSHIP_LEADING_ZEROS,
        }
    }
}

#[derive(Debug)]
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
    Fleet,
    System,
    RealityBubble,
    FactionActivity,
}
impl Membership {
    const MAX: usize = 4;

    pub fn get_min_row_size(&self) -> f32 {
        match self {
            Membership::Fleet => crate::ecs_components::FleetCollider::RADIUS_MAX * 3.0,
            Membership::System => crate::ecs_components::SystemCollider::RADIUS_MAX * 3.0,
            Membership::RealityBubble => crate::ecs_components::RealityBubbleCollider::RADIUS * 3.0,
            Membership::FactionActivity => crate::ecs_components::FactionActivityCollider::RADIUS_MAX * 3.0,
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
    pub custom_data: u32,
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

#[derive(Debug)]
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

#[derive(Debug)]
struct AccelerationStructure {
    /// Sorted on the y axis.
    pub colliders: IndexMap<ColliderId, Collider>,
    /// The difference between each row's start and end can not be smaller than this.
    pub min_row_size: f32,
    /// Sorted on the x axis.
    pub rows: Vec<SAPRow>,
}
impl AccelerationStructure {
    const MIN_COLLIDER_PER_ROW: usize = 5;

    pub fn new(min_row_size: f32) -> Self {
        Self {
            colliders: IndexMap::new(),
            min_row_size,
            rows: Vec::new(),
        }
    }

    fn update(&mut self) {
        // Recycle old rows.
        let num_old_row = self.rows.len();
        let mut old_row = std::mem::replace(&mut self.rows, Vec::with_capacity(num_old_row + 8));
        for row in &mut old_row {
            row.data.clear();
        }

        if self.colliders.is_empty() {
            return;
        }

        // Sort on y axis.
        self.colliders
            .sort_by(|_, v1, _, v2| v1.position.y.partial_cmp(&v2.position.y).unwrap_or(Ordering::Equal));

        // Prepare first row.
        let mut current_row = old_row.pop().unwrap_or_default();
        // First row's start should be very large negative number.
        current_row.start = -10000000000.0;
        let mut num_in_current_row = 0usize;
        // Create rows.
        for collider in self.colliders.values() {
            num_in_current_row += 1;
            current_row.end = collider.position.y;
            if num_in_current_row >= Self::MIN_COLLIDER_PER_ROW {
                // We have the minimum number of collider to make a row.
                if current_row.end - current_row.start >= self.min_row_size {
                    // We also have the minimun size.
                    self.rows.push(current_row);

                    // Prepare next row.
                    current_row = old_row.pop().unwrap_or_default();
                    current_row.start = collider.position.y;
                    num_in_current_row = 0;
                }
            }
        }
        // Add non-full row.
        if num_in_current_row > 0 {
            self.rows.push(current_row);
        }
        // Last row's end should be very large.
        self.rows.last_mut().unwrap().end = 10000000000.0;

        // Add colliders to overlapping rows.
        let mut i = 0u32;
        for collider in self.colliders.values() {
            let bottom = collider.position.y - collider.radius;
            let top = collider.position.y + collider.radius;
            let first_overlapping_row = self.rows.partition_point(|row| row.end < bottom);
            for row in &mut self.rows[first_overlapping_row..] {
                if row.start > top {
                    break;
                }
                row.data.push(i);
            }

            i += 1;
        }

        // Sort each row on the x axis.
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

    pub fn intersect_collider(&self, collider: Collider) -> bool {
        let mut to_test = Vec::with_capacity(16);
        let bottom = collider.position.y - collider.radius;
        let top = collider.position.y + collider.radius;
        let first_overlapping_row = self.rows.partition_point(|row| row.end < bottom);
        for row in &self.rows[first_overlapping_row..] {
            if row.start > top {
                break;
            }
            // The collider overlap this row.

            let closest = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < collider.position.x);

            // The furthest we should look in each dirrections.
            let threshold = collider.radius + row.biggest_radius;

            // Look to the left.
            let mut left = closest.saturating_sub(1);
            while let Some(i) = row.data.get(left) {
                let other = self.colliders[*i as usize];
                if collider.position.x - other.position.x > threshold {
                    break;
                }
                to_test.push(*i);

                if left == 0 {
                    break;
                } else {
                    left -= 1;
                }
            }
            // Look to the right.
            let mut right = closest;
            while let Some(i) = row.data.get(right) {
                let other = self.colliders[*i as usize];
                if other.position.x - collider.position.x > threshold {
                    break;
                }
                to_test.push(*i);

                right += 1;
            }
        }

        // Remove duplicate.
        to_test.sort_unstable();
        to_test.dedup();

        // Test each Collider we have collected.
        for i in to_test.into_iter() {
            if collider.intersection_test(self.colliders[i as usize]) {
                return true;
            }
        }

        false
    }

    pub fn intersect_point(&self, point: Vec2) -> bool {
        let mut to_test = Vec::with_capacity(16);
        let overlapping_row = self.rows.partition_point(|row| row.end < point.y);
        if let Some(row) = self.rows.get(overlapping_row) {
            // The closest collider to this point.
            let closest = row
            .data
            .partition_point(|i| self.colliders[*i as usize].position.x < point.x);

            // The furthest we should look in each dirrections.
            let threshold = row.biggest_radius;

            // Look to the left.
            let mut left = closest.saturating_sub(1);
            while let Some(i) = row.data.get(left) {
                let other = self.colliders[*i as usize];
                if point.x - other.position.x > threshold {
                    break;
                }
                to_test.push(*i);

                if left == 0 {
                    break;
                } else {
                    left -= 1;
                }
            }
            // Look to the right.
            let mut right = closest;
            while let Some(i) = row.data.get(right) {
                let other = self.colliders[*i as usize];
                if other.position.x - point.x > threshold {
                    break;
                }
                to_test.push(*i);

                right += 1;
            }
        }

        // Remove duplicate.
        to_test.sort_unstable();
        to_test.dedup();

        // Test each Collider we have collected.
        for i in to_test.into_iter() {
            if self.colliders[i as usize].intersection_test_point(point) {
                return true;
            }
        }

        false
    }

    /// Get the line between each row.
    pub fn get_rows_separation(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(self.rows.len() + 1);

        self.rows.iter().for_each(|row| {
            v.push(row.start);
        });

        if let Some(last) = self.rows.last() {
            v.push(last.end);
        }

        v
    }
}

/// Allow fast circle-circle intersection test.
#[derive(Debug)]
pub struct IntersectionPipeline {
    collider_id_dispenser: ColliderIdDispenser,
    memberships: [AccelerationStructure; Membership::MAX],
    remove_queue: Vec<ColliderId>,
}
impl IntersectionPipeline {
    pub fn new() -> Self {
        Self {
            collider_id_dispenser: ColliderIdDispenser::new(),
            memberships: [
                AccelerationStructure::new(Membership::Fleet.get_min_row_size()),
                AccelerationStructure::new(Membership::System.get_min_row_size()),
                AccelerationStructure::new(Membership::RealityBubble.get_min_row_size()),
                AccelerationStructure::new(Membership::FactionActivity.get_min_row_size()),
            ],
            remove_queue: Vec::new(),
        }
    }

    pub fn insert_collider(&mut self, collider: Collider, membership: Membership) -> ColliderId {
        let collider_id = self.collider_id_dispenser.new_collider_id(membership);

        self.memberships[membership].colliders.insert(collider_id, collider);

        collider_id
    }

    pub fn remove_collider(&mut self, collider_id: ColliderId) {
        self.remove_queue.push(collider_id);
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

    /// Get a copy of every colliders separated by Membership.
    pub fn get_colliders_copy(&self) -> Vec<Vec<Collider>> {

        self.memberships.iter().map(|acc| {
            acc.colliders.values().copied().collect::<Vec<Collider>>()
        }).collect()
    }

    /// Get a mutable reference to every collider of thos membership.
    pub fn get_colliders_mut(&mut self, membership: Membership) -> indexmap::map::ValuesMut<'_, ColliderId, Collider> {
        self.memberships[membership].colliders.values_mut()
    }

    /// Get every ColliderId in this membership.
    pub fn get_collider_id(&self, membership: Membership) -> indexmap::map::Keys<'_, ColliderId, Collider> {
        self.memberships[membership].colliders.keys()
    }

    /// Test for an intersection with the provided Collider.
    pub fn test_collider(&self, collider: Collider, filter: Membership) -> bool {
        self.memberships[filter].intersect_collider(collider)
    }

    pub fn test_point(&self, point: Vec2, filter: Membership) -> bool {
        self.memberships[filter].intersect_point(point)
    }

    pub fn test_collider_brute(&self, collider: Collider, filter: Membership) -> bool {
        for other in self.memberships[filter].colliders.values() {
            if collider.intersection_test(*other) {
                return true;
            }
        }
        false
    }

    /// Get the separation line between each row. Useful for debug.
    pub fn get_rows_separation(&self, filter: Membership) -> Vec<f32> {
        self.memberships[filter].get_rows_separation()
    }

    pub fn update(&mut self) {
        // Remove queued collider.
        for collider_id in self.remove_queue.drain(..) {
            if self.memberships[Membership::from(collider_id)]
                .colliders
                .remove(&collider_id)
                .is_some()
            {
                self.collider_id_dispenser.recycle_collider_id(collider_id);
            }
        }

        // Update each membership in parrallele.
        self.memberships.par_iter_mut().for_each(|acceleration_structure| {
            acceleration_structure.update();
        });
    }
}

#[test]
fn test_basic() {
    use glam::vec2;

    let mut intersection_pipeline = IntersectionPipeline::new();

    // Empty.
    assert!(!intersection_pipeline.test_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
            custom_data: 0
        },
        Membership::Fleet
    ));

    // Basic test.
    let first_collider_id = intersection_pipeline.insert_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
            custom_data: 0,
        },
        Membership::Fleet,
    );
    intersection_pipeline.update();
    println!("{:?}", &intersection_pipeline.memberships[Membership::Fleet]);
    assert!(intersection_pipeline.test_collider(
        Collider {
            radius: 10.0,
            position: vec2(-4.0, 0.0),
            custom_data: 0
        },
        Membership::Fleet
    ));
    assert!(intersection_pipeline.test_collider(
        Collider {
            radius: 10.0,
            position: vec2(4.0, 0.0),
            custom_data: 0
        },
        Membership::Fleet
    ));

    // Removing collider.
    intersection_pipeline.remove_collider(first_collider_id);
    intersection_pipeline.update();

    for row in &intersection_pipeline.memberships[Membership::Fleet].rows {
        assert!(row.data.is_empty(), "should be empty");
    }

    println!("\n{:?}", &intersection_pipeline.memberships[Membership::Fleet]);
    assert!(!intersection_pipeline.test_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
            custom_data: 0
        },
        Membership::Fleet
    ));
}

#[test]
fn test_row() {
    use glam::vec2;

    let mut intersection_pipeline = IntersectionPipeline::new();

    intersection_pipeline.insert_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
            custom_data: 0,
        },
        Membership::Fleet,
    );
    intersection_pipeline.update();
    println!("{:?}", &intersection_pipeline.memberships[Membership::Fleet].rows);
    assert_eq!(intersection_pipeline.memberships[Membership::Fleet].rows.len(), 1);

    // Do we have 2 rows?
    for _ in 0..AccelerationStructure::MIN_COLLIDER_PER_ROW {
        intersection_pipeline.insert_collider(
            Collider {
                radius: 10.0,
                position: vec2(0.0, 10000.0),
                custom_data: 0,
            },
            Membership::Fleet,
        );
    }
    intersection_pipeline.update();
    println!("\n{:?}", &intersection_pipeline.memberships[Membership::Fleet].rows);
    assert_eq!(intersection_pipeline.memberships[Membership::Fleet].rows.len(), 2);

    for _ in 0..AccelerationStructure::MIN_COLLIDER_PER_ROW - 1 {
        intersection_pipeline.insert_collider(
            Collider {
                radius: 10.0,
                position: vec2(0.0, 5000.0),
                custom_data: 0,
            },
            Membership::Fleet,
        );
    }
    let mid = intersection_pipeline.insert_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 5000.0),
            custom_data: 0,
        },
        Membership::Fleet,
    );
    intersection_pipeline.update();
    println!("\n{:?}", &intersection_pipeline.memberships[Membership::Fleet].rows);
    assert_eq!(intersection_pipeline.memberships[Membership::Fleet].rows.len(), 3);

    intersection_pipeline.remove_collider(mid);
    intersection_pipeline.update();
    println!("\n{:?}", &intersection_pipeline.memberships[Membership::Fleet].rows);
    assert_eq!(intersection_pipeline.memberships[Membership::Fleet].rows.len(), 2);
}

#[test]
fn test_random() {
    use glam::vec2;
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        intersection_pipeline.insert_collider(
            Collider {
                radius: random::<f32>() * 256.0,
                position: vec2(random::<f32>() * 512.0 - 256.0, random::<f32>() * 512.0 - 256.0),
                custom_data: 0,
            },
            Membership::System,
        );
        intersection_pipeline.update();

        let other = Collider {
            radius: random::<f32>() * 256.0,
            position: vec2(random::<f32>() * 512.0 - 256.0, random::<f32>() * 512.0 - 256.0),
            custom_data: 0,
        };

        assert_eq!(
            intersection_pipeline.test_collider(other, Membership::System),
            intersection_pipeline.test_collider_brute(other, Membership::System),
            "\n{:?}\n\n{:?}\n",
            &intersection_pipeline.memberships[Membership::System],
            other
        );
    }
}

#[test]
fn test_random_point() {
    use glam::vec2;
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        intersection_pipeline.insert_collider(
            Collider {
                radius: random::<f32>() * 256.0,
                position: vec2(random::<f32>() * 512.0 - 256.0, random::<f32>() * 512.0 - 256.0),
                custom_data: 0,
            },
            Membership::System,
        );
        intersection_pipeline.update();

        let point = vec2(random::<f32>() * 512.0 - 256.0, random::<f32>() * 512.0 - 256.0);
        let other = Collider {
            radius: 0.0,
            position: point,
            custom_data: 0,
        };

        assert_eq!(
            intersection_pipeline.test_point(point, Membership::System),
            intersection_pipeline.test_collider_brute(other, Membership::System),
            "\n{:?}\n\n{:?}\n",
            &intersection_pipeline.memberships[Membership::System],
            other
        );
    }
}
