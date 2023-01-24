fn main() {
    for i in 1..10 {
        println!("{}, {}", -10i32.rem_euclid(i), i);
    }
}
