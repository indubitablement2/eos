fn main() {
    let f = 1.1f32;
    let u = (f * u16::MAX as f32) as u16;
    println!("{}", u);

    println!("");

    let neg = -5.0f32;
    println!("{}", neg as u32);

    println!("");

    let arr = [0u8; 1];

    for u in &arr[1..] {
        println!("{}", u);
    }
}
