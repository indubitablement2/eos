use ahash::AHashSet;
use common::collider::Collider;
use crossbeam_channel::{bounded, Receiver, Sender};
use glam::Vec2;
use std::{fmt::Debug, thread::spawn};

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
    biggest_radius: f32,
    /// The colliders indices on the colliders IndexMap that overlap this row.
    /// Sorted on the x axis.
    data: Vec<u32>,
}
impl Default for SAPRow {
    fn default() -> Self {
        Self {
            biggest_radius: 0.0,
            data: Vec::with_capacity(16),
        }
    }
}
impl SAPRow {
    /// Insert all possible overlap from `self` in `buffer`.
    fn get_possible_overlap(&self, colliders: &[Collider], collider: Collider, buffer: &mut AHashSet<u32>) {
        // The furthest we should look to the left and right.
        let left_threshold = collider.left() - self.biggest_radius;
        let right_threshold = collider.right() + self.biggest_radius;

        let left_index = self
            .data
            .partition_point(|i| colliders[*i as usize].position.x < left_threshold);

        // Look from left to right.
        for i in &self.data[left_index..] {
            let other = colliders[*i as usize];
            if other.position.x > right_threshold {
                break;
            }
            buffer.insert(*i);
        }
    }

    /// Insert all possible overlap from `self` in `buffer`.
    fn get_possible_overlap_point(&self, colliders: &[Collider], point: Vec2, buffer: &mut AHashSet<u32>) {
        // The furthest we should look to the left and right.
        let left_threshold = point.x - self.biggest_radius;
        let right_threshold = point.x + self.biggest_radius;

        let left_index = self
            .data
            .partition_point(|i| colliders[*i as usize].position.x < left_threshold);

        // Look from left to right.
        for i in &self.data[left_index..] {
            let other = colliders[*i as usize];
            if other.position.x > right_threshold {
                break;
            }
            buffer.insert(*i);
        }
    }
}

/// # Safety
/// After modifying this and until it is updated,
/// any test result will be at best meaningless or will panic due to out of bound array index.
#[derive(Debug)]
pub struct AccelerationStructure {
    pub colliders: Vec<Collider>,
    /// The top of the first row.
    rows_top: f32,
    /// The bot of the last row
    rows_bot: f32,
    /// The distance between each row.
    /// This is also equal to the biggest radius found clamped within some bound.
    rows_lenght: f32,
    /// Rows are sorted on the y axis. From top to bottom.
    rows: Vec<SAPRow>,
}
impl AccelerationStructure {
    const ROWS_LENGHT_MIN: f32 = 16.0;
    const ROWS_LENGHT_MAX: f32 = 256.0;

    fn new() -> Self {
        Self {
            colliders: Vec::new(),
            rows_top: 0.0,
            rows_bot: 0.0,
            rows_lenght: Self::ROWS_LENGHT_MIN,
            rows: vec![SAPRow::default()],
        }
    }

