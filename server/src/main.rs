mod battlescape;
mod central_server;
mod connection;
mod instance_server;
mod logger;
mod metascape;

use ahash::{AHashMap, AHashSet, RandomState};
use battlescape::entity::{EntityData, EntityDataId};
use connection::*;
use indexmap::IndexMap;
use rand::prelude::*;
use rapier2d::na::{self, Vector2};
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

static mut TOKIO: Option<tokio::runtime::Runtime> = None;
fn tokio() -> &'static mut tokio::runtime::Runtime {
    unsafe { TOKIO.as_mut().unwrap() }
}

fn main() {
    logger::Logger::init();

    EntityData::set_data(vec![EntityData::default()]);

    #[cfg(all(feature = "instance_server", feature = "central_server"))]
    unsafe {
        TOKIO = Some(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(200));
            instance_server::InstanceServer::start();
        });
        central_server::CentralServer::start();
    }
    #[cfg(feature = "instance_server")]
    #[cfg(not(feature = "central_server"))]
    unsafe {
        TOKIO = Some(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(1)
                .build()
                .unwrap(),
        );
        instance_server::InstanceServer::start();
    }
    #[cfg(feature = "central_server")]
    #[cfg(not(feature = "instance_server"))]
    unsafe {
        TOKIO = Some(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
        central_server::CentralServer::start();
    }
}
