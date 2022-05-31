use ahash::AHashSet;
use glam::Vec2;
use std::hash::Hash;
use std::u32;

#[derive(Debug, Clone, Copy)]
pub struct Collider<F>
where
    F: Filter + Copy,
{
    pub position: Vec2,
    pub radius: f32,
    pub filter: F,
}
impl<F> Collider<F>
where
    F: Filter + Copy,
{
    pub fn new(position: Vec2, radius: f32, filter: F) -> Self {
        Self {
            radius,
            position,
            filter,
        }
    }

    pub fn top(&self) -> f32 {
        self.position.y - self.radius
    }

    pub fn bot(&self) -> f32 {
        self.position.y + self.radius
    }

    pub fn right(&self) -> f32 {
        self.position.x + self.radius
    }

    pub fn left(&self) -> f32 {
        self.position.x - self.radius
    }

    /// Return true if these colliders intersect (use filter).
    pub fn intersection_test(&self, other: &Self) -> bool {
        self.filter.compare(&other.filter)
            && self.position.distance_squared(other.position)
                <= (self.radius + other.radius).powi(2)
    }

    /// Return true if this collider intersect a point (no filter).
    pub fn intersection_test_point(&self, point: Vec2) -> bool {
        self.position.distance_squared(point) <= self.radius.powi(2)
    }

    /// Return true if these colliders fully overlap (no filter).
    pub fn incorporate_test(&self, other: &Self) -> bool {
        self.position.distance_squared(other.position) <= (self.radius - other.radius).powi(2)
    }

    /// Return half of the biggest horizontal lenght within two possibly intersecting horizontal lines.
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

#[derive(Debug, Clone, Copy)]
pub struct ColliderInternal<I, F>
where
    I: Copy + Hash + Eq,
    F: Filter + Copy,
{
    pub collider: Collider<F>,
    pub id: I,
}

/// _________ rows top
///
/// first row
///
/// _________ rows top + lenght * 1
///
/// second row
///
/// _________ rows top + lenght * 2
///
/// last row
///
/// _________ rows top + lenght * num row
#[derive(Debug, Clone)]
struct SAPRow<I, F>
where
    I: Copy + Hash + Eq,
    F: Filter + Copy,
{
    /// Biggest radius found in this row.
    threshold: f32,
    /// The colliders indices on the colliders Vector that overlap this row.
    /// Sorted on the x axis.
    data: Vec<ColliderInternal<I, F>>,
}
impl<I, F> SAPRow<I, F>
where
    I: Copy + Hash + Eq,
    F: Filter + Copy,
{
    fn clear(&mut self) {
        self.threshold = 0.0;
        self.data.clear();
    }
}
impl<I, F> Default for SAPRow<I, F>
where
    I: Copy + Hash + Eq,
    F: Filter + Copy,
{
    fn default() -> Self {
        Self {
            threshold: 0.0,
            data: Vec::with_capacity(16),
        }
    }
}

pub trait Filter {
    /// Return if these 2 filters match.
    fn compare(&self, rhs: &Self) -> bool;
}
impl Filter for () {
    fn compare(&self, _rhs: &Self) -> bool {
        true
    }
}
macro_rules! impl_Filter{
    ($($type:ty),*) => {$( impl Filter for $type  { fn compare(&self, rhs: &Self) -> bool { self.eq(rhs) } })*}
}
impl_Filter! {u8, u16, u32, u64, i8, i16, i32, i64}

/// Allow fast circle-circle and circle-point test.
///
/// After `update()` is called, the current colliders are copied internaly.
/// Modifying the colliders will not change the intersection result
/// until `update()` is called again.
///
/// The intended use is:
/// - Clear previous colliders
/// - Copy all colliders needed into this
/// - Call `update()`
/// - Do intersection tests
///
/// ## Pro tips:
/// - Use this.extend(iter<(collider, id)>) to insert many colliders
/// - `intersection_test()` only takes references thus can be used in
/// multithreaded code if you have many (10k+) colliders to test
/// - Filter can be disabled by using `()`
/// - Filter is already implemented for many int type as `self == rhs`
#[derive(Debug, Default)]
pub struct AccelerationStructure<I, F>
where
    I: Copy + Hash + Eq,
    F: Filter + Copy,
{
    colliders: Vec<ColliderInternal<I, F>>,
    /// The top of the first row.
    rows_top: f32,
    /// The bot of the last row
    rows_bot: f32,
    /// The distance between each row.
    /// This is also equal to the average diameter found.
    rows_lenght: f32,
    /// Rows are sorted on the y axis. From top to bottom.
    rows: Vec<SAPRow<I, F>>,
}
impl<I, F> AccelerationStructure<I, F>
where
    I: Copy + Hash + Eq,
    F: Filter + Copy,
{
    pub fn new() -> Self {
        Self {
            colliders: Default::default(),
            rows_top: 0.0,
            rows_bot: 0.0,
            rows_lenght: 0.0,
            rows: Default::default(),
        }
    }

    /// New collider will be used after `update()` is called.
    pub fn push(&mut self, collider: Collider<F>, id: I) {
        self.colliders.push(ColliderInternal { collider, id })
    }

    pub fn clear(&mut self) {
        self.colliders.clear();
    }

    /// # Panic
    /// Will panic if any collider's position is not real (inf, nan).
    pub fn update(&mut self) {
        if self.colliders.is_empty() {
            self.rows.clear();
            self.rows_bot = 0.0;
            self.rows_top = 0.0;
            self.rows_lenght = 0.0;
            return;
        }

        // Sort the colliders on the x axis.
        self.colliders.sort_unstable_by(|a, b| {
            a.collider
                .position
                .x
                .partial_cmp(&b.collider.position.x)
                .expect("A collider's position is not real")
        });

        // Find rows parameters.
        let mut upper = 0.0f32;
        let mut lower = 0.0f32;
        let mut biggest_radius = 0.0f32;
        let mut average_diameter = 0.0f32;
        for collider_internal in self.colliders.iter() {
            upper = upper.min(collider_internal.collider.position.y);
            lower = lower.max(collider_internal.collider.position.y);
            biggest_radius = biggest_radius.max(collider_internal.collider.radius);
            average_diameter += collider_internal.collider.radius;
        }
        average_diameter = average_diameter / self.colliders.len() as f32 * 2.0;
        upper -= biggest_radius;
        lower += biggest_radius;

        // Clean the rows to reuse them.
        for row in self.rows.iter_mut() {
            row.clear();
        }

        // Get the number of rows we should create.
        let num_row = ((lower - upper) / average_diameter) as usize + 1;
        if num_row > self.rows.len() {
            self.rows.resize_with(num_row, Default::default);
        }

        self.rows_top = upper;
        self.rows_lenght = average_diameter;
        self.rows_bot = ((num_row + 1) as f32).mul_add(self.rows_lenght, self.rows_top);

        // Add colliders to overlapping rows.
        for collider_internal in self.colliders.iter() {
            let first_overlapping_row = self.find_row_at_position(collider_internal.collider.top());

            let mut row_bot = self
                .rows_lenght
                .mul_add((first_overlapping_row + 1) as f32, self.rows_top);

            let collider_bot = collider_internal.collider.bot();

            for row in &mut self.rows[first_overlapping_row..] {
                let threshold = collider_internal.collider.radius;

                row.data.push(*collider_internal);

                row.threshold = row.threshold.max(threshold);

                if collider_bot < row_bot {
                    break;
                }

                row_bot += self.rows_lenght;
            }
        }
    }

    /// Get the index of the row that this position fit into.
    ///
    /// If this is used with the top of a collider, it return the first row that this collider overlap.
    ///
    /// This index is clamped to be within the valid part of this AccelerationStructure.
    fn find_row_at_position(&self, y_postion: f32) -> usize {
        ((y_postion.min(self.rows_bot) - self.rows_top) / self.rows_lenght) as usize
    }

    /// Brute test a collider against every collider until one return true.
    /// Useful for debug.
    pub fn test_collider_brute(&self, collider: Collider<F>) -> bool {
        for collider_internal in self.colliders.iter() {
            if collider.intersection_test(&collider_internal.collider) {
                return true;
            }
        }
        false
    }

    /// Return all colliders that intersect the provided collider.
    ///
    /// Take a closure with the intersecting collider
    /// which return if we should stop the query.
    /// 
    /// - `true` -> stop query
    /// - `false` -> continue query
    pub fn intersect_collider(
        &self,
        collider: Collider<F>,
        mut closure: impl FnMut(&ColliderInternal<I, F>) -> bool,
    ) {
        let first_overlapping_row = self.find_row_at_position(collider.top());

        if first_overlapping_row >= self.rows.len() {
            return;
        }

        let mut seen = AHashSet::new();

        let collider_bot = collider.bot();

        let mut row_bot = self
            .rows_lenght
            .mul_add((first_overlapping_row + 1) as f32, self.rows_top);

        for row in &self.rows[first_overlapping_row..] {
            // The furthest we should look to the left and right.
            let left_threshold = collider.position.x - row.threshold - collider.radius;
            let right_threshold = collider.position.x + row.threshold + collider.radius;

            let left_index = row.data.partition_point(|&collider_internal| {
                collider_internal.collider.position.x < left_threshold
            });

            // Look from left to right.
            for collider_internal in &row.data[left_index..] {
                if collider_internal.collider.position.x > right_threshold {
                    break;
                } else if seen.insert(collider_internal.id)
                    && collider.intersection_test(&collider_internal.collider)
                {
                    if closure(collider_internal) {
                        return;
                    }
                }
            }

            if collider_bot < row_bot {
                break;
            }
            row_bot += self.rows_lenght;
        }
    }

    /// Return all colliders that intersect the provided point.
    ///
    /// Take a closure with the intersecting collider
    /// which return if we should stop the query.
    /// 
    /// - `true` -> stop query
    /// - `false` -> continue query
    pub fn intersect_point(
        &self,
        point: Vec2,
        filter: F,
        mut closure: impl FnMut(&ColliderInternal<I, F>) -> bool,
    ) {
        let mut seen = AHashSet::new();

        let overlapping_row = self.find_row_at_position(point.y);

        if let Some(row) = self.rows.get(overlapping_row) {
            // The furthest we should look to the left and right.
            let left_threshold = point.x - row.threshold;
            let right_threshold = point.x + row.threshold;

            let left_index = row.data.partition_point(|&collider_internal| {
                collider_internal.collider.position.x < left_threshold
            });

            // Look from left to right.
            for collider_internal in &row.data[left_index..] {
                if collider_internal.collider.position.x > right_threshold {
                    break;
                } else if filter.compare(&collider_internal.collider.filter)
                    && seen.insert(collider_internal.id)
                    && collider_internal.collider.intersection_test_point(point)
                {
                    if closure(collider_internal) {
                        return;
                    }
                }
            }
        }
    }

    /// Get the separation line between each row. Useful for debug.
    pub fn get_rows_separation(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(self.rows.len() + 1);
        v.push(self.rows_top);

        for (row, i) in self.rows.iter().zip(1..) {
            if row.data.is_empty() {
                break;
            }
            v.push((i as f32).mul_add(self.rows_lenght, self.rows_top));
        }

        v
    }
}
impl<I, F> Extend<(Collider<F>, I)> for AccelerationStructure<I, F>
where
    I: Copy + Hash + Eq,
    F: Filter + Copy,
{
    fn extend<T: IntoIterator<Item = (Collider<F>, I)>>(&mut self, iter: T) {
        self.colliders.extend(
            iter.into_iter()
                .map(|(collider, id)| ColliderInternal { collider, id }),
        )
    }
}

#[test]
fn test_random_colliders() {
    use rand::prelude::*;
    let mut rng = thread_rng();

    // Random test.
    for _ in 0..50000 {
        let mut acc: AccelerationStructure<u32, u8> = AccelerationStructure::new();

        let og_collider = Collider::new(
            rng.gen::<Vec2>() * 96.0 - 48.0,
            rng.gen_range(0.0f32..16.0),
            rng.gen_range(0u8..4),
        );

        let mut expected_result = Vec::new();

        // Add colliders.
        for i in 0..rng.gen_range(0u32..64) {
            let new_collider = Collider::new(
                rng.gen::<Vec2>() * 96.0 - 48.0,
                rng.gen_range(0.0f32..16.0),
                rng.gen_range(0u8..4),
            );
            acc.push(new_collider, i);

            if og_collider.intersection_test(&new_collider) {
                expected_result.push(i);
            }
        }

        expected_result.sort();

        acc.update();

        let mut result = Vec::new();
        acc.intersect_collider(og_collider, |c| {
            result.push(c.id);
            false
        });
        result.sort();

        assert_eq!(result, expected_result,);
    }
}