    /// # Safety
    /// Will panic if any collider's position is not real.
    fn update(&mut self) {
        // Find the upper and lower collider.
        let mut upper = 0.0f32;
        let mut lower = 0.0f32;
        let mut biggest_radius = 0.0f32;
        for collider in self.colliders.iter() {
            upper = upper.min(collider.position.y);
            lower = lower.max(collider.position.y);
            biggest_radius = biggest_radius.max(collider.radius);
        }
        upper -= biggest_radius + 1.0;
        lower += biggest_radius + 1.0;
        self.rows_top = upper;
        self.rows_bot = lower;
        self.rows_lenght = biggest_radius.clamp(Self::ROWS_LENGHT_MIN, Self::ROWS_LENGHT_MAX);

        // Clean the rows to reuse them.
        for row in self.rows.iter_mut() {
            row.biggest_radius = 0.0;
            row.data.clear();
        }

        // Create rows.
        let num_row = ((lower - upper) / self.rows_lenght) as usize + 1;
        if num_row > self.rows.len() {
            self.rows.resize_with(num_row, Default::default);
        }

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

        // Find biggest radius for each row.
        for row in &mut self.rows {
            row.biggest_radius = row
                .data
                .iter()
                .fold(0.0, |acc, i| self.colliders[*i as usize].radius.max(acc));
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

    /// Get all colliders that should be tested.
    fn get_colliders_to_test(&self, collider: Collider) -> AHashSet<u32> {
        let mut to_test = AHashSet::with_capacity(16);

        let first_overlapping_row = self.find_row_at_position(collider.top());

        if first_overlapping_row < self.rows.len() {
            let mut row_bot = self
                .rows_lenght
                .mul_add((first_overlapping_row + 1) as f32, self.rows_top);
            let collider_bot = collider.bot();
            for row in &self.rows[first_overlapping_row..] {
                row.get_possible_overlap(&self.colliders, collider, &mut to_test);
                if collider_bot < row_bot {
                    break;
                }
                row_bot += self.rows_lenght;
            }
        }

        to_test
    }

    /// Get all colliders that should be tested.
    ///
    /// This version is for point
    fn get_colliders_to_test_point(&self, point: Vec2) -> AHashSet<u32> {
        let mut to_test = AHashSet::with_capacity(16);

        let overlapping_row = self.find_row_at_position(point.y);

        if let Some(row) = self.rows.get(overlapping_row) {
            row.get_possible_overlap_point(&self.colliders, point, &mut to_test);
        }

        to_test
    }

    /// Brute test a collider against every collider until one return true. Useful for debug.
    pub fn test_collider_brute(&self, collider: Collider) -> bool {
        for other in self.colliders.iter() {
            if collider.intersection_test(*other) {
                return true;
            }
        }
        false
    }

    /// Test if a any collider intersect the provided collider.
    pub fn test_collider(&self, collider: Collider) -> bool {
        for i in self.get_colliders_to_test(collider).into_iter() {
            if collider.intersection_test(self.colliders[i as usize]) {
                return true;
            }
        }

        false
    }

    /// Return all colliders that intersect the provided collider.
    /// `buffer` will containt the result.
    ///
    /// Provided `buffer` should be clear.
    pub fn intersect_collider_into(&self, collider: Collider, buffer: &mut Vec<u32>) {
        for i in self.get_colliders_to_test(collider).into_iter() {
            let other = self.colliders[i as usize];
            if collider.intersection_test(other) {
                buffer.push(other.id);
            }
        }
    }

    /// Test if any collider intersect with the provided point.
    pub fn test_point(&self, point: Vec2) -> bool {
        for i in self.get_colliders_to_test_point(point).into_iter() {
            if self.colliders[i as usize].intersection_test_point(point) {
                return true;
            }
        }

        false
    }

    /// Return all colliders that intersect the provided point.
    /// Buffer will containt the result.
    pub fn intersect_point_into(&self, point: Vec2, buffer: &mut Vec<u32>) {
        for i in self.get_colliders_to_test_point(point).into_iter() {
            let other = self.colliders[i as usize];
            if other.intersection_test_point(point) {
                buffer.push(other.id);
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

// TODO: Intersection that can filter based on collider id.
/// Allow fast circle-circle intersection and test between colliders.
/// This intersection pipeline is fully async, but there is a delay before commands take effect.
#[derive(Debug)]
pub struct IntersectionPipeline {
    pub update_request_sender: Sender<AccelerationStructure>,
    pub update_result_receiver: Receiver<AccelerationStructure>,
    /// Does not do anything.
    /// This is just there to know when was the last time an update was done.
    /// If this is 0, snapshot has just been updated.
    pub last_update_delta: u64,

    pub snapshot: AccelerationStructure,
}
impl IntersectionPipeline {
    pub fn new() -> Self {
        let (update_request_sender, update_request_receiver) = bounded(0);
        let (update_result_sender, update_result_receiver) = bounded(0);

        spawn(move || runner_loop(update_request_receiver, update_result_sender));

        update_request_sender
            .send(AccelerationStructure::new())
            .expect("Could not send acceleration structure.");

        Self {
            update_request_sender,
            update_result_receiver,
            last_update_delta: 1000,
            snapshot: AccelerationStructure::new(),
        }
    }

    /// Drop the current runner thread and start a new one.
    ///
    /// Its snapshot will be lost and remplaced with a new empty one.
    pub fn start_new_runner_thread(&mut self) {
        let (update_request_sender, update_request_receiver) = bounded(0);
        let (update_result_sender, update_result_receiver) = bounded(0);

        spawn(move || runner_loop(update_request_receiver, update_result_sender));

        update_request_sender
            .send(AccelerationStructure::new())
            .expect("Could not send acceleration structure.");

        self.update_request_sender = update_request_sender;
        self.update_result_receiver = update_result_receiver;
    }
}

fn runner_loop(
    update_request_receiver: Receiver<AccelerationStructure>,
    update_result_sender: Sender<AccelerationStructure>,
) {
    while let Ok(mut acceleration_structure) = update_request_receiver.recv() {
        acceleration_structure.update();
        if update_result_sender.send(acceleration_structure).is_err() {
            break;
        }
    }
}

#[test]
fn test_basic() {
    use glam::vec2;

    let mut intersection_pipeline = IntersectionPipeline::new();

    // Empty.
    assert!(!intersection_pipeline
        .snapshot
        .test_collider(Collider::new(0, 10.0, vec2(0.0, 0.0))));

    // Basic test.
    intersection_pipeline
        .snapshot
        .colliders
        .push(Collider::new_idless(10.0, vec2(0.0, 0.0)));
    intersection_pipeline.snapshot.update();
    println!("{:?}", &intersection_pipeline.snapshot);
    assert!(intersection_pipeline
        .snapshot
        .test_collider(Collider::new(0, 10.0, vec2(-4.0, 0.0))));
    assert!(intersection_pipeline
        .snapshot
        .test_collider(Collider::new(0, 10.0, vec2(4.0, 0.0))));
}

#[test]
fn test_random() {
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        intersection_pipeline.snapshot.colliders.push(Collider::new_idless(
            random::<f32>() * 16.0,
            random::<Vec2>() * 64.0 - 32.0,
        ));
        intersection_pipeline.snapshot.update();

        let other = Collider::new_idless(random::<f32>() * 16.0, random::<Vec2>() * 64.0 - 32.0);

        assert_eq!(
            intersection_pipeline.snapshot.test_collider(other),
            intersection_pipeline.snapshot.test_collider_brute(other),
            "\n{:?}\n\n{:?}\n",
            &intersection_pipeline.snapshot,
            other
        );
    }
}

#[test]
fn test_random_point() {
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        intersection_pipeline.snapshot.colliders.push(Collider::new_idless(
            random::<f32>() * 16.0,
            random::<Vec2>() * 64.0 - 32.0,
        ));
        intersection_pipeline.snapshot.update();

        let point = random::<Vec2>() * 64.0 - 32.0;
        let other = Collider::new_idless(0.0, point);

        assert_eq!(
            intersection_pipeline.snapshot.test_point(point),
            intersection_pipeline.snapshot.test_collider_brute(other),
            "\n{:?}\n\n{:?}\n",
            &intersection_pipeline.snapshot,
            other
        );
    }
}

#[test]
fn test_overlap_colliders() {
    let mut intersection_pipeline = IntersectionPipeline::new();

    let collider = Collider::new_idless(10.0, Vec2::ZERO);

    for i in 0..10 {
        intersection_pipeline.snapshot.colliders.push(collider);
    }

    intersection_pipeline.snapshot.update();

    let mut result = Vec::new();
    intersection_pipeline
        .snapshot
        .intersect_collider_into(collider, &mut result);
    assert_eq!(10, result.len(),);
}

#[test]
fn test_random_colliders() {
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        let og_collider = Collider::new_idless(random::<f32>() * 16.0, random::<Vec2>() * 64.0 - 32.0);

        let mut expected_result = Vec::new();

        // Add colliders.
        for i in 0..32 {
            let new_collider = Collider::new(i, random::<f32>() * 16.0, random::<Vec2>() * 64.0 - 32.0);
            intersection_pipeline.snapshot.colliders.push(new_collider);

            if og_collider.intersection_test(new_collider) {
                expected_result.push(new_collider.id);
            }
        }

        expected_result.sort();

        intersection_pipeline.snapshot.update();

        let mut result = Vec::new();
        intersection_pipeline
            .snapshot
            .intersect_collider_into(og_collider, &mut result);
        result.sort();

        assert_eq!(result, expected_result, "\n{:?}\n\n{:?}\n", result, expected_result);
    }
}
