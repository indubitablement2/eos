use ahash::AHashSet;
use std::hash::Hash;

extern crate test;

#[derive(Debug, PartialEq, Eq)]
pub struct ArrayDifferenceSubAdd<T> {
    pub add: Vec<T>,
    pub sub: Vec<T>,
}
impl<T> ArrayDifferenceSubAdd<T> {
    /// Return if there is any difference.
    pub fn has_difference(&self) -> bool {
        !self.add.is_empty() || !self.sub.is_empty()
    }
}

/// Return what elements where removed/added between an old and a new array.
/// This is about 6 times faster, but require the arrays to be sorted.
pub fn sorted_arrays_sub_add<T>(old: &[T], new: &[T]) -> ArrayDifferenceSubAdd<T>
where
    T: Copy + Ord,
{
    let mut new_iter = new.iter();
    let mut old_iter = old.iter();

    let mut dif = ArrayDifferenceSubAdd {
        add: Vec::new(),
        sub: Vec::new(),
    };

    let mut n = match new_iter.next() {
        Some(v) => *v,
        None => {
            for rest in old_iter {
                dif.sub.push(*rest);
            }
            return dif;
        }
    };
    let mut o = match old_iter.next() {
        Some(v) => *v,
        None => {
            dif.add.push(n);
            for rest in new_iter {
                dif.add.push(*rest);
            }
            return dif;
        }
    };

    loop {
        if n > o {
            dif.sub.push(o);
            o = match old_iter.next() {
                Some(v) => *v,
                None => {
                    dif.add.push(n);
                    for rest in new_iter {
                        dif.add.push(*rest);
                    }
                    return dif;
                }
            };
        } else if n < o {
            dif.add.push(n);
            n = match new_iter.next() {
                Some(v) => *v,
                None => {
                    dif.sub.push(o);
                    for rest in old_iter {
                        dif.sub.push(*rest);
                    }
                    return dif;
                }
            };
        } else {
            n = match new_iter.next() {
                Some(v) => *v,
                None => {
                    for rest in old_iter {
                        dif.sub.push(*rest);
                    }
                    return dif;
                }
            };
            o = match old_iter.next() {
                Some(v) => *v,
                None => {
                    dif.add.push(n);
                    for rest in new_iter {
                        dif.add.push(*rest);
                    }
                    return dif;
                }
            };
        }
    }
}

