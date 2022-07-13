use ahash::AHashSet;
use glam::Vec2;
use std::{hash::Hash, ops::RangeInclusive};

pub trait Collider {
    fn intersection_test(&self, other: &Self) -> bool;
    fn intersection_test_point(&self, point: Vec2) -> bool;
    fn left(&self) -> f32;
    fn right(&self) -> f32;
    /// Up is negative.
    fn top(&self) -> f32;
    /// Down is positive.
    fn bot(&self) -> f32;
    fn from_aabb(left: f32, right: f32, top: f32, bot: f32) -> Self;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}
impl Collider for Circle {
    fn intersection_test(&self, other: &Self) -> bool {
        self.center.distance_squared(other.center) <= (self.radius + other.radius).powi(2)
    }

    fn intersection_test_point(&self, point: Vec2) -> bool {
        self.center.distance_squared(point) <= self.radius.powi(2)
    }

    fn left(&self) -> f32 {
        self.center.x - self.radius
    }

    fn right(&self) -> f32 {
        self.center.x + self.radius
    }

    fn top(&self) -> f32 {
        self.center.y - self.radius
    }

    fn bot(&self) -> f32 {
        self.center.y + self.radius
    }

    fn from_aabb(left: f32, right: f32, top: f32, _bot: f32) -> Self {
        let radius = (right - left) * 0.5;
        Self {
            center: Vec2::new(left, top) + radius,
            radius,
        }
    }
}
impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct AABB<C: Collider> {
    left: f32,
    top: f32,
    right: f32,
    bot: f32,
    _marker: std::marker::PhantomData<C>,
}
impl<C: Collider> AABB<C> {
    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn height(&self) -> f32 {
        self.bot - self.top
    }

    fn new(collider: &C) -> Self {
        AABB {
            left: collider.left(),
            top: collider.top(),
            right: collider.right(),
            bot: collider.bot(),
            _marker: Default::default(),
        }
    }

    fn collider(&self) -> C {
        C::from_aabb(self.left, self.right, self.top, self.bot)
    }
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
struct SAPRow<C: Copy + Collider, I: Copy> {
    /// Biggest width found in this row.
    biggest_width: f32,
    /// The aabbs that overlap this row.
    /// Sorted by right edge.
    aabbs: Vec<AABB<C>>,
    /// The idx of the aabbs using the same order as `aabbs`.
    idx: Vec<I>,
}
impl<C: Copy + Collider, I: Copy> SAPRow<C, I> {
    fn push(&mut self, aabb: AABB<C>, id: I, aabb_width: f32) {
        self.biggest_width = self.biggest_width.max(aabb_width);
        self.aabbs.push(aabb);
        self.idx.push(id);
    }

    fn clear(&mut self) {
        self.biggest_width = 0.0;
        self.aabbs.clear();
        self.idx.clear();
    }

