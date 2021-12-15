use ahash::AHashSet;
use common::collider::Collider;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use glam::Vec2;
use indexmap::IndexMap;
use std::{cmp::Ordering, fmt::Debug, thread::spawn};

/// Recycled after a collider is removed.
/// For Eos, this id is the same as the entity's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColliderId(pub u32);
impl ColliderId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone)]
struct SAPRow {
    /// y position where this row ends and the next one (if there is one) ends.
    end: f32,
    /// y position where this row start and the previous one (if there is one) ends.
    start: f32,
    /// The biggest radius found in this row.
    biggest_radius: f32,
    /// An indice on the colliders IndexMap.
    data: Vec<u32>,
}
impl Default for SAPRow {
    fn default() -> Self {
        Self {
            end: 0.0,
            start: 0.0,
            biggest_radius: 0.0,
            data: Vec::with_capacity(AccelerationStructureRunner::MIN_COLLIDER_PER_ROW * 4),
        }
    }
}

#[derive(Debug)]
struct AccelerationStructureRunner {
    /// Sorted on the y axis.
    colliders: IndexMap<ColliderId, Collider>,
    /// Sorted on the x axis.
    rows: Vec<SAPRow>,

    modify_collider_receiver: Receiver<(ColliderId, Collider)>,
    remove_collider_receiver: Receiver<ColliderId>,
    /// Sometime remove order may be received before insert when both are called at the same time.
    /// This prevent that by waiting an extra update before removing colliders.
    remove_queue: Vec<ColliderId>,
}
impl AccelerationStructureRunner {
    const MIN_COLLIDER_PER_ROW: usize = 8;
    const MIN_ROW_SIZE: f32 = 512.0;

    fn new(
        remove_collider_receiver: Receiver<ColliderId>,
        modify_collider_receiver: Receiver<(ColliderId, Collider)>,
    ) -> Self {
        Self {
            colliders: IndexMap::new(),
            rows: Vec::new(),
            modify_collider_receiver,
            remove_collider_receiver,
            remove_queue: Vec::new(),
        }
    }

