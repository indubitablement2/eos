use ahash::AHashSet;
use common::collider::Collider;
use crossbeam_channel::{bounded, Receiver, Sender};
use glam::Vec2;
use indexmap::IndexMap;
use std::{fmt::Debug, thread::spawn};

/// Recycled after a collider is removed.
/// For Eos, this id is the same as the entity's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColliderId(pub u32);
impl ColliderId {
    pub fn new(id: u32) -> Self {
        Self(id)
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
    /// Row have a static lenght.
    /// TODO: Is this the optimal row lenght?
    const LENGHT: f32 = 64.0;

    /// Insert all possible overlap from `self` in `buffer`.
    fn get_possible_overlap(
        &self,
        colliders: &IndexMap<ColliderId, Collider>,
        collider: Collider,
        buffer: &mut AHashSet<u32>,
    ) {
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
    fn get_possible_overlap_point(
        &self,
        colliders: &IndexMap<ColliderId, Collider>,
        point: Vec2,
        buffer: &mut AHashSet<u32>,
    ) {
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
    pub colliders: IndexMap<ColliderId, Collider>,
    /// The top of the first row.
    rows_top: f32,
    /// The bot of the last row
    rows_bot: f32,
    /// Rows are sorted on the y axis. From top to bottom.
    rows: Vec<SAPRow>,
}
impl AccelerationStructure {
    fn new() -> Self {
        Self {
            colliders: IndexMap::new(),
            rows_top: 0.0,
            rows_bot: 0.0,
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
        for collider in self.colliders.values() {
            upper = upper.min(collider.position.y);
            lower = lower.max(collider.position.y);
            biggest_radius = biggest_radius.max(collider.radius);
        }
        upper -= biggest_radius + 1.0;
        lower += biggest_radius + 1.0;
        self.rows_top = upper;
        self.rows_bot = lower;

        // Clean the rows to reuse them.
        for row in self.rows.iter_mut() {
            row.biggest_radius = 0.0;
            row.data.clear();
        }

        // Create rows.
        let num_row = ((lower - upper) / SAPRow::LENGHT) as usize + 1;
        if num_row > self.rows.len() {
            self.rows.resize_with(num_row, Default::default);
        }

        // Add colliders to overlapping rows.
        for (collider, collider_index) in self.colliders.values().zip(0u32..) {
            let first_overlapping_row = self.find_row_at_position(collider.top());

            let mut row_bot = SAPRow::LENGHT.mul_add((first_overlapping_row + 1) as f32, self.rows_top);
            let collider_bot = collider.bot();
            for row in &mut self.rows[first_overlapping_row..] {
                row.data.push(collider_index);
                if collider_bot < row_bot {
                    break;
                }
                row_bot += SAPRow::LENGHT;
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
        ((y_postion.min(self.rows_bot) - self.rows_top) / SAPRow::LENGHT) as usize
    }

    /// Get all colliders that should be tested.
    fn get_colliders_to_test(&self, collider: Collider) -> AHashSet<u32> {
        let mut to_test = AHashSet::with_capacity(16);

        let first_overlapping_row = self.find_row_at_position(collider.top());

        if first_overlapping_row < self.rows.len() {
            let mut row_bot = SAPRow::LENGHT.mul_add((first_overlapping_row + 1) as f32, self.rows_top);
            let collider_bot = collider.bot();
            for row in &self.rows[first_overlapping_row..] {
                row.get_possible_overlap(&self.colliders, collider, &mut to_test);
                if collider_bot < row_bot {
                    break;
                }
                row_bot += SAPRow::LENGHT;
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
        for other in self.colliders.values() {
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
    pub fn intersect_collider_into(&self, collider: Collider, buffer: &mut Vec<ColliderId>) {
        for i in self.get_colliders_to_test(collider).into_iter() {
            if let Some((collider_id, other)) = self.colliders.get_index(i as usize) {
                if collider.intersection_test(*other) {
                    buffer.push(*collider_id);
                }
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
    pub fn intersect_point_into(&self, point: Vec2, buffer: &mut Vec<ColliderId>) {
        for i in self.get_colliders_to_test_point(point).into_iter() {
            if let Some((collider_id, other)) = self.colliders.get_index(i as usize) {
                if other.intersection_test_point(point) {
                    buffer.push(*collider_id);
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
            v.push((i as f32).mul_add(SAPRow::LENGHT, self.rows_top));
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
    assert!(!intersection_pipeline.snapshot.test_collider(Collider {
        radius: 10.0,
        position: vec2(0.0, 0.0),
    }));

    // Basic test.
    let first_collider_id = ColliderId::new(1);
    intersection_pipeline.snapshot.colliders.insert(
        first_collider_id,
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
        },
    );
    intersection_pipeline.snapshot.update();
    println!("{:?}", &intersection_pipeline.snapshot);
    assert!(intersection_pipeline.snapshot.test_collider(Collider {
        radius: 10.0,
        position: vec2(-4.0, 0.0),
    }));
    assert!(intersection_pipeline.snapshot.test_collider(Collider {
        radius: 10.0,
        position: vec2(4.0, 0.0),
    }));

    // Removing collider.
    intersection_pipeline.snapshot.colliders.remove(&first_collider_id);
    intersection_pipeline.snapshot.update();
    for row in &intersection_pipeline.snapshot.rows {
        assert_eq!(row.data.len(), 0, "row should be empty");
    }
    assert_eq!(
        intersection_pipeline.snapshot.rows.len(),
        1,
        "there should only be one row"
    );
    assert!(!intersection_pipeline.snapshot.test_collider(Collider {
        radius: 10.0,
        position: vec2(0.0, 0.0),
    }));
}

#[test]
fn test_random() {
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        intersection_pipeline.snapshot.colliders.insert(
            ColliderId::new(1),
            Collider {
                radius: random::<f32>() * 256.0,
                position: random::<Vec2>() * 512.0 - 256.0,
            },
        );
        intersection_pipeline.snapshot.update();

        let other = Collider {
            radius: random::<f32>() * 256.0,
            position: random::<Vec2>() * 512.0 - 256.0,
        };

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

        intersection_pipeline.snapshot.colliders.insert(
            ColliderId::new(1),
            Collider {
                radius: random::<f32>() * 256.0,
                position: random::<Vec2>() * 512.0 - 256.0,
            },
        );
        intersection_pipeline.snapshot.update();

        let point = random::<Vec2>() * 512.0 - 256.0;
        let other = Collider {
            radius: 0.0,
            position: point,
        };

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

    let og_collider = Collider {
        radius: 10.0,
        position: Vec2::ZERO,
    };

    for i in 0..10 {
        let new_collider = Collider {
            radius: 10.0,
            position: Vec2::ZERO,
        };

        intersection_pipeline
            .snapshot
            .colliders
            .insert(ColliderId::new(i), new_collider);
    }

    intersection_pipeline.snapshot.update();

    let mut result = Vec::new();
    intersection_pipeline
        .snapshot
        .intersect_collider_into(og_collider, &mut result);
    assert_eq!(10, result.len(),);
}

#[test]
fn test_random_colliders() {
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        let og_collider = Collider {
            radius: random::<f32>() * 128.0,
            position: (random::<Vec2>() * 512.0 - 256.0),
        };

        let mut expected_result = Vec::new();

        // Add colliders.
        for i in 0..32 {
            let new_collider = Collider {
                radius: random::<f32>() * 128.0,
                position: (random::<Vec2>() * 512.0 - 256.0),
            };

            let new_id = ColliderId::new(i);
            intersection_pipeline.snapshot.colliders.insert(new_id, new_collider);

            if og_collider.intersection_test(new_collider) {
                expected_result.push(new_id);
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

#[test]
fn test_remove_collider() {
    use indexmap::IndexSet;
    use rand::random;

    let mut intersection_pipeline = IntersectionPipeline::new();

    let collider = Collider {
        radius: 1.0,
        position: Vec2::ZERO,
    };

    let mut used = IndexSet::new();
    for i in 0..1000 {
        assert!(used.insert({
            let id = ColliderId::new(i);
            intersection_pipeline.snapshot.colliders.insert(id, collider);
            id
        }));

        if used.is_empty() {
            continue;
        }

        match random::<u32>() % 20 == 0 {
            true => intersection_pipeline.snapshot.update(),
            false => {
                let id = used.swap_remove_index(random::<usize>() % used.len()).unwrap();
                intersection_pipeline.snapshot.colliders.remove(&id);
            }
        }
    }

    intersection_pipeline.snapshot.update();

    assert_eq!(used.len(), intersection_pipeline.snapshot.colliders.len());
}
