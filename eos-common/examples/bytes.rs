use bytes::*;

fn main() {
    let mut bytes = BytesMut::with_capacity(1024);

    {
        // Put and remove i32.
        println!("put_i32");
        bytes.put_i32(-123);
        println!("{:?}\n", bytes);

        println!("get_i32");
        assert_eq!(-123, bytes.get_i32());
        println!("{:?}", bytes);
        println!("bytes is empty again\n");

        // Create slice 1 and 2.
        let mut slice1 = BytesMut::new();
        slice1.extend_from_slice(&[69u8; 100]);
        println!("creating slice1 {:?}", slice1);
        let mut slice2 = BytesMut::new();
        slice2.extend_from_slice(&[84u8; 42]);
        println!("creating slice2 {:?}\n", slice2);

        // Put slice1.
        println!("put_slice(slice1)");
        bytes.put_slice(&slice1);
        println!("{:?}\n", bytes);

        // Put f32.
        println!("put_f32");
        bytes.put_f32(-123.0);
        println!("{:?}\n", bytes);

        // Put slice2.
        println!("put_slice(slice2)");
        bytes.put_slice(&slice2);
        println!("{:?}\n", bytes);

        println!("put_T is always appended at the end.\n");

        // Recover slice1.
        println!("split_to(100) to recover slice1. split_to is at the front [0..at]");
        let slice_front1 = bytes.split_to(100);
        assert_eq!(slice_front1, slice1);
        println!("{:?}\n", bytes);

        // Put a new f64.
        println!("put_f64");
        bytes.put_f64(666.666);
        println!("{:?}\n", bytes);

        // Get f32.
        assert_eq!(-123.0, bytes.get_f32());
        println!("{:?}", bytes);
        println!("get_f32\n");

        println!("get_T and split_to always recover value at the front.\n");

        // Recover slice2.
        let slice_front2 = bytes.split_to(42);
        println!("{:?}", bytes);
        println!("split_to(42) to recover slice2.\n");

        // Get f64.
        assert_eq!(slice_front2, slice2);
        assert_eq!(666.666f64, bytes.get_f64());
        println!("{:?}", bytes);
        println!("get_f64\n");

        // Bytes is now empty.
        println!("bytes is now empty.");
        println!("{:?}\n", bytes);

        // Some of bytes's capacity has been lost.
        println!("Some of bytes's capacity has been lost.");
        println!("{:?}\n", bytes.capacity());
    }

    // Lets recover it by sending slice_front1 and slice_front2 out of scope and reserving.
    println!("if the slices we have taken with split_to go out of scope, capacity is still lost.");
    println!("{:?}\n", bytes.capacity());

    // Capacity is back to 1024.
    println!("we need to call reserve to recover lost capacity.");
    bytes.reserve(1024);
    println!("{:?}\n", bytes.capacity());

    // Bytes is still empty.
    println!("bytes is still empty.");
    println!("{:?}\n", bytes);
}
