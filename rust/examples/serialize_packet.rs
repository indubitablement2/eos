fn main() {
    let na_vec_generic = nalgebra::vector![123.123f32, 45.0f32, 11.0f32];
    let na_vec_generic_ser = bincode::serialize(&na_vec_generic).unwrap();

    println!("{:?}, {}\n", na_vec_generic_ser, na_vec_generic_ser.len());

    let gl_vec = glam::Vec2::new(123.123f32, 45.0f32);
    let gl_vec_ser = bincode::serialize(&gl_vec).unwrap();

    println!("{:?}, {}\n", gl_vec_ser, gl_vec_ser.len());
}
