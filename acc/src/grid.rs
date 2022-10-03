use crate::bounding_shape::*;
use ahash::{AHashMap, AHashSet};
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GridConfig {
    /// Cell size is determined by the average shape's width mutiplied by this.
    pub cell_size_multiplier: f32,
    /// Cell will have at least this size. Set to <=0 to disable.
    pub min_cell_size: f32,
}
impl Default for GridConfig {
    fn default() -> Self {
        Self {
            cell_size_multiplier: 4.0,
            min_cell_size: 0.0,
        }
    }
}

/// This is slower and takes a lot more memory.
#[derive(Debug, Clone)]
pub struct Grid<B>
where
    B: BoundingShape,
{
    /// The shapes to use for the next call to `update()`.
    next_shapes: Vec<B>,
    /// The sum of all shape's width in `next_shape`.
    next_sum_width: f32,
    next_leftmost: f32,
    next_topmost: f32,
    next_rightmost: f32,
    next_botmost: f32,

    /// Where the grid begins.
    left: f32,
    /// Where the grid begins.
    top: f32,
    /// Cell are always square, so this is the width and height of cell.
    cell_size: f32,
    /// Number of cell per row.
    width: usize,
    /// Number of cell per column.
    height: usize,
    /// Shapes that intersect this cell are stored in `shape_indices`.
    cells: Vec<Range<u32>>,
    /// Indice of shape in `current_shapes`.
    shape_indices: Vec<u32>,
    /// The shapes in use. Indices fetch into this.
    current_shapes: Vec<B>,

    /// Used while updating only.
    buckets: AHashMap<u32, Vec<u32>>,

    pub config: GridConfig,
}
impl<B> Grid<B>
where
    B: BoundingShape,
{
    /// New shape will be used after `update()` is called.
    ///
    /// Return the id of the shape which is just its index in the queue.
    /// eg:
    /// - `queue(first_shape) -> 0`
    /// - `queue(second_shape) -> 1`
    /// - `queue(n_shape) -> n - 1`
    pub fn queue(&mut self, shape: B) -> u32 {
        self.next_sum_width += shape.width();
        self.next_leftmost = self.next_leftmost.min(shape.left());
        self.next_topmost = self.next_topmost.min(shape.top());
        self.next_rightmost = self.next_rightmost.max(shape.right());
        self.next_botmost = self.next_botmost.max(shape.bot());

        let index = self.next_shapes.len() as u32;

        self.next_shapes.push(shape);

        index
    }

    /// Reserve capacity for `additional` shapes in the queue.
    pub fn reserve(&mut self, additional: usize) {
        self.next_shapes.reserve(additional);
    }

    /// Clear the shapes that would've been used next update.
    ///
    /// Does not affect the current snapshot.
    pub fn clear_queue(&mut self) {
        self.next_sum_width = 0.0;
        self.next_leftmost = f32::MAX;
        self.next_topmost = f32::MAX;
        self.next_rightmost = f32::MIN;
        self.next_botmost = f32::MIN;

        self.next_shapes.clear();
    }

    /// Clear the snapshot.
    ///
    /// Does not affect the queue.
    pub fn clear_snapshot(&mut self) {
        self.cells.clear();
        self.current_shapes.clear();
    }

    /// Return if the shapes to be used when calling `update()` is empty.
    pub fn is_queue_empty(&self) -> bool {
        self.next_shapes.is_empty()
    }

    /// Return if the current snapshot is empty.
    /// All intersection test will return nothing in this case.
    pub fn is_snapshot_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn update(&mut self) {
        self.clear_snapshot();

        if self.is_queue_empty() {
            self.clear_queue();
            return;
        }

        std::mem::swap(&mut self.next_shapes, &mut self.current_shapes);

        // Compute grid's parameters.
        self.left = self.next_leftmost;
        self.top = self.next_topmost;
        let average_width = self.next_sum_width / self.current_shapes.len() as f32;
        self.cell_size =
            (average_width * self.config.cell_size_multiplier).max(self.config.min_cell_size);
        self.width = ((self.next_rightmost - self.next_leftmost) / self.cell_size) as usize + 1;
        self.height = ((self.next_botmost - self.next_topmost) / self.cell_size) as usize + 1;

        // Reset next shapes accumulators.
        self.next_sum_width = 0.0;
        self.next_leftmost = f32::MAX;
        self.next_topmost = f32::MAX;
        self.next_rightmost = f32::MIN;
        self.next_botmost = f32::MIN;

        // Add shape index to buckets.
        let mut num_index = 0;
        for (shape, i) in self.current_shapes.iter().zip(0u32..) {
            for y in self.find_row_range(shape) {
                for x in self.find_column_range(shape) {
                    self.buckets
                        .entry((y * self.width + x) as u32)
                        .or_default()
                        .push(i);
                    num_index += 1;
                }
            }
        }

        // Pre-alocate grid.
        self.shape_indices.resize(num_index, 0);
        self.cells.resize(self.width * self.height, 0..0);

        // Copy bucket content to grid.
        let mut next_start = 0;
        for (&cell_index, bucket) in self.buckets.iter() {
            let end = next_start + bucket.len();
            self.shape_indices[next_start..end].copy_from_slice(&bucket);
            self.cells[cell_index as usize] = next_start as u32..end as u32;
            next_start = end;
        }
        self.buckets.clear();
        // for (bucket, cell) in self.buckets.iter_mut().zip(self.cells.iter_mut()) {
        //     if !bucket.is_empty() {
        //         let end = next_start + bucket.len();
        //         self.shape_indices[next_start..end].copy_from_slice(&bucket);
        //         bucket.clear();
        //         *cell = next_start as u32..end as u32;
        //         next_start = end;
        //     }
        // }
    }

    /// Find which column a point belong to.
    ///
    /// This index may be out of bound.
    fn find_column(&self, x: f32) -> usize {
        ((x - self.left) / self.cell_size) as usize
    }

    /// Find which row a point belong to.
    ///
    /// This index may be out of bound.
    fn find_row(&self, y: f32) -> usize {
        ((y - self.top) / self.cell_size) as usize
    }

    /// Return the index range of this shape.
    ///
    /// The range may be out of bound.
    fn find_column_range(&self, shape: &B) -> Range<usize> {
        self.find_column(shape.left())..self.find_column(shape.right()) + 1
    }

    /// Return the index range of this shape.
    ///
    /// The range may be out of bound.
    fn find_row_range(&self, shape: &B) -> Range<usize> {
        self.find_row(shape.top())..self.find_row(shape.bot()) + 1
    }

    /// Return the column index range of this shape clamped to always be valid.
    fn find_column_range_bounded(&self, shape: &B) -> Range<usize> {
        let r = self.find_column_range(shape);
        r.start.min(self.width)..r.end.min(self.width)
    }

    /// Return the row index range of this shape clamped to always be valid.
    fn find_row_range_bounded(&self, shape: &B) -> Range<usize> {
        let r = self.find_row_range(shape);
        r.start.min(self.height)..r.end.min(self.height)
    }

    /// Brute test a shape against every other shapes in the snapshot.
    ///
    /// **Useful for debug only.**
    pub fn intersect_brute(&self, shape: &B, mut closure: impl FnMut(u32, &B) -> bool) {
        for (other, i) in self.current_shapes.iter().zip(0u32..) {
            if shape.intersect(other) {
                if closure(i, other) {
                    return;
                }
            }
        }
    }

    fn get_cell_range(&self, cell_index: usize) -> Range<usize> {
        let r = &self.cells[cell_index];
        r.start as usize..r.end as usize
    }

    /// `seen` are shape index that will be ignored.
    ///
    /// Return all shapes that intersect the provided shape.
    ///
    /// Take a closure with the intersecting shape and its index
    /// which return if we should stop the query early.
    ///
    /// - `true` -> stop query
    /// - `false` -> continue query
    ///
    /// Shape index (`u32`) is the shape queue order.
    pub fn intersect(
        &self,
        shape: &B,
        seen: &mut AHashSet<u32>,
        mut closure: impl FnMut(u32, &B) -> bool,
    ) {
        if self.is_snapshot_empty() {
            return;
        }

        for y in self.find_row_range_bounded(shape) {
            for x in self.find_column_range_bounded(shape) {
                for &i in self.shape_indices[self.get_cell_range(y * self.width + x)].iter() {
                    if seen.insert(i) {
                        let other = &self.current_shapes[i as usize];
                        if shape.intersect(other) {
                            if closure(i, other) {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Call closure for all unique intersecting pairs.
    ///
    /// See `intersect()` for more info.
    ///
    /// Closure's fields:
    /// - first shape and its index
    /// - second shape and its index
    /// - num pair so far for first shape
    ///
    /// `handled` are shape index that will be ignored.
    pub fn intersecting_pairs(&self, mut closure: impl FnMut((u32, &B), (u32, &B))) {
        let mut seen = AHashSet::new();

        for (shape, index) in self.current_shapes.iter().zip(0u32..) {
            seen.insert(index);

            self.intersect(shape, &mut seen, |other_index, other| {
                if index < other_index {
                    closure((index, shape), (other_index, other))
                }
                false
            });

            seen.clear();
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
impl<B> Extend<B> for Grid<B>
where
    B: BoundingShape,
{
    fn extend<T: IntoIterator<Item = B>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let additional = hint.1.unwrap_or(hint.0);

        self.reserve(additional);

        for shape in iter {
            self.queue(shape);
        }
    }
}
impl<B> Default for Grid<B>
where
    B: BoundingShape,
{
    fn default() -> Self {
        Self {
            next_shapes: Default::default(),
            next_sum_width: Default::default(),
            next_leftmost: f32::MAX,
            next_topmost: f32::MAX,
            next_rightmost: f32::MIN,
            next_botmost: f32::MIN,
            left: Default::default(),
            top: Default::default(),
            cell_size: Default::default(),
            width: Default::default(),
            height: Default::default(),
            cells: Default::default(),
            current_shapes: Default::default(),
            buckets: Default::default(),
            config: Default::default(),
            shape_indices: Default::default(),
        }
    }
}
