use ahash::AHashSet;
use crossbeam_channel::{bounded, Receiver, Sender};
use glam::Vec2;
use std::thread::spawn;
use std::u32;
extern crate test;

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub radius: f32,
    pub position: Vec2,
}
impl Collider {
    pub fn new(radius: f32, position: Vec2) -> Self {
        Self { radius, position }
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

    /// Return true if these colliders fully overlap.
    pub fn incorporate_test(self, other: Collider) -> bool {
        self.position.distance_squared(other.position) <= (self.radius - other.radius).powi(2)
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
            let distance = (self.position.y - top).abs().min((self.position.y - bot).abs());
            // This is used instead of the collider's radius as it is smaller.
            (self.radius.powi(2) - distance.powi(2)).sqrt()
        }
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
    /// The biggest radius found in this row.
    ///
    /// . / " \\ <- same radius, but smaller threshold in this row
    /// ___
    /// / . . . \
    threshold: f32,
    /// The colliders indices on the colliders Vector that overlap this row.
    /// Sorted on the x axis.
    data: Vec<u32>,
}
impl Default for SAPRow {
    fn default() -> Self {
        Self {
            threshold: 0.0,
            data: Vec::with_capacity(16),
        }
    }
}

/// Allow fast circle-circle and circle-point test.
///
/// # Safety
/// After any modification, and until it is updated,
/// any test result will at best be meaningless or at worst will cause a panic due to out of bound array index.
#[derive(Debug)]
pub struct AccelerationStructure<T>
where
    T: Sized + Send + Copy + 'static,
{
    colliders: Vec<Collider>,
    custom_data: Vec<T>,
    /// The top of the first row.
    rows_top: f32,
    /// The bot of the last row
    rows_bot: f32,
    /// The distance between each row.
    /// This is also equal to the average diameter found.
    rows_lenght: f32,
    /// Rows are sorted on the y axis. From top to bottom.
    rows: Vec<SAPRow>,
}
impl<T> Extend<(Collider, T)> for AccelerationStructure<T>
where
    T: Sized + Send + Copy + 'static,
{
    fn extend<I: IntoIterator<Item = (Collider, T)>>(&mut self, iter: I) {
        for (collider, data) in iter {
            self.colliders.push(collider);
            self.custom_data.push(data);
        }
    }
}
impl<T> AccelerationStructure<T>
where
    T: Sized + Send + Copy + 'static,
{
    pub fn new() -> Self {
        Self {
            colliders: Vec::new(),
            custom_data: Vec::new(),
            rows_top: 0.0,
            rows_bot: 0.0,
            rows_lenght: 0.0,
            rows: vec![SAPRow::default()],
        }
    }

    pub fn push(&mut self, collider: Collider, data: T) {
        self.colliders.push(collider);
        self.custom_data.push(data);
    }

    pub fn clear(&mut self) {
        self.colliders.clear();
        self.custom_data.clear();
    }

    /// This function is expensive and warrant its own thread (see `IntersectionPipeline`).
    /// # Safety
    /// Will panic if any collider's position is not real.
    pub fn update(&mut self) {
        if self.colliders.is_empty() {
            self.rows.clear();
            self.rows_bot = 0.0;
            self.rows_top = 0.0;
            self.rows_lenght = 0.0;
            return;
        }

        // Find rows parameters.
        let mut upper = 0.0f32;
        let mut lower = 0.0f32;
        let mut biggest_radius = 0.0f32;
        let mut average_diameter = 0.0f32;
        for collider in self.colliders.iter() {
            upper = upper.min(collider.position.y);
            lower = lower.max(collider.position.y);
            biggest_radius = biggest_radius.max(collider.radius);
            average_diameter += collider.radius;
        }
        average_diameter = average_diameter / self.colliders.len() as f32 * 2.0;
        upper -= biggest_radius;
        lower += biggest_radius;

        // Clean the rows to reuse them.
        for row in self.rows.iter_mut() {
            row.data.clear();
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
        for (collider, collider_index) in self.colliders.iter().zip(0u32..) {
            let first_overlapping_row = self.find_row_at_position(collider.top());

            let mut row_bot = self
                .rows_lenght
                .mul_add((first_overlapping_row + 1) as f32, self.rows_top);
            let collider_bot = collider.bot();
            for row in &mut self.rows[first_overlapping_row..] {
                row.data.push(collider_index);
                if collider_bot < row_bot {
                    break;
                }
                row_bot += self.rows_lenght;
            }
        }

        // Sort each row on the x axis.
        for row in &mut self.rows {
            row.data.sort_unstable_by(|a, b| {
                self.colliders[*a as usize]
                    .position
                    .x
                    .partial_cmp(&self.colliders[*b as usize].position.x)
                    .unwrap_or_else( || {
                        error!("A collider has a position on the x axis that is not a real number. Terminating intersection pipeline update loop...");
                        panic!("A collider has a position on the x axis that is not a real number.");
                    })
            });
        }

        // Find biggest distance in each row.
        let mut row_top = self.rows_top;
        for row in &mut self.rows {
            let row_bot = row_top + self.rows_lenght;
            row.threshold = row.data.iter().fold(0.0, |acc, i| {
                self.colliders[*i as usize]
                    .biggest_slice_within_row(row_top, row_bot)
                    .max(acc)
            });
            row_top += self.rows_lenght;
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
    pub fn test_collider_brute(&self, collider: Collider) -> bool {
        for other in self.colliders.iter() {
            if collider.intersection_test(*other) {
                return true;
            }
        }
        false
    }

    /// Return all colliders that intersect the provided collider.
    pub fn intersect_collider_into(&self, collider: Collider, buffer: &mut Vec<T>) {
        buffer.clear();

        let first_overlapping_row = self.find_row_at_position(collider.top());

        if first_overlapping_row >= self.rows.len() {
            return;
        }

        let mut seen = AHashSet::new();

        let collider_bot = collider.bot();
        let mut row_top = self.rows_lenght.mul_add(first_overlapping_row as f32, self.rows_top);
        let mut row_bot = row_top + self.rows_lenght;
        for row in &self.rows[first_overlapping_row..] {
            // This is used instead of the collider's radius as it is smaller.
            let threshold = collider.biggest_slice_within_row(row_top, row_bot);

            // The furthest we should look to the left and right.
            let left_threshold = collider.position.x - row.threshold - threshold;
            let right_threshold = collider.position.x + row.threshold + threshold;

            let left_index = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < left_threshold);

            // Look from left to right.
            for i in &row.data[left_index..] {
                let other = self.colliders[*i as usize];
                if other.position.x > right_threshold {
                    break;
                } else if seen.insert(*i) {
                    if collider.intersection_test(other) {
                        buffer.push(self.custom_data[*i as usize]);
                    }
                }
            }

            if collider_bot < row_bot {
                break;
            }
            row_bot += self.rows_lenght;
            row_top += self.rows_lenght;
        }
    }

    /// Return all colliders that intersect the provided collider.
    ///
    /// See `intersect_collider_into()` if you want to reuse the buffer to store the result.
    pub fn intersect_collider(&self, collider: Collider) -> Vec<T> {
        let mut buffer = Vec::new();
        self.intersect_collider_into(collider, &mut buffer);
        buffer
    }

    /// Return the first collider intersecting the provided collider if any.
    pub fn intersect_collider_first(&self, collider: Collider) -> Option<T> {
        let first_overlapping_row = self.find_row_at_position(collider.top());

        if first_overlapping_row >= self.rows.len() {
            return None;
        }

        let mut seen = AHashSet::new();

        let collider_bot = collider.bot();
        let mut row_top = self.rows_lenght.mul_add(first_overlapping_row as f32, self.rows_top);
        let mut row_bot = row_top + self.rows_lenght;
        for row in &self.rows[first_overlapping_row..] {
            // This is used instead of the collider's radius as it is smaller.
            let threshold = collider.biggest_slice_within_row(row_top, row_bot);

            // The furthest we should look to the left and right.
            let left_threshold = collider.position.x - row.threshold - threshold;
            let right_threshold = collider.position.x + row.threshold + threshold;

            let left_index = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < left_threshold);

            // Look from left to right.
            for i in &row.data[left_index..] {
                let other = self.colliders[*i as usize];
                if other.position.x > right_threshold {
                    break;
                } else if seen.insert(*i) {
                    if collider.intersection_test(other) {
                        return Some(self.custom_data[*i as usize]);
                    }
                }
            }

            if collider_bot < row_bot {
                break;
            }
            row_bot += self.rows_lenght;
            row_top += self.rows_lenght;
        }

        None
    }

    /// Return all colliders that intersect the provided point.
    /// Buffer will containt the result.
    pub fn intersect_point_into(&self, point: Vec2, buffer: &mut Vec<T>) {
        buffer.clear();

        let mut seen = AHashSet::new();

        let overlapping_row = self.find_row_at_position(point.y);

        if let Some(row) = self.rows.get(overlapping_row) {
            // The furthest we should look to the left and right.
            let left_threshold = point.x - row.threshold;
            let right_threshold = point.x + row.threshold;

            let left_index = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < left_threshold);

            // Look from left to right.
            for i in &row.data[left_index..] {
                let other = self.colliders[*i as usize];
                if other.position.x > right_threshold {
                    break;
                } else if seen.insert(*i) {
                    if other.intersection_test_point(point) {
                        buffer.push(self.custom_data[*i as usize]);
                    }
                }
            }
        }
    }

    /// Return all colliders that intersect the provided point.
    ///
    /// See `intersect_point_into()` if you want to reuse the buffer to store the result.
    pub fn intersect_point(&self, point: Vec2) -> Vec<T> {
        let mut buffer = Vec::new();
        self.intersect_point_into(point, &mut buffer);
        buffer
    }

    /// Return the first collider intersecting this point if any.
    pub fn intersect_point_first(&self, point: Vec2) -> Option<T> {
        let mut seen = AHashSet::new();

        let overlapping_row = self.find_row_at_position(point.y);

        if let Some(row) = self.rows.get(overlapping_row) {
            // The furthest we should look to the left and right.
            let left_threshold = point.x - row.threshold;
            let right_threshold = point.x + row.threshold;

            let left_index = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < left_threshold);

            // Look from left to right.
            for i in &row.data[left_index..] {
                let other = self.colliders[*i as usize];
                if other.position.x > right_threshold {
                    break;
                }
                if seen.insert(*i) {
                    if other.intersection_test_point(point) {
                        return Some(self.custom_data[*i as usize]);
                    }
                }
            }
        }

        None
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

/// This wrap two `AccelerationStructure`, so that while one is used to make intersection test,
/// the other is being updated. Update are therefore fully async, but there is a delay before changes take effect.
/// If you want no delay, use the `AccelerationStructure` directly.
#[derive(Debug)]
pub struct IntersectionPipeline<T>
where
    T: Sized + Send + Copy + 'static,
{
    pub update_request_sender: Sender<AccelerationStructure<T>>,
    pub update_result_receiver: Receiver<AccelerationStructure<T>>,
    pub snapshot: AccelerationStructure<T>,
    /// A place to park another `AccelerationStructure`,
    /// if you want to wait before requesting another update.
    pub outdated: Option<AccelerationStructure<T>>,
}
impl<T> IntersectionPipeline<T>
where
    T: Sized + Send + Copy + 'static,
{
    pub fn new() -> Self {
        let (update_request_sender, update_request_receiver) = bounded(0);
        let (update_result_sender, update_result_receiver) = bounded(0);

        spawn(move || Self::runner_loop(update_request_receiver, update_result_sender));

        Self {
            update_request_sender,
            update_result_receiver,
            snapshot: AccelerationStructure::new(),
            outdated: Some(AccelerationStructure::new()),
        }
    }

    /// Drop the current runner thread and start a new one.
    ///
    /// Its snapshot will be lost and remplaced with a new empty one.
    pub fn start_new_runner_thread(&mut self) {
        let (update_request_sender, update_request_receiver) = bounded(0);
        let (update_result_sender, update_result_receiver) = bounded(0);

        spawn(move || Self::runner_loop(update_request_receiver, update_result_sender));

        if self.outdated.is_none() {
            self.outdated = Some(AccelerationStructure::new());
        }

        self.update_request_sender = update_request_sender;
        self.update_result_receiver = update_result_receiver;
    }

    fn runner_loop(
        update_request_receiver: Receiver<AccelerationStructure<T>>,
        update_result_sender: Sender<AccelerationStructure<T>>,
    ) {
        while let Ok(mut acceleration_structure) = update_request_receiver.recv() {
            acceleration_structure.update();
            if update_result_sender.send(acceleration_structure).is_err() {
                break;
            }
        }
    }
}

#[test]
fn test_random_colliders() {
    use rand::random;

    // Random test.
    for _ in 0..10000 {
        let mut acc: AccelerationStructure<u32> = AccelerationStructure::new();

        let og_collider = Collider::new(random::<f32>() * 16.0, random::<Vec2>() * 64.0 - 32.0);

        let mut expected_result = Vec::new();

        // Add colliders.
        for i in 0..random::<u32>() % 64 {
            let new_collider = Collider::new(random::<f32>() * 16.0, random::<Vec2>() * 64.0 - 32.0);
            acc.push(new_collider, i);

            if og_collider.intersection_test(new_collider) {
                expected_result.push(i);
            }
        }

        expected_result.sort();

        acc.update();

        let mut result = acc.intersect_collider(og_collider);
        result.sort();

        assert_eq!(result, expected_result, "\n{:?}\n\n{:?}\n", result, expected_result);
    }
}

/// v0.0.1 3200 ns
/// v0.0.2 2450 ns
#[bench]
fn bench_intersect_collider(b: &mut test::Bencher) {
    use rand::Rng;
    use test::black_box;

    // Create a large random intersection pipeline.
    let mut rng = rand::thread_rng();
    let mut acc: AccelerationStructure<u32> = AccelerationStructure::new();
    for i in 0..5000 {
        acc.push(
            Collider::new(rng.gen::<f32>() * 32.0, rng.gen::<Vec2>() * 1024.0 - 512.0),
            i,
        );
    }
    acc.update();

    b.iter(|| {
        let collider = Collider::new(rng.gen::<f32>() * 32.0, rng.gen::<Vec2>() * 1024.0 - 512.0);
        let result = acc.intersect_collider(collider);
        black_box(result);
    });
}
