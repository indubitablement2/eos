fn main() {
    let f = 1.1f32;
    let u = (f * u16::MAX as f32) as u16;
    println!("{}", u);
}
