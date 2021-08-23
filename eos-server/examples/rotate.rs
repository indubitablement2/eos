fn main() {
    let vec_size = 3;
    let mut vec_tl: Vec<u8> = Vec::with_capacity(vec_size * vec_size);

    for y in 0..vec_size as u8 {
        for x in 0..vec_size as u8 {
            vec_tl.push(x + y * vec_size as u8);
        }
    }

    let mut vec_br = vec_tl.clone();
    vec_br.reverse();

    let mut vec_tr = vec_tl.clone();


    let mut vec_bl = vec_tl.clone();

    for i in 0..vec_size {
        print!("{:?}", &vec_tl[i * vec_size..i * vec_size + vec_size]);
        println!("{:?}", &vec_tr[i * vec_size..i * vec_size + vec_size]);
    }
    for i in 0..vec_size {
        print!("{:?}", &vec_bl[i * vec_size..i * vec_size + vec_size]);
        println!("{:?}", &vec_br[i * vec_size..i * vec_size + vec_size]);
    }

}


fn print_vec(vec: &Vec<u8>, vec_size: usize) {
    for i in 0..vec_size {
        println!("{:?}", &vec[i * 3..i * 3 + 3]);
    }
    println!("");
}