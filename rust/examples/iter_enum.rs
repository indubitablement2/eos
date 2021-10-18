fn main() {
    let order = (0..8).into_iter().collect::<Vec<u8>>();

    order.iter().enumerate().rev().for_each(|(i, og)| {
        assert_eq!(order[i], *og);
    });
}
