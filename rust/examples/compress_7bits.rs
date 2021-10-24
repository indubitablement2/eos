fn main() {
    // Result:
    // rrrrrrra
    // ggggggga
    // bbbbbbba
    // aaaaiiii

    let r = 1.0f32;
    let g = 1.0f32;
    let b = 1.0f32;
    let a = 0.0f32;
    let id = 0b1110u8;

    let mut bits: [u8; 4] = [255, 255, 255, 0b11110000];

    // Convert f32 to u8.
    unsafe {
        bits[0] = (r * 127.0).to_int_unchecked();
        println!("{}", bits[0]);
        bits[1] = (g * 127.0).to_int_unchecked();
        println!("{}", bits[1]);
        bits[2] = (b * 127.0).to_int_unchecked();
        println!("{}", bits[2]);
        bits[3] = (a * 127.0).to_int_unchecked();
        println!("{}\n", bits[3]);
    }

    // 0rrrrrrr
    // 0ggggggg
    // 0bbbbbbb
    // 0aaaaaaa
    println!("{:08b}\n{:08b}\n{:08b}\n{:08b}\n", bits[0], bits[1], bits[2], bits[3]);

    // Shift bits to the left.
    bits[0] <<= 1;
    bits[1] <<= 1;
    bits[2] <<= 1;

    // rrrrrrr0
    // ggggggg0
    // bbbbbbb0
    // 0aaaaaaa
    println!("{:08b}\n{:08b}\n{:08b}\n{:08b}\n", bits[0], bits[1], bits[2], bits[3]);

    // Separate alpha bits.
    bits[0] += bits[3] % 2;
    bits[3] >>= 1;
    bits[1] += bits[3] % 2;
    bits[3] >>= 1;
    bits[2] += bits[3] % 2;
    bits[3] >>= 1;

    bits[3] <<= 4;

    // rrrrrrra
    // ggggggga
    // bbbbbbba
    // aaaa0000
    println!("{:08b}\n{:08b}\n{:08b}\n{:08b}\n", bits[0], bits[1], bits[2], bits[3]);

    bits[3] += id;

    println!("{:08b}\n{:08b}\n{:08b}\n{:08b}\n", bits[0], bits[1], bits[2], bits[3]);

    let u = u32::from_le_bytes(bits);
    println!("aaaaiiiirrrrrrragggggggabbbbbbba");
    println!("{:032b}, {}", u, u);

    // What we want to avoid.
    println!("\n{:032b}, nan\n", f32::NAN.to_bits());

    let f1 = f32::from_bits(u);

    // Convert back to normalized color.
    let bites_back = f1.to_bits();
    println!("aaaaiiiirrrrrrragggggggabbbbbbba");
    println!("{:032b}, {}\n", bites_back, bites_back);
    assert_eq!(bites_back, u);

    let r2 = (bites_back << 8) >> 25;
    println!("{}", r2);
    let g2 = (bites_back << 16) >> 25;
    println!("{}", g2);
    let b2 = (bites_back << 24) >> 25;
    println!("{}", b2);
    let a2 = (bites_back % 2) + ((bites_back >> 8) % 2) << 1 + ((bites_back >> 16) % 2) << 2 + (bites_back >> 28) << 4;
    println!("{}", a2);
    let i2 = (bites_back << 4) >> 28;
    println!("{:04b}", i2);
}