/// Return what elements where removed/added between an old and a new array.
/// If the arrays are sorted, sorted_arrays_sub_add is about 6 times faster.
pub fn arrays_sub_add<T>(old: &[T], new: &[T]) -> ArrayDifferenceSubAdd<T>
where
    T: Copy + Ord + Hash,
{
    let hash_new: AHashSet<T> = new.iter().copied().collect();
    let hash_old: AHashSet<T> = old.iter().copied().collect();

    ArrayDifferenceSubAdd {
        add: new.iter().filter(|v| !hash_old.contains(*v)).copied().collect(),
        sub: old.iter().filter(|v| !hash_new.contains(*v)).copied().collect(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArrayDifferenceAdd<T> {
    /// If any difference was detected (not just addition).
    pub has_changed: bool,
    pub add: Vec<T>,
}

/// Return what elements where added between an old and a new array.
/// This is about 6 times faster, but require the arrays to be sorted.
pub fn sorted_arrays_add<T>(old: &[T], new: &[T]) -> ArrayDifferenceAdd<T>
where
    T: Copy + Ord,
{
    let mut new_iter = new.iter();
    let mut old_iter = old.iter();

    let mut dif = ArrayDifferenceAdd {
        has_changed: false,
        add: Vec::new(),
    };

    let mut n = match new_iter.next() {
        Some(v) => *v,
        None => {
            if old_iter.next().is_some() {
                dif.has_changed = true;
            }
            return dif;
        }
    };
    let mut o = match old_iter.next() {
        Some(v) => *v,
        None => {
            dif.add.push(n);
            dif.has_changed = true;
            for rest in new_iter {
                dif.add.push(*rest);
            }
            return dif;
        }
    };

    loop {
        if n > o {
            dif.has_changed = true;
            o = match old_iter.next() {
                Some(v) => *v,
                None => {
                    dif.add.push(n);
                    for rest in new_iter {
                        dif.add.push(*rest);
                    }
                    return dif;
                }
            };
        } else if n < o {
            dif.add.push(n);
            dif.has_changed = true;
            n = match new_iter.next() {
                Some(v) => *v,
                None => {
                    return dif;
                }
            };
        } else {
            n = match new_iter.next() {
                Some(v) => *v,
                None => {
                    if old_iter.next().is_some() {
                        dif.has_changed = true;
                    }
                    return dif;
                }
            };
            o = match old_iter.next() {
                Some(v) => *v,
                None => {
                    dif.add.push(n);
                    dif.has_changed = true;
                    for rest in new_iter {
                        dif.add.push(*rest);
                    }
                    return dif;
                }
            };
        }
    }
}

/// Return what elements where removed/added between an old and a new array.
/// If the arrays are sorted, sorted_arrays_sub_add is about 6+ times faster.
pub fn arrays_add<T>(old: &[T], new: &[T]) -> ArrayDifferenceAdd<T>
where
    T: Copy + Ord + Hash,
{
    let hash_new: AHashSet<T> = new.iter().copied().collect();
    let hash_old: AHashSet<T> = old.iter().copied().collect();
    let add: Vec<T> = hash_new.difference(&hash_old).copied().collect();

    ArrayDifferenceAdd {
        has_changed: hash_new != hash_old,
        add,
    }
}

#[test]
fn test_arrays_sub_add() {
    use rand::random;

    for _ in 0..1000 {
        let mut new = (0..random::<u32>() % 20)
            .into_iter()
            .map(|_| random::<u32>() % 20)
            .collect::<Vec<u32>>();
        new.sort_unstable();
        new.dedup();
        let mut old = (0..random::<u32>() % 20)
            .into_iter()
            .map(|_| random::<u32>() % 20)
            .collect::<Vec<u32>>();
        old.sort_unstable();
        old.dedup();

        let answer = arrays_sub_add(&old, &new);

        let result = sorted_arrays_sub_add(&old, &new);

        assert_eq!(answer, result, "\nold: {:?}\nnew: {:?}", old, new);
    }
}

#[bench]
fn bench_arrays_sub_add(b: &mut test::Bencher) {
    use rand::random;

    let news: Vec<Vec<u32>> = (0..100)
        .into_iter()
        .map(|_| {
            let mut new = (0..random::<u32>() % 20)
                .into_iter()
                .map(|_| random::<u32>() % 20)
                .collect::<Vec<u32>>();
            new.sort_unstable();
            new.dedup();
            new
        })
        .collect();
    let mut news_iter = news.iter().cycle();

    let olds: Vec<Vec<u32>> = (0..100)
        .into_iter()
        .map(|_| {
            let mut old = (0..random::<u32>() % 20)
                .into_iter()
                .map(|_| random::<u32>() % 20)
                .collect::<Vec<u32>>();
            old.sort_unstable();
            old.dedup();
            old
        })
        .collect();
    let mut olds_iter = olds.iter().cycle();

    b.iter(|| {
        test::black_box(arrays_sub_add(olds_iter.next().unwrap(), &news_iter.next().unwrap()));
    });
}

#[bench]
fn bench_sorted_arrays_sub_add(b: &mut test::Bencher) {
    use rand::random;

    let news: Vec<Vec<u32>> = (0..100)
        .into_iter()
        .map(|_| {
            let mut new = (0..random::<u32>() % 20)
                .into_iter()
                .map(|_| random::<u32>() % 20)
                .collect::<Vec<u32>>();
            new.sort_unstable();
            new.dedup();
            new
        })
        .collect();
    let mut news_iter = news.iter().cycle();

    let olds: Vec<Vec<u32>> = (0..100)
        .into_iter()
        .map(|_| {
            let mut old = (0..random::<u32>() % 20)
                .into_iter()
                .map(|_| random::<u32>() % 20)
                .collect::<Vec<u32>>();
            old.sort_unstable();
            old.dedup();
            old
        })
        .collect();
    let mut olds_iter = olds.iter().cycle();

    b.iter(|| {
        test::black_box(sorted_arrays_sub_add(
            olds_iter.next().unwrap(),
            &news_iter.next().unwrap(),
        ));
    });
}

#[test]
fn test_arrays_add() {
    use rand::random;

    for _ in 0..1000 {
        let mut new = (0..random::<u32>() % 20)
            .into_iter()
            .map(|_| random::<u32>() % 20)
            .collect::<Vec<u32>>();
        new.sort_unstable();
        new.dedup();
        let mut old = (0..random::<u32>() % 20)
            .into_iter()
            .map(|_| random::<u32>() % 20)
            .collect::<Vec<u32>>();
        old.sort_unstable();
        old.dedup();

        let mut answer = arrays_add(&old, &new);
        answer.add.sort();

        let result = sorted_arrays_add(&old, &new);

        assert_eq!(answer, result, "\nold: {:?}\nnew: {:?}", old, new);
        assert_eq!(new != old, result.has_changed, "\nold: {:?}\nnew: {:?}", old, new);
    }
}
