fn main() {
    let buf1 = &[0u8; 10];
    let mut buf2 = vec![0u8; 10];
    let buf3: Vec<u8> = Vec::with_capacity(10);
    let slice = &buf2[0..10];

    println!("buf1 len: {:?}, capacity: {:?}", buf1.len(), buf1.len());
    println!("buf2 len: {:?}, capacity: {:?}", buf2.len(), buf2.capacity());
    println!("slice len: {:?}, capacity: {:?}", slice.len(), slice.len());

    buf2.clear();
    println!("buf2 (clear) len: {:?}, capacity: {:?}", buf2.len(), buf2.capacity());
    // This will panic.
    // let mut slice = &buf2[0..10];

    println!("buf3 len: {:?}, capacity: {:?}", buf3.len(), buf3.capacity());
}
