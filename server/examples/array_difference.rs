use rand::random;
use std::collections::HashSet;

fn main() {
    for _ in 0..1000 {
        test()
    }
}

fn test() {
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

    let answer: HashSet<u32> = new.iter().chain(old.iter()).copied().collect();

    let (add, idem, sub) = find_difference(&new, &old);

    assert_eq!(
        answer.len(),
        add.len() + idem.len() + sub.len(),
        "\nnew: {:?}\nold:{:?}\nadd: {:?}\nidem: {:?}\nsub: {:?}\n",
        new,
        old,
        add,
        idem,
        sub
    );
}

fn find_difference(new: &[u32], old: &[u32]) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut new_iter = new.iter();
    let mut old_iter = old.iter();

    let mut add = Vec::new();
    let mut idem = Vec::new();
    let mut sub = Vec::new();

    let mut n = match new_iter.next() {
        Some(v) => *v,
        None => {
            for rest in old_iter {
                sub.push(*rest);
            }
            return (add, idem, sub);
        }
    };
    let mut o = match old_iter.next() {
        Some(v) => *v,
        None => {
            add.push(n);
            for rest in new_iter {
                add.push(*rest);
            }
            return (add, idem, sub);
        }
    };

    loop {
        if n > o {
            sub.push(o);
            o = match old_iter.next() {
                Some(v) => *v,
                None => {
                    add.push(n);
                    for rest in new_iter {
                        add.push(*rest);
                    }
                    return (add, idem, sub);
                }
            };
        } else if n < o {
            add.push(n);
            n = match new_iter.next() {
                Some(v) => *v,
                None => {
                    sub.push(o);
                    for rest in old_iter {
                        sub.push(*rest);
                    }
                    return (add, idem, sub);
                }
            };
        } else {
            idem.push(n);
            n = match new_iter.next() {
                Some(v) => *v,
                None => {
                    for rest in old_iter {
                        sub.push(*rest);
                    }
                    return (add, idem, sub);
                }
            };
            o = match old_iter.next() {
                Some(v) => *v,
                None => {
                    add.push(n);
                    for rest in new_iter {
                        add.push(*rest);
                    }
                    return (add, idem, sub);
                }
            };
        }
    }
}
