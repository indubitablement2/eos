// . . . .
// . 1 3 .
// . 2 4 .
// . . . .
fn main() {
    let vertices = [-0.5f32, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5];
    let buf: Vec<u8> = vertices.iter().flat_map(|v| v.to_ne_bytes()).collect();
    println!("{:?}", buf);
}
