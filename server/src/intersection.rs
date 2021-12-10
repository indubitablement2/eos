use ahash::{AHashMap, AHashSet};
use common::collider::Collider;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use crossbeam_queue::SegQueue;
use glam::Vec2;
use indexmap::IndexMap;
use std::{
    cmp::Ordering,
    fmt::Debug,
    sync::{atomic::AtomicU32, Arc},
    thread::spawn,
};

/// Recycled after a collider is removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColliderId(pub u32);

#[derive(Debug)]
struct ColliderIdDispenser {
    last: AtomicU32,
    available: SegQueue<ColliderId>,
}
impl ColliderIdDispenser {
    fn new() -> Self {
        Self {
            last: AtomicU32::new(0),
            available: SegQueue::new(),
        }
    }

    fn new_collider_id(&self) -> ColliderId {
        self.available
            .pop()
            .unwrap_or_else(|| ColliderId(self.last.fetch_add(1, std::sync::atomic::Ordering::Relaxed)))
    }

    fn recycle_collider_id(&self, collider_id: ColliderId) {
        self.available.push(collider_id);
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
    collider_custom_data: AHashMap<ColliderId, u64>,
    /// The difference between each row's start and end can not be smaller than this.
    /// Sorted on the x axis.
    rows: Vec<SAPRow>,

    insert_collider_receiver: Receiver<(ColliderId, Collider, u64)>,
    modify_collider_receiver: Receiver<(ColliderId, Collider)>,
    remove_collider_receiver: Receiver<ColliderId>,
    /// Sometime remove may be called before insert when both are called at the same time.
    /// This prevent that by waiting an extra update before removing colliders.
    remove_queue: Vec<ColliderId>,
    /// Collider id that have just been removed on the runner may still be in use on the snapshot.
    /// This prevent that by waiting an extra update before recycling collider id.
    free_collider_id: Vec<ColliderId>,
}
impl AccelerationStructureRunner {
    const MIN_COLLIDER_PER_ROW: usize = 8;
    const MIN_ROW_SIZE: f32 = 512.0;

    fn new(
        remove_collider_receiver: Receiver<ColliderId>,
        insert_collider_receiver: Receiver<(ColliderId, Collider, u64)>,
        modify_collider_receiver: Receiver<(ColliderId, Collider)>,
    ) -> Self {
        Self {
            colliders: IndexMap::new(),
            collider_custom_data: AHashMap::new(),
            rows: Vec::new(),
            insert_collider_receiver,
            modify_collider_receiver,
            remove_collider_receiver,
            remove_queue: Vec::new(),
            free_collider_id: Vec::new(),
        }
    }

