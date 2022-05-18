use super::fleet::Fleet;
use dioptre::Field;
use soak::{RawTable, Columns};

#[derive(Debug, Clone, Copy)]
pub struct Entity<T> {
    index: u32,
    generation: u32,
    _marker: std::marker::PhantomData<T>,
}
impl<T> Entity<T> {
    pub fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: std::marker::PhantomData::default(),
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

pub trait Components: Sized + Columns {
    const GENERATION: Field<Self, u32>;
    fn generation(&self) -> u32;
    unsafe fn move_to_table(self, raw_table: &mut RawTable<Self>, index: usize);
    unsafe fn drop_from_table(raw_table: &mut RawTable<Self>, index: usize);
}

pub trait RecyclableRawTable<T: soak::Columns> {
    fn new(initial_capacity: usize) -> Self;
    // # Invariants:
    // Each element of T need to be manualy moved to the raw table.
    fn push(&mut self, components: T) -> Entity<T>;
    // # Invariants:
    // Drop need to be manualy called for element of T.
    fn remove(&mut self, entity: Entity<T>);
    /// Reserve space for count more elements.
    fn reserve(&mut self, count: usize);
    /// Similar to Vec's len, but may contain invalid object within len.
    /// Mark where no more object will be valid.
    fn end(&self) -> usize;
}

pub struct Fleets {
    pub raw_table: RawTable<Fleet>,
    end: usize,
    removed: std::collections::VecDeque<usize>,
}
impl RecyclableRawTable<Fleet> for Fleets {
    fn new(initial_capacity: usize) -> Self {
        Self {
            raw_table: RawTable::with_capacity(initial_capacity),
            end: 0,
            removed: (0..initial_capacity).collect(),
        }
    }

    fn push(&mut self, components: Fleet) -> Entity<Fleet> {
        // Get a free index.
        let index = self.removed.pop_front().unwrap_or_else(|| {
            self.reserve(self.raw_table.capacity());
            self.removed.pop_front().expect("oom")
        });

        self.end = self.end.max(index + 1);

        let entity = Entity::new(index as u32, components.generation);

        // Move componments.
        unsafe {
            self.raw_table
                .ptr(Fleet::detected_radius)
                .add(index)
                .write(components.detected_radius);
            self.raw_table
                .ptr(Fleet::detector_radius)
                .add(index)
                .write(components.detector_radius);
            self.raw_table
                .ptr(Fleet::faction_id)
                .add(index)
                .write(components.faction_id);
            self.raw_table
                .ptr(Fleet::fleet_detected)
                .add(index)
                .write(components.fleet_detected);
            self.raw_table
                .ptr(Fleet::fleet_id)
                .add(index)
                .write(components.fleet_id);
            self.raw_table
                .ptr(Fleet::generation)
                .add(index)
                .write(components.generation);
            self.raw_table
                .ptr(Fleet::idle_counter)
                .add(index)
                .write(components.idle_counter);
            self.raw_table
                .ptr(Fleet::in_system)
                .add(index)
                .write(components.in_system);
            self.raw_table
                .ptr(Fleet::orbit)
                .add(index)
                .write(components.orbit);
            self.raw_table
                .ptr(Fleet::position)
                .add(index)
                .write(components.position);
            self.raw_table
                .ptr(Fleet::radius)
                .add(index)
                .write(components.radius);
            self.raw_table
                .ptr(Fleet::velocity)
                .add(index)
                .write(components.velocity);
            self.raw_table
                .ptr(Fleet::wish_position)
                .add(index)
                .write(components.wish_position);
        }

        entity
    }

    fn remove(&mut self, entity: Entity<Fleet>) {
        let mut index = entity.index() as usize;

        if index >= self.end {
            // Index was already removed or is oob.
            log::debug!("Requested to remove a fleet with index >= end.");
            return;
        }

        unsafe {
            let start = self.raw_table.ptr(Fleet::generation);

            // Mark the fleet as invalid.
            let cur_gen_ptr = start.add(index);
            if cur_gen_ptr.read() > entity.generation() {
                // Fleet in table is newer.
                log::debug!("Requested to remove fleet, but it has been replaced already.");
                return;
            }
            cur_gen_ptr.write(u32::MAX);

            // Drop components that aren't copy.
            self.raw_table.ptr(Fleet::fleet_detected).drop_in_place();

            // We keep lower index at the front to avoid too many hole in the raw table.
            if index * 2 > self.end {
                self.removed.push_back(index);
            } else {
                self.removed.push_front(index);
            }

            // Recalculate the end if it changed.
            if index + 1 == self.end {
                while index > 0 {
                    index -= 1;
                    if *start.add(index) == u32::MAX {
                        break;
                    }
                }
                self.end = index + 1;
            }
        }
    }

