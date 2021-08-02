fn main() {
    // 8 * 8 chunk.
    // 4 *4 update chunk.
    let mut v = vec![2u8; 64];

    let mut v_ptr: Vec<*mut u8> = Vec::with_capacity(64);

    for i in 0..64usize {
        v_ptr.push(&mut v[i] as *mut u8);
    }

    println!("{:?}", v);

    for i in 0..16usize {
        let id = i * 4;

        unsafe {
            *v_ptr[id] += *v_ptr[id + 1];
        }
    }

    println!("{:?}", v);
}