    fn update(&mut self, collider_id_dispenser: &Arc<ColliderIdDispenser>) {
        // Recycle collider id.
        for collider_id in self.free_collider_id.drain(..) {
            collider_id_dispenser.recycle_collider_id(collider_id);
        }
        // Insert new colliders.
        while let Ok((collider_id, collider, custom_data)) = self.insert_collider_receiver.try_recv() {
            self.colliders.insert(collider_id, collider);
            self.collider_custom_data.insert(collider_id, custom_data);
        }
        // Modify colliders.
        while let Ok((collider_id, new_collider)) = self.modify_collider_receiver.try_recv() {
            if let Some(collider) = self.colliders.get_mut(&collider_id) {
                *collider = new_collider;
            }
        }
        // Try to remove collider id that could not be found last update again.
        for collider_id in self.remove_queue.drain(..) {
            if self.colliders.remove(&collider_id).is_some() {
                self.collider_custom_data.remove(&collider_id);
                self.free_collider_id.push(collider_id);
            } else {
                warn!("A collider id could not be removed. That could mean a memory leak. Ignoring...");
            }
        }
        // Remove colliders and recycle collider id.
        while let Ok(collider_id) = self.remove_collider_receiver.try_recv() {
            if self.colliders.remove(&collider_id).is_some() {
                self.collider_custom_data.remove(&collider_id);
                self.free_collider_id.push(collider_id);
            } else {
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
        self.colliders
            .sort_by(|_, v1, _, v2| v1.position.y.partial_cmp(&v2.position.y).unwrap_or(Ordering::Equal));

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
    collider_custom_data: AHashMap<ColliderId, u64>,
    rows: Vec<SAPRow>,
}
impl AccelerationStructureSnapshot {
    fn new() -> Self {
        Self {
            colliders: IndexMap::new(),
            collider_custom_data: AHashMap::new(),
            rows: Vec::new(),
        }
    }

    // Update snapshot with the data of a runner.
    fn clone_from_runner(&mut self, runner: &AccelerationStructureRunner) {
        self.colliders.clone_from(&runner.colliders);
        self.collider_custom_data.clone_from(&runner.collider_custom_data);
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
            if row.start > top {
                break;
            }
            // The collider overlap this row.

            let closest = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < collider.position.x);

            // The furthest we should look in each dirrections.
            let threshold = collider.radius + row.biggest_radius;

            // Look to the left.
            let mut left = closest.saturating_sub(1);
            while let Some(i) = row.data.get(left) {
                let other = self.colliders[*i as usize];
                if collider.position.x - other.position.x > threshold {
                    break;
                }
                to_test.insert(*i);

                if left == 0 {
                    break;
                } else {
                    left -= 1;
                }
            }
            // Look to the right.
            let mut right = closest;
            while let Some(i) = row.data.get(right) {
                let other = self.colliders[*i as usize];
                if other.position.x - collider.position.x > threshold {
                    break;
                }
                to_test.insert(*i);

                right += 1;
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
    fn intersect_collider(&self, collider: Collider) -> Vec<ColliderId> {
        let mut to_test = AHashSet::with_capacity(16);
        let bottom = collider.position.y - collider.radius;
        let top = collider.position.y + collider.radius;
        let first_overlapping_row = self.rows.partition_point(|row| row.end < bottom);
        for row in &self.rows[first_overlapping_row..] {
            if row.start > top {
                break;
            }
            // The collider overlap this row.

            let closest = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < collider.position.x);

            // The furthest we should look in each dirrections.
            let threshold = collider.radius + row.biggest_radius;

            // Look to the left.
            let mut left = closest.saturating_sub(1);
            while let Some(i) = row.data.get(left) {
                let other = self.colliders[*i as usize];
                if collider.position.x - other.position.x > threshold {
                    break;
                }
                to_test.insert(*i);

                if left == 0 {
                    break;
                } else {
                    left -= 1;
                }
            }
            // Look to the right.
            let mut right = closest;
            while let Some(i) = row.data.get(right) {
                let other = self.colliders[*i as usize];
                if other.position.x - collider.position.x > threshold {
                    break;
                }
                to_test.insert(*i);

                right += 1;
            }
        }

        // Test each Collider we have collected.
        let mut result = Vec::with_capacity(to_test.len());
        for i in to_test.into_iter() {
            if let Some((collider_id, other)) = self.colliders.get_index(i as usize) {
                if collider.intersection_test(*other) {
                    result.push(*collider_id);
                }
            }
        }

        result
    }

    /// Test if any collider intersect with the provided point.
    fn test_point(&self, point: Vec2) -> bool {
        let mut to_test = AHashSet::with_capacity(16);
        let overlapping_row = self.rows.partition_point(|row| row.end < point.y);
        if let Some(row) = self.rows.get(overlapping_row) {
            // The closest collider to this point.
            let closest = row
                .data
                .partition_point(|i| self.colliders[*i as usize].position.x < point.x);

            // The furthest we should look in each dirrections.
            let threshold = row.biggest_radius;

            // Look to the left.
            let mut left = closest.saturating_sub(1);
            while let Some(i) = row.data.get(left) {
                let other = self.colliders[*i as usize];
                if point.x - other.position.x > threshold {
                    break;
                }
                to_test.insert(*i);

                if left == 0 {
                    break;
                } else {
                    left -= 1;
                }
            }
            // Look to the right.
            let mut right = closest;
            while let Some(i) = row.data.get(right) {
                let other = self.colliders[*i as usize];
                if other.position.x - point.x > threshold {
                    break;
                }
                to_test.insert(*i);

                right += 1;
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

/// Allow fast circle-circle intersection and test between colliders.
/// This intersection pipeline is fully async, but there is a delay before commands take effect.
#[derive(Debug)]
pub struct IntersectionPipeline {
    collider_id_dispenser: Arc<ColliderIdDispenser>,

    update_request_sender: Sender<AccelerationStructureRunner>,
    update_result_receiver: Receiver<AccelerationStructureRunner>,

    remove_collider_sender: Sender<ColliderId>,
    modify_collider_sender: Sender<(ColliderId, Collider)>,
    insert_collider_sender: Sender<(ColliderId, Collider, u64)>,

    snapshot: AccelerationStructureSnapshot,
}
impl IntersectionPipeline {
    pub fn new() -> Self {
        let (update_request_sender, update_request_receiver) = bounded(0);
        let (update_result_sender, update_result_receiver) = bounded(0);

        let (remove_collider_sender, remove_collider_receiver) = unbounded();
        let (modify_collider_sender, modify_collider_receiver) = unbounded();
        let (insert_collider_sender, insert_collider_receiver) = unbounded();

        let collider_id_dispenser = Arc::new(ColliderIdDispenser::new());
        let collider_id_dispenser_clone = collider_id_dispenser.clone();

        spawn(move || {
            runner_loop(
                update_request_receiver,
                update_result_sender,
                collider_id_dispenser_clone,
            )
        });

        let runner = AccelerationStructureRunner::new(
            remove_collider_receiver,
            insert_collider_receiver,
            modify_collider_receiver,
        );

        update_request_sender
            .send(runner)
            .expect("Could not send runner to thread.");

        Self {
            collider_id_dispenser,
            update_request_sender,
            update_result_receiver,
            remove_collider_sender,
            modify_collider_sender,
            insert_collider_sender,
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

    /// Insert a collider with custom data.
    pub fn insert_collider(&self, collider: Collider, custom_data: u64) -> ColliderId {
        let collider_id = self.collider_id_dispenser.new_collider_id();
        let _ = self.insert_collider_sender.send((collider_id, collider, custom_data));
        collider_id
    }

    /// Modify a collider.
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

    /// Get a copy of a collider's custom data by its id.
    pub fn get_collider_custom_data(&self, collider_id: ColliderId) -> Option<u64> {
        self.snapshot
            .collider_custom_data
            .get(&collider_id)
            .map(|custom_data| custom_data.to_owned())
    }

    /// Return all colliders that intersect the provided collider.
    pub fn intersect_collider(&self, collider: Collider) -> Vec<ColliderId> {
        self.snapshot.intersect_collider(collider)
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
    collider_id_dispenser: Arc<ColliderIdDispenser>,
) {
    while let Ok(mut runner) = update_request_receiver.recv() {
        runner.update(&collider_id_dispenser);
        if update_result_sender.send(runner).is_err() {
            info!("Intersection pipeline update runner thread dropped.");
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
    let first_collider_id = intersection_pipeline.insert_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
        },
        0,
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

    // Collider id are recycled.
    assert_eq!(
        intersection_pipeline.collider_id_dispenser.new_collider_id(),
        ColliderId(0)
    );
}

#[test]
fn test_row() {
    use glam::vec2;

    let mut intersection_pipeline = IntersectionPipeline::new();

    intersection_pipeline.insert_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 0.0),
        },
        0,
    );
    intersection_pipeline.update();
    intersection_pipeline.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 1);

    // Do we have 2 rows?
    for _ in 0..AccelerationStructureRunner::MIN_COLLIDER_PER_ROW {
        intersection_pipeline.insert_collider(
            Collider {
                radius: 10.0,
                position: vec2(0.0, 10000.0),
            },
            0,
        );
    }
    intersection_pipeline.update();
    intersection_pipeline.update();
    assert_eq!(intersection_pipeline.snapshot.rows.len(), 2);

    for _ in 0..AccelerationStructureRunner::MIN_COLLIDER_PER_ROW - 1 {
        intersection_pipeline.insert_collider(
            Collider {
                radius: 10.0,
                position: vec2(0.0, 5000.0),
            },
            0,
        );
    }
    let mid = intersection_pipeline.insert_collider(
        Collider {
            radius: 10.0,
            position: vec2(0.0, 5000.0),
        },
        0,
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

        intersection_pipeline.insert_collider(
            Collider {
                radius: random::<f32>() * 256.0,
                position: random::<Vec2>() * 512.0 - 256.0,
            },
            0,
        );
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

        intersection_pipeline.insert_collider(
            Collider {
                radius: random::<f32>() * 256.0,
                position: random::<Vec2>() * 512.0 - 256.0,
            },
            0,
        );
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

    for _ in 0..10 {
        let new_collider = Collider {
            radius: 10.0,
            position: Vec2::ZERO,
        };

        intersection_pipeline.insert_collider(new_collider, 0);
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
        for _ in 0..32 {
            let new_collider = Collider {
                radius: random::<f32>() * 128.0,
                position: (random::<Vec2>() * 512.0 - 256.0),
            };

            let new_id = intersection_pipeline.insert_collider(new_collider, 0);

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
fn test_reclycling_collider() {
    use indexmap::IndexSet;
    use rand::random;

    let mut intersection_pipeline = IntersectionPipeline::new();

    let collider = Collider {
        radius: 1.0,
        position: Vec2::ZERO,
    };

    let mut used = IndexSet::new();
    for _ in 0..1000 {
        assert!(used.insert({
            let id = intersection_pipeline.insert_collider(collider, 0);

            assert!(intersection_pipeline.snapshot.collider_custom_data.get(&id).is_none());

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
