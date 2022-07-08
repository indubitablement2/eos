mod accept_loop;
pub mod auth;
pub mod connection;
pub mod offline;
pub mod offline_client;
pub mod online;
pub mod online_client;
pub mod packets;
mod tcp_connection;
mod tcp_loops;
mod udp_loops;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub use auth::*;
pub use connection::*;
pub use packets::*;

/// Udp packet above this size will be truncated.
pub const MAX_UNRELIABLE_PACKET_SIZE: usize = 1024;
/// Tcp packet above this size will cause the stream to be corrupted.
pub const MAX_RELIABLE_PACKET_SIZE: usize = u16::MAX as usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerAddr {
    pub udp: SocketAddr,
    pub tcp: SocketAddr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerAddrs {
    pub v4_addr: Option<ServerAddr>,
    pub v6_addr: Option<ServerAddr>,
}
