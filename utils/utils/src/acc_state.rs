use ahash::{AHashSet, AHashMap};
use glam::Vec2;
use std::{hash::Hash, ops::RangeInclusive, sync::Mutex};

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
        todo!()
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
struct SAPRow {
    /// Biggest width found in this row.
    biggest_width: f32,
    /// The aabbs that overlap this row.
    /// Sorted by right edge.
    indices: Vec<u32>,
}
impl SAPRow {
    fn push(&mut self, index: u32, aabb_width: f32) {
        self.biggest_width = self.biggest_width.max(aabb_width);
        self.indices.push(index);
    }

    fn clear(&mut self) {
        self.biggest_width = 0.0;
        self.indices.clear();
    }

    fn reserve(&mut self, additional: usize) {
        self.indices.reserve(additional);
    }
}
impl Default for SAPRow {
    fn default() -> Self {
        Self {
            biggest_width: Default::default(),
            indices: Default::default(),
        }
    }
}

/// Allow fast collider-collider and collider-point intersection test.
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
#[derive(Debug, Default)]
pub struct AccelerationStructure<C: Collider + Copy, I: Copy + Hash + Eq> {
    colliders: Vec<(Vec2, C)>,
    aabbs: Vec<AABB<C>>,
    idx: Vec<I>,
    indices: AHashMap<I, usize>,

    queue_remove: Mutex<AHashSet<I>>,
    queue_set: Mutex<AHashMap<I, C>>,
    queue_move: Mutex<AHashMap<I, Vec2>>,

    /// The index of the last row.
    row_end_index: usize,
    /// The top of the first row.
    rows_top: f32,
    /// The distance between each row.
    /// This is also equal to the average height of the colliders.
    rows_height: f32,
    /// Rows are sorted top to bottom.
    rows: Vec<SAPRow>,
}
impl<C: Collider + Copy, I: Copy + Hash + Eq> AccelerationStructure<C, I> {
    pub fn new() -> Self {
        Self {
            row_end_index: 0,
            rows_top: 0.0,
            rows_height: 0.00001, // Avoid division by 0
            rows: Default::default(),
            colliders: Default::default(),
            aabbs: Default::default(),
            idx: Default::default(),
            indices: Default::default(),
            queue_set: Default::default(),
            queue_remove: Default::default(),
            queue_move: Default::default(),
        }
    }