    fn reserve(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.removed
            .extend(self.raw_table.capacity()..self.raw_table.capacity() + count);
        self.raw_table
            .reserve_exact(self.raw_table.capacity(), count);
    }

    fn end(&self) -> usize {
        self.end
    }
}

#[test]
fn test_fleets() {
    use super::fleet::FleetBuilder;
    use common::idx::*;

    let num = 999;

    let mut f = Fleets::new(512);
    let entities: Vec<Entity<Fleet>> = (0..num as u64)
        .map(|i| f.push(FleetBuilder::new(1, FleetId(i), FactionId(i), glam::Vec2::ZERO).build()))
        .collect();

    let mut expected_removed = 1024 - num;

    assert_eq!(f.end(), num);
    assert_eq!(f.raw_table.capacity(), 1024);

    // Try to remove invalid entities.
    f.remove(Entity::new(0, 0));
    assert_eq!(f.removed.len(), expected_removed);
    f.remove(Entity::new(1024, 1));
    assert_eq!(f.removed.len(), expected_removed);

    for entity in entities.into_iter() {
        f.remove(entity);
        expected_removed += 1;
        assert_eq!(f.removed.len(), expected_removed);

        // TODO: Check that raw table generation are ok.
    }
}

pub struct RecyclableRawTableT<T: soak::Columns + Components> {
    raw_table: RawTable<T>,
    end: usize,
    removed: std::collections::VecDeque<usize>,
    /// - 0: Components to be added to the table.
    /// - 1: Capacity needed.
    queue_add: (Vec<(T, usize)>, usize),
    queue_remove: Vec<Entity<T>>,
}
impl<T: soak::Columns + Components> RecyclableRawTableT<T> {
    const MIN_ALOCATION: usize = 512;

    pub fn new() -> Self {
        Self {
            raw_table: RawTable::with_capacity(Self::MIN_ALOCATION),
            end: 0,
            removed: (0..Self::MIN_ALOCATION).collect(),
            queue_add: (Vec::new(), Self::MIN_ALOCATION),
            queue_remove: Vec::new(),
        }
    }

    pub fn queue_add(&mut self,  components: T) -> Entity<T> {
        let generation = components.generation();
        
        // Get an index.
        let index = if let Some(index) = self.removed.pop_front() {
            index
        } else {
            let index = self.queue_add.1;
            self.queue_add.1 += 1;
            index
        };

        self.queue_add.0.push((components, index));
        Entity::new(index as u32, generation)
    }

    pub fn queue_remove(&mut self, entity: Entity<T>) {
        self.queue_remove.push(entity);
    }

    pub fn handle_queues(&mut self) {
        // Remove entities.
        let start = self.raw_table.ptr(T::GENERATION);
        for entity in self.queue_remove.drain(..) {
            let mut index = entity.index() as usize;

            if index >= self.end {
                // Index was already removed or is oob.
                log::debug!("Requested to remove an entity with index >= end.");
                continue;
            }

            unsafe {
                // Mark the fleet as invalid.
                let cur_gen_ptr = start.add(index);
                if cur_gen_ptr.read() > entity.generation() {
                    // Entity in table is newer.
                    log::debug!("Requested to remove entity, but it has been replaced already.");
                    return;
                }
                cur_gen_ptr.write(u32::MAX);
    
                // Drop components that aren't copy.
                T::drop_from_table(&mut self.raw_table, index);
    
                // We keep lower index at the front to avoid too many hole in the raw table.
                if index * 2 > self.end {
                    self.removed.push_back(index);
                } else {
                    self.removed.push_front(index);
                }
            }
        }

        // Recalculate the end if it changed.
        if self.end > 0 {
            unsafe {
                let mut index = self.end - 1;
                while index > 0 {
                    index -= 1;
                    if *start.add(index) != u32::MAX {
                        break;
                    }
                }
                self.end = index + 1;
            }
        }

        // Alocate space for the new entities.
        if self.queue_add.1 > self.raw_table.capacity() {
            self.reserve(self.queue_add.1 - self.raw_table.capacity());
        }

        // Add entities.
        for (components, index) in self.queue_add.0.drain(..) {
            // Move componments.
            unsafe {
                components.move_to_table(&mut self.raw_table, index);
            }

            self.end = self.end.max(index + 1);
        }
    }

    pub fn reserve(&mut self, mut count: usize) {
        if count == 0 {
            return;
        }

        // Never alocate less than the current capacity or MIN_ALOCATION.
        count = self.raw_table.capacity().max(Self::MIN_ALOCATION).max(count);

        self.removed
            .extend(self.raw_table.capacity()..self.raw_table.capacity() + count);
        self.raw_table
            .reserve_exact(self.raw_table.capacity(), count);
        self.queue_add.1 = self.raw_table.capacity();
    }

    fn end(&self) -> usize {
        self.end
    }
}