    fn reserve(&mut self, additional: usize) {
        self.aabbs.reserve(additional);
        self.idx.reserve(additional);
    }
}
impl<C: Copy + Collider, I: Copy> Default for SAPRow<C, I> {
    fn default() -> Self {
        Self {
            biggest_width: Default::default(),
            aabbs: Default::default(),
            idx: Default::default(),
        }
    }
}

/// A simple intersection acceleration structure that keeps no state.
///
/// After `update()` is called, the current colliders are copied internaly.
/// Modifying the colliders here will not change the intersection result
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
#[derive(Debug, Clone)]
pub struct AccelerationStructure<C: Collider + Copy, I: Copy + Hash + Eq> {
    /// The collider/idx pairs to use for the next call to `update()`.
    data: Vec<(AABB<C>, I)>,
    /// The index of the last row.
    row_end_index: usize,
    /// The top of the first row.
    rows_top: f32,
    /// The distance between each row.
    /// This is also equal to the average height of the colliders.
    rows_height: f32,
    /// Rows are sorted top to bottom.
    rows: Vec<SAPRow<C, I>>,
}
impl<C: Collider + Copy, I: Copy + Hash + Eq> AccelerationStructure<C, I> {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            row_end_index: 0,
            rows_top: 0.0,
            rows_height: 0.00001, // Avoid division by 0
            rows: Default::default(),
        }
    }

    /// New collider will be used after `update()` is called.
    pub fn push(&mut self, collider: &C, id: I) {
        self.data.push((AABB::new(collider), id))
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Return if the data to be used when calling `update()` is empty.
    pub fn is_future_data_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Return if the data curently in use is empty.
    /// All intersection test will return nothing in this case.
    pub fn is_snapshot_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// # Panic
    /// Will panic if any collider's position is not real (inf, nan).
    pub fn update(&mut self) {
        if self.data.is_empty() {
            self.rows.clear();
            self.row_end_index = 0;
            self.rows_top = 0.0;
            self.rows_height = 0.00001; // Avoid division by 0
            return;
        }

        // Sort by aabb right edge.
        self.data.sort_unstable_by(|(a, _), (b, _)| {
            a.right
                .partial_cmp(&b.right)
                .expect("A collider's right is not real")
        });

        // Find row's parameters.
        let mut upper = f32::MAX;
        let mut lower = f32::MIN;
        let mut biggest_height = 0.0f32;
        let mut average_height = 0.0f32;
        for (aabb, _) in self.data.iter() {
            upper = upper.min(aabb.top);
            lower = lower.max(aabb.bot);
            let height = aabb.height();
            biggest_height = biggest_height.max(height);
            average_height += height;
        }
        average_height = average_height / self.data.len() as f32;

        // Get the number of rows we should create.
        let num_row = ((lower - upper) / average_height) as usize + 1;
        self.rows.resize_with(num_row, Default::default);

        self.row_end_index = self.rows.len() - 1;
        self.rows_top = upper;
        self.rows_height = average_height;

        // Pre-alocate & clear rows.
        let mut nums = vec![0usize; num_row];
        for (aabb, _) in self.data.iter() {
            let range = self.find_rows_range(aabb);
            nums[range].iter_mut().for_each(|n| *n += 1);
        }
        for (additional, row) in nums.iter().zip(self.rows.iter_mut()) {
            row.clear();
            row.reserve(*additional);
        }

        // Add aabbs to overlapping rows.
        for (aabb, id) in self.data.iter() {
            let aabb_width = aabb.width();
            let range = self.find_rows_range(aabb);
            for row in &mut self.rows[range] {
                row.push(*aabb, *id, aabb_width);
            }
        }
    }

    /// Return the index range of this aabb.
    fn find_rows_range(&self, aabb: &AABB<C>) -> RangeInclusive<usize> {
        ((aabb.top - self.rows_top) / self.rows_height) as usize
            ..=((aabb.bot - self.rows_top) / self.rows_height) as usize
    }

    /// Return the index range of this aabb clamped to always be valid.
    fn find_rows_range_bounded(&self, aabb: &AABB<C>) -> RangeInclusive<usize> {
        (((aabb.top - self.rows_top) / self.rows_height) as usize).min(self.row_end_index)
            ..=(((aabb.bot - self.rows_top) / self.rows_height) as usize).min(self.row_end_index)
    }

    fn find_row_unbounded(&self, y: f32) -> usize {
        ((y - self.rows_top) / self.rows_height) as usize
    }

    /// Brute test a collider against every colliders until one return true.
    /// Useful for debug.
    pub fn intersect_collider_brute(&self, collider: C) -> bool {
        for (other, _) in self.data.iter() {
            if collider.intersection_test(&other.collider()) {
                return true;
            }
        }
        false
    }

    /// Return all colliders that intersect the provided collider.
    ///
    /// Take a closure with the collider and its id of the intersecting collider
    /// which return if we should stop the query early.
    ///
    /// - `true` -> stop query
    /// - `false` -> continue query
    pub fn intersect(&self, collider: &C, mut closure: impl FnMut(&C, &I) -> bool) {
        if self.rows.is_empty() {
            return;
        }

        let aabb = AABB::new(collider);
        let mut seen = AHashSet::new();
        let range = self.find_rows_range_bounded(&aabb);

        for row in &self.rows[range] {
            // The furthest we should look to the right.
            let right_threshold = aabb.right + row.biggest_width;

            // Where we will start our search.
            // The first collider with right >= our left.
            let start_index = row
                .aabbs
                .partition_point(|other_aabb| other_aabb.right < aabb.left);

            // Look from left to right.
            for (other_aabb, other_id) in row.aabbs[start_index..]
                .iter()
                .zip(row.idx[start_index..].iter())
            {
                if other_aabb.right > right_threshold {
                    break;
                }

                if other_aabb.left <= aabb.right
                    && other_aabb.bot >= aabb.top
                    && other_aabb.top <= aabb.bot
                    && seen.insert(*other_id)
                {
                    let other_collider = other_aabb.collider();
                    if collider.intersection_test(&other_collider) {
                        if closure(&other_collider, other_id) {
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Return all colliders that intersect the provided point.
    ///
    /// Take a closure with the collider and its id of the intersecting collider
    /// which return if we should stop the query early.
    ///
    /// - `true` -> stop query
    /// - `false` -> continue query
    pub fn intersect_point(&self, point: Vec2, mut closure: impl FnMut(&C, &I) -> bool) {
        let mut seen = AHashSet::new();
        let row_index = self.find_row_unbounded(point.y);

        if let Some(row) = self.rows.get(row_index) {
            // The furthest we should look to the right.
            let right_threshold = row.biggest_width;

            // Where we will start our search.
            // The first collider with right >= our left.
            let start_index = row
                .aabbs
                .partition_point(|other_aabb| other_aabb.right < point.x);

            // Look from left to right.
            for (other_aabb, other_id) in row.aabbs[start_index..]
                .iter()
                .zip(row.idx[start_index..].iter())
            {
                if other_aabb.right > right_threshold {
                    break;
                }

                if other_aabb.left <= point.x
                    && other_aabb.bot >= point.x
                    && other_aabb.top <= point.x
                    && seen.insert(*other_id)
                {
                    let other_collider = other_aabb.collider();
                    if other_collider.intersection_test_point(point) {
                        if closure(&other_collider, other_id) {
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Get the separation line between each row. Useful for debug.
    pub fn get_rows_separation(&self) -> Vec<f32> {
        (0..=self.rows.len())
            .map(|i| i as f32 * self.rows_height)
            .collect()
    }
}
impl<C: Collider + Copy, I: Copy + Hash + Eq> Extend<(C, I)> for AccelerationStructure<C, I> {
    fn extend<T: IntoIterator<Item = (C, I)>>(&mut self, iter: T) {
        self.data.extend(
            iter.into_iter()
                .map(|(collider, id)| (AABB::new(&collider), id)),
        )
    }
}
impl<C: Collider + Copy, I: Copy + Hash + Eq> Default for AccelerationStructure<C, I> {
    fn default() -> Self {
        Self::new()
    }
}
