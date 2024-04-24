use super::*;

enum Bucket {
    Empty { next_empty: Option<u32> },
    Occupied { value: Hull },
}

/// Generational arena which prioritizes reusing index
/// that would serialize in the least bytes.
#[derive(Default)]
pub struct HullArena {
    /// < 128, < 16384, Rest
    heads: [Option<u32>; 3],
    data: Vec<(NonZeroU32, Bucket)>,
}
impl HullArena {
    fn find_step(index: u32) -> usize {
        if index < 128 {
            0
        } else if index < 16384 {
            1
        } else {
            2
        }
    }

    pub fn get_index(&self, index: u32) -> Option<&Hull> {
        let (_, bucket) = self.data.get(index as usize)?;
        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn get_index_mut(&mut self, index: u32) -> Option<&mut Hull> {
        let (_, bucket) = self.data.get_mut(index as usize)?;
        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn get(&self, key: HullId) -> Option<&Hull> {
        let (generation, bucket) = self.data.get(key.index as usize)?;

        if key.generation != *generation {
            return None;
        }

        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn get_mut(&mut self, key: HullId) -> Option<&mut Hull> {
        let (generation, bucket) = self.data.get_mut(key.index as usize)?;

        if key.generation != *generation {
            return None;
        }

        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn remove(&mut self, key: HullId) -> bool {
        let Some((generation, bucket)) = self.data.get_mut(key.index as usize) else {
            return false;
        };

        if key.generation != *generation {
            return false;
        }

        match bucket {
            Bucket::Empty { .. } => false,
            Bucket::Occupied { .. } => {
                *generation = generation.checked_add(1).unwrap_or(NonZeroU32::MIN);

                let step = Self::find_step(key.index);

                *bucket = Bucket::Empty {
                    next_empty: self.heads[step],
                };
                self.heads[step] = Some(key.index);

                true
            }
        }
    }

    pub fn insert(&mut self, value: Hull) -> (HullId, &mut Hull) {
        for head in self.heads.iter_mut() {
            if let Some(index) = head.take() {
                let (generation, bucket) = &mut self.data[index as usize];
                match bucket {
                    Bucket::Empty { next_empty } => {
                        *head = *next_empty;
                    }
                    Bucket::Occupied { .. } => unreachable!(),
                }
                *bucket = Bucket::Occupied { value };

                let value = match bucket {
                    Bucket::Empty { .. } => unreachable!(),
                    Bucket::Occupied { value } => value,
                };

                return (
                    HullId {
                        index,
                        generation: *generation,
                    },
                    value,
                );
            }
        }

        let index = self.data.len() as u32;
        self.heads[Self::find_step(index)] = Some(index);

        self.data
            .push((NonZeroU32::MIN, Bucket::Occupied { value }));

        let value = match &mut self.data.last_mut().unwrap().1 {
            Bucket::Empty { .. } => unreachable!(),
            Bucket::Occupied { value } => value,
        };

        return (
            HullId {
                index,
                generation: NonZeroU32::MIN,
            },
            value,
        );
    }

    pub fn leak_take_index(&mut self, index: u32) -> (HullId, Hull) {
        let (generation, bucket) = &mut self.data[index as usize];
        match std::mem::replace(bucket, Bucket::Empty { next_empty: None }) {
            Bucket::Empty { .. } => panic!(),
            Bucket::Occupied { value } => (
                HullId {
                    generation: *generation,
                    index,
                },
                value,
            ),
        }
    }

    pub fn unleak_remove_index(&mut self, index: u32) {
        let (generation, bucket) = &mut self.data[index as usize];
        *generation = generation.checked_add(1).unwrap_or(NonZeroU32::MIN);

        let step = Self::find_step(index);

        *bucket = Bucket::Empty {
            next_empty: self.heads[step],
        };
        self.heads[step] = Some(index);
    }

    pub fn unleak_set_index(&mut self, index: u32, value: Hull) {
        let (_, bucket) = &mut self.data[index as usize];
        *bucket = Bucket::Occupied { value };
    }
}