    /// New collider will be used after `update()` is called.
    pub fn push(&mut self, collider: &C, id: I) {
        self.data.push((AABB::new(collider), id))
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// # Panic
    /// Will panic if any collider's position is not real (inf, nan).
    pub fn update(&mut self) {
        // Remove.
        for id in self.queue_remove.lock().unwrap().drain() {
            if let Some(index) = self.indices.remove(&id) {
                if index + 1 != self.aabbs.len() {
                    *self.indices.get_mut(self.idx.last().unwrap()).unwrap() = index;
                }
                self.aabbs.swap_remove(index);
                self.colliders.swap_remove(index);
                self.idx.swap_remove(index);
            }
        }

        // Set or insert.
        for (id, collider) in self.queue_set.lock().unwrap().drain() {
            if let Some(asd) = self.indices.get(&id) {
                
            } else {
                
            }
        }

        if self.data.is_empty() {
            self.rows.clear();
            self.row_end_index = 0;
            self.rows_top = 0.0;
            self.rows_height = 0.00001; // Avoid division by 0
            return;
        }

        let now = std::time::Instant::now();
        // Sort by aabb right edge.
        let mut sorted: Vec<usize> = (0..self.aabbs.len()).collect();
        sorted.sort_unstable_by(|a, b| {
            self.aabbs[*a].right.partial_cmp(&self.aabbs[*b].right).expect("An aabb's right is not real")
        });
        for (new_index, current_index) in sorted.into_iter().enumerate() {
            if current_index > new_index {
                self.aabbs.swap(new_index, current_index);
                self.colliders.swap(new_index, current_index);
                self.idx.swap(new_index, current_index);
                *self.indices.get_mut(&self.idx[new_index]).unwrap() = new_index;
            }
        }
        println!("sort: {}", now.elapsed().as_micros());

        let now = std::time::Instant::now();
        // Find row's parameters.
        let mut upper = f32::MAX;
        let mut lower = f32::MIN;
        let mut biggest_height = 0.0f32;
        let mut average_height = 0.0f32;
        for aabb in self.aabbs.iter() {
            upper = upper.min(aabb.top);
            lower = lower.max(aabb.bot);
            let height = aabb.height();
            biggest_height = biggest_height.max(height);
            average_height += height;
        }
        average_height = average_height / self.aabbs.len() as f32;
        println!("find data: {}", now.elapsed().as_micros());

        // Get the number of rows we should create.
        let num_row = ((lower - upper) / average_height) as usize + 1;
        self.rows.resize_with(num_row, Default::default);

        self.row_end_index = self.rows.len() - 1;
        self.rows_top = upper;
        self.rows_height = average_height;

        let now = std::time::Instant::now();
        // Pre-alocate & clear rows.
        let mut nums = vec![0usize; num_row];
        for aabb in self.aabbs.iter() {
            let range = self.find_rows_range(aabb);
            nums[range].iter_mut().for_each(|n| *n += 1);
        }
        for (additional, row) in nums.iter().zip(self.rows.iter_mut()) {
            row.clear();
            row.reserve(*additional);
        }

        // Add indices to overlapping rows.
        for (aabb, index) in self.aabbs.iter().zip(0u32..) {
            let aabb_width = aabb.width();
            let range = self.find_rows_range(aabb);
            for row in &mut self.rows[range] {
                row.push(index, aabb_width);
            }
        }
        println!("place colliders: {}", now.elapsed().as_micros());
    }

    /// Get the index of the row that this position fit into.
    ///
    /// If this is used with the top of an aabb, it return the first row that this collider overlap.
    /// With the bot of an aabb, it return the last row.
    ///
    /// This index is **NOT** clamped to be within the valid part of this AccelerationStructure.
    fn find_row_at_position(&self, y_postion: f32) -> usize {
        ((y_postion - self.rows_top) / self.rows_height) as usize
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

    /// Brute test a collider against every colliders until one return true.
    /// Useful for debug.
    pub fn intersect_collider_brute(&self, collider: C) -> bool {
        for other in self.colliders.iter() {
            if collider.intersection_test(&other) {
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
    pub fn intersect_collider(&self, collider: &C, mut closure: impl FnMut(&C, &I) -> bool) {
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
                .indices
                .partition_point(|other_index| self.aabbs[*other_index as usize].right < aabb.left);

            // Look from left to right.
            for other_index in row.indices[start_index..].iter() {
                let other_index_usize = *other_index as usize;
                let other_aabb = &self.aabbs[other_index_usize];

                if other_aabb.right > right_threshold {
                    break;
                }

                if other_aabb.left <= aabb.right
                    && other_aabb.bot >= aabb.top
                    && other_aabb.top <= aabb.bot
                    && seen.insert(*other_index)
                {
                    let other_collider = &self.colliders[other_index_usize];
                    if collider.intersection_test(&other_collider) {
                        if closure(&other_collider, &self.idx[other_index_usize]) {
                            return;
                        }
                    }
                }
            }
        }
    }

    // /// Return all colliders that intersect the provided point.
    // ///
    // /// Take a closure with the intersecting collider
    // /// which return if we should stop the query.
    // ///
    // /// - `true` -> stop query
    // /// - `false` -> continue query
    // pub fn intersect_point(
    //     &self,
    //     point: Vec2,
    //     filter: F,
    //     mut closure: impl FnMut(&ColliderInternal<I, F>) -> bool,
    // ) {
    //     let mut seen = AHashSet::new();

    //     let overlapping_row = self.find_row_at_position(point.y);

    //     if let Some(row) = self.rows.get(overlapping_row) {
    //         // The furthest we should look to the left and right.
    //         let left_threshold = point.x - row.threshold;
    //         let right_threshold = point.x + row.threshold;

    //         let left_index = row.data.partition_point(|&collider_internal| {
    //             collider_internal.collider.position.x < left_threshold
    //         });

    //         // Look from left to right.
    //         for collider_internal in &row.data[left_index..] {
    //             if collider_internal.collider.position.x > right_threshold {
    //                 break;
    //             } else if filter.compare(&collider_internal.collider.filter)
    //                 && seen.insert(collider_internal.id)
    //                 && collider_internal.collider.intersection_test_point(point)
    //             {
    //                 if closure(collider_internal) {
    //                     return;
    //                 }
    //             }
    //         }
    //     }
    // }

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
