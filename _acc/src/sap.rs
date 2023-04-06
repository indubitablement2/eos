use crate::bounding_shape::*;
use ahash::AHashSet;
use std::ops::Range;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SapConfig {
    /// Column width is determined by the average shape's width mutiplied by this.
    ///
    /// Default: `8.0`
    pub column_width_multiplier: f32,
    /// Columns will have at least this width. Set to 0 to disable.
    ///
    /// Default: `0.00001` to avoid potential division by 0
    pub min_column_width: f32,
    /// Used to avoid making too many column when there are few shapes that are far apart.
    /// ## Example:
    /// If this is 4 and there are 15 shapes, no more than `15 / 4 + 1` (4) column will be created.
    ///
    /// Default: `64`
    pub min_shapes_per_column: u32,
    /// If we should use stable sorting. Otherwise unstable is used.
    ///
    /// Default: `false`
    pub sort_stable: bool,
    /// Skip the sorting step when calling `update()`.
    /// Only set to true if you are 100% sure the shapes will be sorted by bottom edge.
    ///
    /// This can save quite a bit of time if your shapes already need to be sorted this way
    /// eg. for drawing.
    ///
    /// Default: `false`
    pub skip_sort: bool,
}
impl Default for SapConfig {
    fn default() -> Self {
        Self {
            column_width_multiplier: 8.0,
            min_column_width: 0.00001, // Avoid division by 0.
            min_shapes_per_column: 64,
            sort_stable: false,
            skip_sort: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Column<B> {
    /// Biggest height found in this column.
    biggest_height: f32,
    /// The unique index of the shape using the same order as `shapes`.
    indices: Vec<u32>,
    /// The bounding shape that overlap this column.
    /// Sorted by bottom edge.
    shapes: Vec<B>,
}
impl<B> Column<B>
where
    B: BoundingShape + Copy,
{
    pub const fn new() -> Self {
        Self {
            biggest_height: 0.0,
            indices: Vec::new(),
            shapes: Vec::new(),
        }
    }

    fn push(&mut self, shape: B, index: u32, height: f32) {
        self.biggest_height = self.biggest_height.max(height);
        self.indices.push(index);
        self.shapes.push(shape);
    }

    fn clear(&mut self) {
        self.biggest_height = 0.0;
        self.indices.clear();
        self.shapes.clear();
    }

    fn reserve(&mut self, additional: usize) {
        self.indices.reserve(additional);
        self.shapes.reserve(additional);
    }
}

/// A simple intersection acceleration structure that keeps no state between updates.
///
/// After `update()` is called, the current shapes are copied internaly.
/// Queuing more shapes here will not change the snapshot result
/// until `update()` is called again.
///
/// The intended use is:
/// - Copy shapes and idx needed into this with `queue()` or `extend::<(I, B)>()`
/// - Call `update()`
/// - Do intersection tests
///
/// # Panic
/// Some methods will panic if any number is NaN.
///
/// ## Pro tips:
/// - `extend(iter<(I, B)>)` will prealocate memory. It should be the prefered
/// way to insert many shapes.
/// - `intersect()` only takes references. It can be used in
/// multithreaded code if you have many (10k+) shapes to test.
/// - If your shapes are already sorted by bottom edge, it can speed up `update()` by 2-3x.
/// - This will reuse alocated memory and the current config between updates.
/// Nothing else is kept.
#[derive(Debug, Clone)]
pub struct Sap<I, B>
where
    B: BoundingShape + Copy,
{
    /// The shapes to use for the next call to `update()`.
    next_shapes: Vec<B>,
    /// The idx to use for the next call to `update()`.
    next_idx: Vec<I>,
    next_leftmost: f32,
    next_rightmost: f32,
    next_biggest_width: f32,
    next_sum_width: f32,

    /// The left side of the first column.
    column_left: f32,
    /// The distance between each columns.
    column_width: f32,
    /// Columns from left to right.
    columns: Vec<Column<B>>,

    /// The shapes in use. Same order as `current_idx`. Only used for `intersecting_pairs()`.
    current_shapes: Vec<B>,
    /// Unlike the shapes, the idx are not copied into the columns.
    current_idx: Vec<I>,
    /// The index of the id/ shape pair. This is sorted by aabb bottom edge.
    current_indices: Vec<u32>,

    pub config: SapConfig,
}
impl<I, B> Sap<I, B>
where
    B: BoundingShape + Copy,
{
    pub fn new() -> Self {
        Self {
            next_shapes: Default::default(),
            next_idx: Default::default(),
            column_left: Default::default(),
            column_width: 0.00001, // Avoid division by 0
            columns: Default::default(),
            current_idx: Default::default(),
            config: Default::default(),
            current_indices: Default::default(),
            current_shapes: Default::default(),
            next_leftmost: f32::MAX,
            next_rightmost: f32::MIN,
            next_biggest_width: 0.0,
            next_sum_width: 0.0,
        }
    }

    /// New shape will be used after `update()` is called.
    pub fn queue(&mut self, id: I, shape: B) {
        self.next_leftmost = self.next_leftmost.min(shape.left());
        self.next_rightmost = self.next_rightmost.max(shape.right());
        let width = shape.width();
        self.next_biggest_width = self.next_biggest_width.max(width);
        self.next_sum_width += width;

        self.next_shapes.push(shape);
        self.next_idx.push(id);
    }

    /// Reserve capacity for `additional` shapes/idx in the queue.
    pub fn reserve(&mut self, additional: usize) {
        self.next_shapes.reserve(additional);
        self.next_idx.reserve(additional);
    }

    /// Clear the shapes/idx that would've been used next update.
    ///
    /// Does not affect the current snapshot.
    pub fn clear_queue(&mut self) {
        self.next_shapes.clear();
        self.next_idx.clear();

        self.reset_accumulators();
    }

    fn reset_accumulators(&mut self) {
        self.next_leftmost = f32::MAX;
        self.next_rightmost = f32::MIN;
        self.next_biggest_width = 0.0;
        self.next_sum_width = 0.0;
    }

    pub fn clear_snapshot(&mut self) {
        self.current_shapes.clear();
        self.current_idx.clear();
        self.current_indices.clear();

        self.columns.clear();
        self.column_left = 0.0;
        self.column_width = 0.00001; // Avoid division by 0
    }

    /// Return if the shapes/idx to be used when calling `update()` is empty.
    pub fn is_future_data_empty(&self) -> bool {
        self.next_shapes.is_empty()
    }

    /// Return if the shapes/idx curently in use is empty.
    /// All intersection test will return nothing in this case.
    pub fn is_snapshot_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn update(&mut self) {
        self.clear_snapshot();
        std::mem::swap(&mut self.next_idx, &mut self.current_idx);
        std::mem::swap(&mut self.next_shapes, &mut self.current_shapes);

        if self.current_shapes.is_empty() {
            self.reset_accumulators();
            return;
        }

        self.current_indices
            .extend(0..self.current_shapes.len() as u32);

        // Sort by shape bottom edge.
        if !self.config.skip_sort {
            if self.config.sort_stable {
                self.current_indices.sort_by(|a, b| {
                    self.current_shapes[*a as usize]
                        .bot()
                        .partial_cmp(&self.current_shapes[*b as usize].bot())
                        .expect("numbers should not be NaN")
                });
            } else {
                self.current_indices.sort_unstable_by(|a, b| {
                    self.current_shapes[*a as usize]
                        .bot()
                        .partial_cmp(&self.current_shapes[*b as usize].bot())
                        .expect("numbers should not be NaN")
                });
            }
        }

        // Compute column's parameters.
        let average_width = self.next_sum_width / self.current_shapes.len() as f32;
        let mut total_width = self.next_rightmost - self.next_leftmost;
        // Add padding.
        self.next_leftmost -= total_width * 0.01;
        self.next_rightmost += total_width * 0.01;
        total_width = self.next_rightmost - self.next_leftmost;
        let desired_columns_width =
            (average_width * self.config.column_width_multiplier).max(self.config.min_column_width);
        let desired_num_column = (total_width / desired_columns_width) as usize + 1;
        let max_num_column =
            self.current_shapes.len() / self.config.min_shapes_per_column as usize + 1;
        let num_column = desired_num_column.min(max_num_column).max(1);
        self.column_width = total_width / num_column as f32;
        self.column_left = self.next_leftmost;
        self.columns.resize(num_column, Column::new());

        self.reset_accumulators();

        // Pre-alocate & clear columns.
        let mut nums = vec![0usize; self.columns.len()];
        for b in self.current_shapes.iter() {
            let range = self.find_columns_range(b);
            nums[range].iter_mut().for_each(|n| *n += 1);
        }
        for (additional, column) in nums.into_iter().zip(self.columns.iter_mut()) {
            column.clear();
            column.reserve(additional);
        }

        // Add shapes to overlapping columns.
        for &index in self.current_indices.iter() {
            let shape = self.current_shapes[index as usize];
            let height = shape.height();
            let range = self.find_columns_range(&shape);
            for column in self.columns[range].iter_mut() {
                column.push(shape, index, height);
            }
        }
    }

    /// Find which column a point belong to.
    ///
    /// This index may be out of bound.
    fn find_column(&self, x: f32) -> usize {
        ((x - self.column_left) / self.column_width) as usize
    }

    /// Return the index range of this shape.
    ///
    /// The range may be out of bound.
    fn find_columns_range(&self, shape: &B) -> Range<usize> {
        self.find_column(shape.left())..self.find_column(shape.right()) + 1
    }

    /// Return the index range of this shape clamped to always be valid.
    fn find_columns_range_bounded(&self, shape: &B) -> Range<usize> {
        let r = self.find_columns_range(shape);
        r.start.min(self.columns.len())..r.end.min(self.columns.len())
    }

    /// Brute test a shape against every shapes until one return true.
    /// **Useful for debug only.**
    pub fn intersect_brute(&self, shape: &B) -> bool {
        for other in self.current_shapes.iter() {
            if shape.intersect(other) {
                return true;
            }
        }
        false
    }

    /// Called by `intersect()`.
    ///
    /// `ignore` is shape index that will be ignored.
    ///
    /// `u32` are the internal index of the bounding shapes.
    /// This index is the insertion order eg: first queued item has index 0, seconds has 1, etc.
    pub fn intersect_internal(
        &self,
        shape: &B,
        ignore: &mut AHashSet<u32>,
        mut closure: impl FnMut(u32, &B, &I) -> bool,
    ) {
        if self.columns.is_empty() {
            return;
        }

        let range = self.find_columns_range_bounded(shape);

        for column in self.columns[range].iter() {
            // The furthest we should look.
            let bot_threshold = shape.bot() + column.biggest_height;

            // Where we will start our search.
            // The first bounding shape with bot >= our top.
            let top = shape.top();
            let start_index = column.shapes.partition_point(|other| other.bot() < top);

            // Look from top to bottom.
            for (other, &other_index) in column.shapes[start_index..]
                .iter()
                .zip(column.indices[start_index..].iter())
            {
                if other.bot() > bot_threshold {
                    break;
                }

                // We already know that other.bot >= self.top
                // from the partition point above.
                if shape.intersect_fast(other) && ignore.insert(other_index) {
                    if closure(other_index, other, &self.current_idx[other_index as usize]) {
                        return;
                    }
                }
            }
        }
    }

    /// Return all shapes/idx that intersect the provided shape.
    ///
    /// Take a closure with the intersecting shape and its id
    /// which return if we should stop the query early.
    ///
    /// - `true` -> stop query
    /// - `false` -> continue query
    pub fn intersect(&self, shape: &B, mut closure: impl FnMut(&B, &I) -> bool) {
        let mut ignore = AHashSet::new();
        self.intersect_internal(shape, &mut ignore, |_, other, id| closure(other, id));
    }

    /// Called by `intersect_point()`.
    ///
    /// `ignore` is shape index that will be ignored.
    ///
    /// `u32` are the internal index of the bounding shapes.
    /// This index is the insertion order eg: first queued item has index 0, seconds has 1, etc.
    pub fn intersect_point_internal(
        &self,
        x: f32,
        y: f32,
        ignore: &mut AHashSet<u32>,
        closure: impl FnMut(u32, &B, &I) -> bool,
    ) {
        self.intersect_internal(&B::from_point(x, y), ignore, closure)
    }

    /// Return all shapes/idx that intersect the provided point.
    ///
    /// Take a closure with the intersecting shape and its id
    /// which return if we should stop the query early.
    ///
    /// - `true` -> stop query
    /// - `false` -> continue query
    pub fn intersect_point(&self, x: f32, y: f32, mut closure: impl FnMut(&B, &I) -> bool) {
        let mut ignore = AHashSet::new();
        self.intersect_point_internal(x, y, &mut ignore, |_, other, id| closure(other, id));
    }

    /// Return all unique intersecting pairs.
    ///
    /// See `intersecting_pairs()` for a simple version that use this internally.
    ///
    /// See `intersect_internal()` for what `u32` is.
    pub fn intersecting_pairs_internal(
        &self,
        ignore: &mut AHashSet<u32>,
        max_pair: Option<usize>,
        mut closure: impl FnMut((u32, &B, &I), (u32, &B, &I)),
    ) {
        let mut seen = AHashSet::new();

        for &i in self.current_indices.iter() {
            seen.insert(i);
            ignore.insert(i);

            let id = &self.current_idx[i as usize];
            let b = &self.current_shapes[i as usize];

            let mut num_pair = 0;

            self.intersect_internal(b, &mut seen, |other_index, other_b, other_id| {
                if !ignore.contains(&other_index) {
                    closure((i, b, id), (other_index, other_b, other_id))
                }

                num_pair += 1;

                if max_pair.is_some_and(|max_pair| num_pair >= max_pair) {
                    true
                } else {
                    false
                }
            });

            seen.clear();
        }
    }

    /// Return all unique intersecting pairs.
    pub fn intersecting_pairs(
        &self,
        max_pair: Option<usize>,
        mut closure: impl FnMut((&B, &I), (&B, &I)),
    ) {
        let mut ignore = AHashSet::with_capacity(self.current_idx.len());
        self.intersecting_pairs_internal(&mut ignore, max_pair, |a, b| {
            closure((a.1, a.2), (b.1, b.2))
        })
    }

    /// Get the separation line between each column of the current snapshot.
    /// Include leftmost and rightmost.
    /// Useful for debug.
    pub fn get_columns_separation(&self) -> Vec<f32> {
        (0..=self.columns.len())
            .map(|i| i as f32 * self.column_width)
            .collect()
    }
}
impl<I, B> Extend<(I, B)> for Sap<I, B>
where
    B: BoundingShape + Copy,
{
    fn extend<T: IntoIterator<Item = (I, B)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let additional = hint.1.unwrap_or(hint.0);

        self.reserve(additional);

        for (id, shape) in iter {
            self.queue(id, shape);
        }
    }
}
impl<I, B> Default for Sap<I, B>
where
    B: BoundingShape + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}