    fn update(&mut self) {
        // Insert / Modify colliders.
        while let Ok((collider_id, collider)) = self.modify_collider_receiver.try_recv() {
            self.colliders.insert(collider_id, collider);
        }
        // Try to remove collider id that could not be found last update again.
        for collider_id in self.remove_queue.drain(..) {
            if self.colliders.remove(&collider_id).is_none() {
                warn!("A collider could not be removed. That could be a memory leak. Ignoring...");
            }
        }
        // Remove colliders and recycle collider id.
        while let Ok(collider_id) = self.remove_collider_receiver.try_recv() {
            if self.colliders.remove(&collider_id).is_none() {
                // We will try again next update.
                self.remove_queue.push(collider_id);
            }
        }

        // Recycle old rows.
        let num_old_row = self.rows.len();
        let mut old_row = std::mem::replace(&mut self.rows, Vec::with_capacity(num_old_row + 4));
        for row in &mut old_row {
            row.data.clear();
        }

        if self.colliders.is_empty() {
            return;
        }

        // Sort on y axis.
        self.colliders.sort_by(|_, v1, _, v2| {
            v1.position.y.partial_cmp(&v2.position.y).unwrap_or_else(|| {
                error!("A collider's position has an imaginary number. Terminating runner thread...");
                panic!("A collider's position has an imaginary number. Terminating runner thread...");
            })
        });

        // Prepare first row.
        let mut current_row = old_row.pop().unwrap_or_default();
        // First row's start should be very large negative number.
        current_row.start = -1.0e30f32;
        let mut num_in_current_row = 0usize;
        // Create rows.
        for collider in self.colliders.values() {
            num_in_current_row += 1;
            current_row.end = collider.position.y;
            if num_in_current_row >= Self::MIN_COLLIDER_PER_ROW {
                // We have the minimum number of collider to make a row.
                if current_row.end - current_row.start >= Self::MIN_ROW_SIZE {
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
        self.rows.last_mut().unwrap().end = 1.0e30f32;

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
}

#[derive(Debug)]
struct AccelerationStructureSnapshot {
    colliders: IndexMap<ColliderId, Collider>,
    rows: Vec<SAPRow>,
}
impl AccelerationStructureSnapshot {
    fn new() -> Self {
        Self {
            colliders: IndexMap::new(),
            rows: Vec::new(),
        }
    }

    /// Update snapshot with the data of a runner.
    /// TODO: Overridde clone from to reuse the resources of self to avoid unnecessary allocations.
    fn clone_from_runner(&mut self, runner: &AccelerationStructureRunner) {
        self.colliders.clone_from(&runner.colliders);
        self.rows.clone_from(&runner.rows);
    }

    /// Brute test a collider against every collider until one return true. Useful for debug.
    fn test_collider_brute(&self, collider: Collider) -> bool {
        for other in self.colliders.values() {
            if collider.intersection_test(*other) {
                return true;
            }
        }
        false
    }

    /// Test if a any collider intersect the provided collider.
    fn test_collider(&self, collider: Collider) -> bool {
        let mut to_test = AHashSet::with_capacity(16);
        let bottom = collider.position.y - collider.radius;
        let top = collider.position.y + collider.radius;
        let first_overlapping_row = self.rows.partition_point(|row| row.end < bottom);
        for row in &self.rows[first_overlapping_row..] {
            // Check that the collider overlap this row.
            if row.start > top {
                break;
            }

            // The furthest we should look to the left and right.
            let left = collider.position.x - collider.radius - row.biggest_radius;
            let right = collider.position.x + collider.radius + row.biggest_radius;

            let left_index = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < left);

            // Look from left to right.
            for i in &row.data[left_index..] {
                let other = self.colliders[*i as usize];
                if other.position.x > right {
                    break;
                }
                to_test.insert(*i);
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
    fn intersect_collider_into(&self, collider: Collider, mut buffer: &mut Vec<ColliderId>) {
        let mut to_test = AHashSet::with_capacity(16);
        let bottom = collider.position.y - collider.radius;
        let top = collider.position.y + collider.radius;
        let first_overlapping_row = self.rows.partition_point(|row| row.end < bottom);
        for row in &self.rows[first_overlapping_row..] {
            // Check that the collider overlap this row.
            if row.start > top {
                break;
            }

            // The furthest we should look to the left and right.
            let left = collider.position.x - collider.radius - row.biggest_radius;
            let right = collider.position.x + collider.radius + row.biggest_radius;

            let left_index = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < left);

            // The furthest we should look in each dirrections.
            let threshold = collider.radius + row.biggest_radius;

            // Look from left to right.
            for i in &row.data[left_index..] {
                let other = self.colliders[*i as usize];
                if other.position.x > right {
                    break;
                }
                to_test.insert(*i);
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
    fn test_point(&self, point: Vec2) -> bool {
        let mut to_test = AHashSet::with_capacity(16);
        let overlapping_row = self.rows.partition_point(|row| row.end < point.y);
        if let Some(row) = self.rows.get(overlapping_row) {
            // The furthest we should look to the left and right.
            let left = point.x - row.biggest_radius;
            let right = point.x + row.biggest_radius;

            let left_index = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < left);

            // Look from left to right.
            for i in &row.data[left_index..] {
                let other = self.colliders[*i as usize];
                if other.position.x > right {
                    break;
                }
                to_test.insert(*i);
            }
        }

        // Test each Collider we have collected.
        for i in to_test.into_iter() {
            if self.colliders[i as usize].intersection_test_point(point) {
                return true;
            }
        }

        false
    }

    /// Get the separation line between each row. Useful for debug.
    fn get_rows_separation(&self) -> Vec<f32> {
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

// TODO: Intersection that can filter based on collider id.
/// Allow fast circle-circle intersection and test between colliders.
/// This intersection pipeline is fully async, but there is a delay before commands take effect.
#[derive(Debug)]
pub struct IntersectionPipeline {
    update_request_sender: Sender<AccelerationStructureRunner>,
    update_result_receiver: Receiver<AccelerationStructureRunner>,

    remove_collider_sender: Sender<ColliderId>,
    modify_collider_sender: Sender<(ColliderId, Collider)>,

    snapshot: AccelerationStructureSnapshot,
}
impl IntersectionPipeline {
    pub fn new() -> Self {
        let (update_request_sender, update_request_receiver) = bounded(0);
        let (update_result_sender, update_result_receiver) = bounded(0);

        let (remove_collider_sender, remove_collider_receiver) = unbounded();
        let (modify_collider_sender, modify_collider_receiver) = unbounded();

        spawn(move || runner_loop(update_request_receiver, update_result_sender));

        let runner = AccelerationStructureRunner::new(remove_collider_receiver, modify_collider_receiver);

        update_request_sender
            .send(runner)
            .expect("Could not send runner to thread.");

        Self {
            update_request_sender,
            update_result_receiver,
            remove_collider_sender,
            modify_collider_sender,
            snapshot: AccelerationStructureSnapshot::new(),
        }
    }

    /// Take a snapshot of the intersection pipeline then request an update.
    pub fn update(&mut self) {
        // Take back runner.
        if let Ok(runner) = self.update_result_receiver.recv() {
            // Take snapshot.
            self.snapshot.clone_from_runner(&runner);

            // Return runner.
            if self.update_request_sender.send(runner).is_err() {
                error!("Intersection pipeline update runner thread dropped.");
            }
        } else {
            error!("Intersection pipeline update runner thread dropped.");
        }
    }

    /// Modify or insert a collider.
    pub fn modify_collider(&self, collider_id: ColliderId, collider: Collider) {
        let _ = self.modify_collider_sender.send((collider_id, collider));
    }

    /// Remove a collider by its id.
    pub fn remove_collider(&self, collider_id: ColliderId) {
        let _ = self.remove_collider_sender.send(collider_id);
    }

    /// Get a copy of a collider by its id.
    pub fn get_collider(&self, collider_id: ColliderId) -> Option<Collider> {
        self.snapshot
            .colliders
            .get(&collider_id)
            .map(|collider| collider.to_owned())
    }

    /// Return all colliders that intersect the provided collider.
    pub fn intersect_collider(&self, collider: Collider) -> Vec<ColliderId> {
        let mut result = Vec::new();
        self.snapshot.intersect_collider_into(collider, &mut result);
        result
    }

    /// Same as intersect_collider, but reuse a buffer to store the intersected colliders.
    pub fn intersect_collider_into(&self, collider: Collider, buffer: &mut Vec<ColliderId>) {
        self.snapshot.intersect_collider_into(collider, buffer);
    }

    /// Test if a any collider intersect the provided collider.
    pub fn test_collider(&self, collider: Collider) -> bool {
        self.snapshot.test_collider(collider)
    }

    /// Test if any collider intersect with the provided point.
    pub fn test_point(&self, point: Vec2) -> bool {
        self.snapshot.test_point(point)
    }

    /// Brute test a collider against every collider until one return true. Useful for debug.
    pub fn test_collider_brute(&self, collider: Collider) -> bool {
        self.snapshot.test_collider_brute(collider)
    }

    /// Get the separation line between each row. Useful for debug.
    pub fn get_rows_separation(&self) -> Vec<f32> {
        self.snapshot.get_rows_separation()
    }
}

fn runner_loop(
    update_request_receiver: Receiver<AccelerationStructureRunner>,
    update_result_sender: Sender<AccelerationStructureRunner>,
) {
    while let Ok(mut runner) = update_request_receiver.recv() {
        runner.update();
        if update_result_sender.send(runner).is_err() {
            error!("Intersection pipeline update runner thread dropped.");
            break;
        }
    }
}

#[test]
fn test_basic() {
    use glam::vec2;

    let mut intersection_pipeline = IntersectionPipeline::new();

    // Empty.
    assert!(!intersection_pipeline.test_collider(Collider {
        radius: 10.0,
        position: vec2(0.0, 0.0),
    }));

    // Basic test.
    let first_collider_id = ColliderId::new(1);
    intersection_pipeline.modify_collider(
        first_collider_id,
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
        },
    );
    intersection_pipeline.update();
    intersection_pipeline.update();
    println!("{:?}", &intersection_pipeline.snapshot);
    assert!(intersection_pipeline.test_collider(Collider {
        radius: 10.0,
        position: vec2(-4.0, 0.0),
    }));
    assert!(intersection_pipeline.test_collider(Collider {
        radius: 10.0,
        position: vec2(4.0, 0.0),
    }));

    // Removing collider.
    intersection_pipeline.remove_collider(first_collider_id);
    intersection_pipeline.update();
    intersection_pipeline.update();
    intersection_pipeline.update();
    for row in &intersection_pipeline.snapshot.rows {
        assert!(row.data.is_empty(), "should be empty");
    }
    assert!(!intersection_pipeline.test_collider(Collider {
        radius: 10.0,
        position: vec2(0.0, 0.0),
    }));
}

#[test]
fn test_row() {
    use glam::vec2;

    let mut intersection_pipeline = IntersectionPipeline::new();

    intersection_pipeline.modify_collider(
        ColliderId::new(1),
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
        },
    );
    intersection_pipeline.update();
    intersection_pipeline.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 1);

    // We should have 2 rows after this
    for i in 0..AccelerationStructureRunner::MIN_COLLIDER_PER_ROW {
        intersection_pipeline.modify_collider(
            ColliderId::new(2 + i as u32),
            Collider {
                radius: 10.0,
                position: vec2(0.0, 10000.0),
            },
        );
    }
    intersection_pipeline.update();
    intersection_pipeline.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 2);

    // We should have 2 rows.
    for i in 0..AccelerationStructureRunner::MIN_COLLIDER_PER_ROW - 1 {
        intersection_pipeline.modify_collider(
            ColliderId::new(2 + i as u32 + AccelerationStructureRunner::MIN_COLLIDER_PER_ROW as u32),
            Collider {
                radius: 10.0,
                position: vec2(0.0, 5000.0),
            },
        );
    }
    intersection_pipeline.update();
    intersection_pipeline.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 2);

    let mid = ColliderId::new(1000);
    intersection_pipeline.modify_collider(
        mid,
        Collider {
            radius: 10.0,
            position: vec2(0.0, 5000.0),
        },
    );
    intersection_pipeline.update();
    intersection_pipeline.update();
    println!("\n{:?}", &intersection_pipeline.snapshot.rows);
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 3);

    intersection_pipeline.remove_collider(mid);
    intersection_pipeline.update();
    intersection_pipeline.update();
    println!("\n{:?}", &intersection_pipeline.snapshot.rows);
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 2);
}

#[test]
fn test_random() {
    use rand::random;

    // Random test.
    for _ in 0..1000 {
        let mut intersection_pipeline = IntersectionPipeline::new();

        intersection_pipeline.modify_collider(
            ColliderId::new(1),
            Collider {
                radius: random::<f32>() * 256.0,
                position: random::<Vec2>() * 512.0 - 256.0,
            },
        );
        intersection_pipeline.update();
        intersection_pipeline.update();

        let other = Collider {
            radius: random::<f32>() * 256.0,
            position: random::<Vec2>() * 512.0 - 256.0,
        };

        assert_eq!(
            intersection_pipeline.test_collider(other),
            intersection_pipeline.test_collider_brute(other),
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

        intersection_pipeline.modify_collider(
            ColliderId::new(1),
            Collider {
                radius: random::<f32>() * 256.0,
                position: random::<Vec2>() * 512.0 - 256.0,
            },
        );
        intersection_pipeline.update();
        intersection_pipeline.update();

        let point = random::<Vec2>() * 512.0 - 256.0;
        let other = Collider {
            radius: 0.0,
            position: point,
        };

        assert_eq!(
            intersection_pipeline.test_point(point),
            intersection_pipeline.test_collider_brute(other),
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

        intersection_pipeline.modify_collider(ColliderId::new(i), new_collider);
    }

    intersection_pipeline.update();
    intersection_pipeline.update();

    assert_eq!(10, intersection_pipeline.intersect_collider(og_collider).len(),);
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
            intersection_pipeline.modify_collider(new_id, new_collider);

            if og_collider.intersection_test(new_collider) {
                expected_result.push(new_id);
            }
        }

        expected_result.sort();

        intersection_pipeline.update();
        intersection_pipeline.update();

        let mut result = intersection_pipeline.intersect_collider(og_collider);
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
            intersection_pipeline.modify_collider(id, collider);
            id
        }));

        if used.is_empty() {
            continue;
        }

        match random::<u32>() % 20 == 0 {
            true => intersection_pipeline.update(),
            false => {
                let id = used.swap_remove_index(random::<usize>() % used.len()).unwrap();
                intersection_pipeline.remove_collider(id);
            }
        }
    }

    intersection_pipeline.update();
    intersection_pipeline.update();

    assert_eq!(used.len(), intersection_pipeline.snapshot.colliders.len());
}
