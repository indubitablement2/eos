use rapier2d::na;

fn main() {
    let na_vec_generic = na::vector![1.0f32, 2.0f32, 3.0f32];
    let na_vec_generic_ser = bincode::serialize(&na_vec_generic).unwrap();
    println!("{:?}, {}\n", na_vec_generic_ser, na_vec_generic_ser.len());

    let na_vec: na::Vector3<f32> = na::vector![1.0f32, 2.0f32, 3.0f32];
    let na_vec_ser = bincode::serialize(&na_vec).unwrap();
    println!("{:?}, {}\n", na_vec_ser, na_vec_ser.len());

    let gl_vec = glam::Vec3::new(1.0f32, 2.0f32, 3.0f32);
    let gl_vec_ser = bincode::serialize(&gl_vec).unwrap();
    println!("{:?}, {}\n", gl_vec_ser, gl_vec_ser.len());
}
