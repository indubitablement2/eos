use std::fs::File;
use std::io::prelude::*;

#[test]
fn test_file() {
    match File::open("~/.local/share/godot/app_userdata/chaos-cascade/mods/test.txt") {
        Ok(mut file) => {
            let mut buf = String::with_capacity(1024);
            if file.read_to_string(&mut buf).is_ok() {
                println!("{}", buf);
            }
        }
        Err(err) => {
            println!("{:?}", err);
        }
    }
}
