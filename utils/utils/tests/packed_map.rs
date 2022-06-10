use utils::*;

#[test]
fn test_packet_map() {
    let mut v = vec![0, 1, 2, 3];
    let mut p: PackedMap<Vec<i32>, usize> = PackedMap::new();
    p.extend(v.iter().copied().enumerate());
    assert_eq!(&v, &p.container);

    // Push
    v.push(4);
    p.insert(4, 4);
    assert_eq!(&v, &p.container);
    assert_eq!(v.len(), p.len());

    // Pop
    assert_eq!(v.pop().unwrap(), p.pop().unwrap().0);
    assert_eq!(v.len(), p.len());

    // Swap
    v.swap(1, 3);
    p.swap_by_index(1, 3);
    assert_eq!(&v, &p.container);

    // Swap remove index
    assert_eq!(v.swap_remove(1), p.swap_remove_by_index(1).0);
    assert_eq!(v.len(), p.len());

    // Sort
    v.sort_unstable();
    p.sort_unstable_by(|s, a, b| s.container[*a].cmp(&s.container[*b]));
    assert_eq!(&v, &p.container);

    // Swap remove id
    let removed = p.swap_remove_by_id(0).unwrap();
    assert_eq!(v.swap_remove(removed.1), removed.0);
    assert_eq!(&v, &p.container);
    assert_eq!(v.len(), p.len());
}
