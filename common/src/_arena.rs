use std::{
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU64},
};

enum Bucket<V> {
    Empty { next_empty: Option<u32> },
    Occupied { value: V },
}

/// Generational arena which prioritizes reusing index
/// that would serialize in the least bytes.
pub struct Arena<K, V> {
    /// < 128, < 16384, Rest
    heads: [Option<u32>; 3],
    data: Vec<(NonZeroU32, Bucket<V>)>,
    spooky: PhantomData<K>,
}
impl<K: Into<NonZeroU64> + From<NonZeroU64> + Copy + Sized, V> Arena<K, V> {
    const STEPS: [u32; 3] = [127, 16383, u32::MAX];

    fn key_new(index: u32, generation: NonZeroU32) -> K {
        K::from(NonZeroU64::new((index as u64) << 32 | generation.get() as u64).unwrap())
    }

    fn key_generation(key: K) -> NonZeroU32 {
        let u: NonZeroU64 = key.into();
        NonZeroU32::new(u.get() as u32).unwrap()
    }

    fn key_index(key: K) -> usize {
        let u: NonZeroU64 = key.into();
        (u.get() >> 32) as usize
    }

    fn find_step(index: u32) -> usize {
        if index < Self::STEPS[0] {
            return 0;
        } else if index < Self::STEPS[1] {
            return 1;
        } else {
            return 2;
        }
    }

    pub fn get_index(&self, index: usize) -> Option<&V> {
        let (_, bucket) = self.data.get(index)?;
        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn get_mut_index(&mut self, index: usize) -> Option<&mut V> {
        let (_, bucket) = self.data.get_mut(index)?;
        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let (generation, bucket) = self.data.get(Self::key_index(key))?;

        if Self::key_generation(key) != *generation {
            return None;
        }

        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let (generation, bucket) = self.data.get_mut(Self::key_index(key))?;

        if Self::key_generation(key) != *generation {
            return None;
        }

        match bucket {
            Bucket::Empty { .. } => None,
            Bucket::Occupied { value } => Some(value),
        }
    }

    pub fn remove(&mut self, key: K) -> bool {
        let index = Self::key_index(key);
        let Some((generation, bucket)) = self.data.get_mut(index) else {
            return false;
        };

        if Self::key_generation(key) != *generation {
            return false;
        }

        match bucket {
            Bucket::Empty { .. } => false,
            Bucket::Occupied { .. } => {
                let step = Self::find_step(index as u32);

                *bucket = Bucket::Empty {
                    next_empty: self.heads[step],
                };
                self.heads[step] = Some(index as u32);

                true
            }
        }
    }

    pub fn insert(&mut self, value: V) -> K {
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
                return K::from(
                    NonZeroU64::new((index as u64) << 32 | generation.get() as u64).unwrap(),
                );
            }
        }

        let index = self.data.len() as u32;
        self.heads[Self::find_step(index)] = Some(index);

        let key = K::from(NonZeroU64::new((index as u64) << 32 | 1).unwrap());
        self.data
            .push((NonZeroU32::MIN, Bucket::Occupied { value }));

        return key;
    }

    pub fn leak_take_index(&mut self, index: usize) -> (K, V) {
        let (generation, bucket) = &mut self.data[index];
        match std::mem::replace(bucket, Bucket::Empty { next_empty: None }) {
            Bucket::Empty { .. } => panic!(),
            Bucket::Occupied { value } => (Self::key_new(index as u32, *generation), value),
        }
    }

    pub fn unleak_remove_index(&mut self, index: usize) {
        let (generation, bucket) = &mut self.data[index];
        *generation = generation.checked_add(1).unwrap_or(NonZeroU32::MIN);

        let step = Self::find_step(index as u32);

        *bucket = Bucket::Empty {
            next_empty: self.heads[step],
        };
        self.heads[step] = Some(index as u32);
    }

    pub fn unleak_set_index(&mut self, index: usize, value: V) {
        let (_, bucket) = &mut self.data[index];
        *bucket = Bucket::Occupied { value };
    }
}

impl<K, V> Default for Arena<K, V> {
    fn default() -> Self {
        Self {
            heads: Default::default(),
            data: Default::default(),
            spooky: Default::default(),
        }
    }
}
