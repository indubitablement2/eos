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

/// _________ -infinity
/// 
/// first row
/// 
/// _________ Some real number
/// 
/// second row
/// 
/// _________ Some real number
/// 
/// last row
/// 
/// _________ infinity
/// 
/// 
/// Use a really large number instead of infinity to avoid it spreading.
#[derive(Debug, Clone)]
struct SAPRow {
    /// This is a smaller number than bot as up is negative.
    /// This is also bot of the previous row (if there is one).
    top: f32,
    /// This is a bigger number than top as down is positive.
    /// This is also top of the next row (if there is one).
    bot: f32,
    /// The biggest radius found in this row.
    biggest_radius: f32,
    /// The colliders indices on the colliders IndexMap that overlap this row.
    /// Sorted on the x axis.
    data: Vec<u32>,
}
impl Default for SAPRow {
    fn default() -> Self {
        Self {
            top: 0.0,
            bot: 0.0,
            biggest_radius: 0.0,
            data: Vec::with_capacity(AccelerationStructure::MIN_COLLIDER_PER_ROW * 4),
        }
    }
}
impl SAPRow {
    /// The lenght from an extremity to the other.
    fn lenght(&self) -> f32 {
        self.bot - self.top
    }

    /// Split this row in two.
    /// `self` become the top row.
    /// `other` become the bottom row.
    /// 
    /// Takes a SAPRow to allow recycling and avoid unnecessary allocations.
    fn split(&mut self, mut other: Self) -> Self {
        other.bot = self.bot;
        self.bot -= self.lenght() * 0.5;
        other.top = self.bot;
        other
    }

    /// Insert all possible overlap from `self` in `buffer`.
    fn get_possible_overlap(
        &self,
        colliders: &IndexMap<ColliderId, Collider>,
        collider: Collider,
        buffer: &mut AHashSet<u32>
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
        buffer: &mut AHashSet<u32>
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
    /// Rows are sorted on the y axis. From top to bottom.
    rows: Vec<SAPRow>,
}
impl AccelerationStructure {
    /// Row will not be split unless they have at least that many collider.
    const MIN_COLLIDER_PER_ROW: usize = 8;
    /// Row will not be split under this lenght.
    const MIN_ROW_LENGHT: f32 = 512.0;

    fn new() -> Self {
        Self {
            colliders: IndexMap::new(),
            rows: Vec::new(),
        }
    }

