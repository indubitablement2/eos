use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientHello {
    pub username: String,
    pub game_version: u32,
    pub byte: u8,
    pub message: String,
    pub vec: Vec<u8>,
}

fn main() {
    let hello = ClientHello {
        username: "0123456789".to_string(),
        game_version: 123,
        byte: 255,
        message: "pick me".to_string(),
        vec: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    };

    let serialized = bincode::serialize(&hello).unwrap();

    println!("{}, {:?}", serialized.len(), serialized);

    if let Ok(deserialized) = bincode::deserialize::<ClientHello>(&serialized) {
        println!("{:?}", deserialized);
    } else {
        println!("Error deserializing.");
    }
}
