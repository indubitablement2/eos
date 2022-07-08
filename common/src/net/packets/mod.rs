pub mod client;
pub mod server;

pub use client::*;
pub use server::*;

pub trait Packet {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(buffer: &[u8]) -> Self;
}