    /// # Safety
    /// Will panic if any collider's position is not real.
    fn update(&mut self) {
        // Recycle old rows.
        let num_old_row = self.rows.len();
        let mut old_row = std::mem::replace(&mut self.rows, Vec::with_capacity(num_old_row + 4));
        for row in &mut old_row {
            row.data.clear();
        }

        // Prepare first row.
        let mut first_row = old_row.pop().unwrap_or_default();
        first_row.top = -1.0e30f32;
        first_row.bot = 1.0e30f32;
        self.rows.push(first_row);

        if self.colliders.is_empty() {
            return;
        }

        for (collider, index) in self.colliders.values().zip(0u32..) {
            let row_index = self.find_row_at_position(collider.position.y);
            let row = &mut self.rows[row_index];
            row.data.push(index);

            if row.data.len() > Self::MIN_COLLIDER_PER_ROW && row.lenght() > Self::MIN_ROW_LENGHT {
                // Split this row in two.
                let mut new_row = row.split(old_row.pop().unwrap_or_default());

                // Move colliders that now belong in the new row.
                let row_bot = row.bot;
                row.data.drain_filter(|collider_index| {
                    row_bot < self.colliders[*collider_index as usize].position.y
                }).for_each(|collider_index| {
                    new_row.data.push(collider_index)
                });

                // Add the new row after the current row.
                self.rows.insert(row_index, new_row);
            }
        }

        // Add colliders to overlapping rows.
        for (collider, collider_index) in self.colliders.values().zip(0u32..) {
            let collider_top = collider.top();
            let first_overlapping_row = self.find_row_at_position(collider_top);

            let collider_bot = collider.bot();
            for row in &mut self.rows[first_overlapping_row..] {
                row.data.push(collider_index);
                if collider_bot < row.bot  {
                    break;
                }
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
    pub fn find_row_at_position(&self, y_postion: f32) -> usize {
        self.rows.partition_point(|row| row.bot < y_postion)
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
        let mut to_test = AHashSet::with_capacity(16);

        let collider_top = collider.top();
        let first_overlapping_row = self.find_row_at_position(collider_top);

        let collider_bot = collider.bot();
        for row in &self.rows[first_overlapping_row..] {
            row.get_possible_overlap(&self.colliders, collider, &mut to_test);
            if collider_bot < row.bot  {
                break;
            }
        }

        // Test each Collider we have collected.
        for i in to_test.into_iter() {
            if collider.intersection_test(self.colliders[i as usize]) {
                return true;
            }
        }

        false
    }

    /// Return all colliders that intersect the provided collider.
    /// Buffer will containt the result.
    pub fn intersect_collider_into(&self, collider: Collider, buffer: &mut Vec<ColliderId>) {
        let mut to_test = AHashSet::with_capacity(16);

        let collider_top = collider.top();
        let first_overlapping_row = self.find_row_at_position(collider_top);

        let collider_bot = collider.bot();
        for row in &self.rows[first_overlapping_row..] {
            row.get_possible_overlap(&self.colliders, collider, &mut to_test);
            if collider_bot < row.bot  {
                break;
            }
        }

        // Test each Collider we have collected.
        for i in to_test.into_iter() {
            if let Some((collider_id, other)) = self.colliders.get_index(i as usize) {
                if collider.intersection_test(*other) {
                    buffer.push(*collider_id);
                }
            }
        }
    }

    /// Test if any collider intersect with the provided point.
    pub fn test_point(&self, point: Vec2) -> bool {
        let mut to_test = AHashSet::with_capacity(16);

        if let Some(row) = self.rows.get(self.find_row_at_position(point.y)) {
            row.get_possible_overlap_point(&self.colliders, point, &mut to_test);
        }

        // Test each Collider we have collected.
        for i in to_test.into_iter() {
            if self.colliders[i as usize].intersection_test_point(point) {
                return true;
            }
        }

        false
    }

    /// Return all colliders that intersect the provided point.
    /// Buffer will containt the result.
    pub fn intersect_point_into(&self, point: Vec2, buffer: &mut Vec<ColliderId>) {
        let mut to_test = AHashSet::with_capacity(16);

        if let Some(row) = self.rows.get(self.find_row_at_position(point.y)) {
            row.get_possible_overlap_point(&self.colliders, point, &mut to_test);
        }

        // Test each Collider we have collected.
        for i in to_test.into_iter() {
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

        self.rows.iter().for_each(|row| {
            v.push(row.top);
        });

        if let Some(last) = self.rows.last() {
            v.push(last.bot);
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
    intersection_pipeline.snapshot.colliders.insert(first_collider_id, Collider {
        radius: 10.0,
        position: vec2(0.0, 0.0),
    });
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
        assert_eq!(row.data.len(), 1, "should only have one row");
    }
    assert!(!intersection_pipeline.snapshot.test_collider(Collider {
        radius: 10.0,
        position: vec2(0.0, 0.0),
    }));
}

#[test]
fn test_row() {
    use glam::vec2;

    let mut intersection_pipeline = IntersectionPipeline::new();

    intersection_pipeline.snapshot.colliders.insert(
        ColliderId::new(1),
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
        },
    );
    intersection_pipeline.snapshot.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 1);

    // We should have 2 rows after this
    for i in 0..AccelerationStructure::MIN_COLLIDER_PER_ROW {
        intersection_pipeline.snapshot.colliders.insert(
            ColliderId::new(2 + i as u32),
            Collider {
                radius: 10.0,
                position: vec2(0.0, 10000.0),
            },
        );
    }
    intersection_pipeline.snapshot.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 2);

    // We should have 2 rows.
    for i in 0..AccelerationStructure::MIN_COLLIDER_PER_ROW - 1 {
        intersection_pipeline.snapshot.colliders.insert(
            ColliderId::new(2 + i as u32 + AccelerationStructure::MIN_COLLIDER_PER_ROW as u32),
            Collider {
                radius: 10.0,
                position: vec2(0.0, 5000.0),
            },
        );
    }
    intersection_pipeline.snapshot.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 2);

    let mid = ColliderId::new(1000);
    intersection_pipeline.snapshot.colliders.insert(
        mid,
        Collider {
            radius: 10.0,
            position: vec2(0.0, 5000.0),
        },
    );
    intersection_pipeline.snapshot.update();
    println!("\n{:?}", &intersection_pipeline.snapshot.rows);
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 3);

    intersection_pipeline.snapshot.colliders.remove(&mid);
    intersection_pipeline.snapshot.update();
    println!("\n{:?}", &intersection_pipeline.snapshot.rows);
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 2);
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

        intersection_pipeline.snapshot.colliders.insert(ColliderId::new(i), new_collider);
    }

    intersection_pipeline.snapshot.update();

    let mut result = Vec::new();
    intersection_pipeline.snapshot.intersect_collider_into(og_collider, &mut result);
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
        intersection_pipeline.snapshot.intersect_collider_into(og_collider, &mut result);
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
