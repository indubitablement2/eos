mod client;
mod hull;

use super::*;
use client::*;
use connection::*;
use database_packet::{ClientUpdate, SimulationConnection, SimulationRequest, SimulationResponse};
use hull::*;
use ids::*;
use physics::{Hulls, Physics};
use rand::prelude::*;
use ship::{ShipDataId, ShipId};
use std::{
    f32::consts::{PI, TAU},
    ops::Range,
};

pub use hull::HullDataId;

type Clients = HashMap<ClientId, Client>;

pub const DT: Duration = Duration::from_millis(100);

/// How long between simulation saves.
/// Add some randomness to stagger saves.
const SAVE_INTERVAL: Range<f64> = (20.0 * 60.0)..(30.0 * 60.0);

const RADIUS: f32 = 10000.0;

pub fn load_data() {
    hull::data_json::load_hull_data();
}

pub struct Simulation {
    connection: SimulationConnection,

    /// Seconds since unix epoch.
    global_time: f64,
    next_save_global_time: f64,

    physics: Physics,

    clients: Clients,

    // TODO: Use brocoli + vector.
    // Can only query through brocoli
    // Client do not need to keep track of projectiles.
    projectiles: Vec<()>,
}
impl Simulation {
    pub fn new(connection: SimulationConnection) -> Self {
        Self {
            connection,
            physics: Default::default(),
            clients: Default::default(),
            global_time: global_time(),
            next_save_global_time: thread_rng().gen_range(SAVE_INTERVAL),
            projectiles: Default::default(),
        }
    }

    pub fn step(&mut self) {
        self.global_time = global_time();

        // Take new clients.
        // TODO: Verify client is added to simulation.
        while let Some((client_id, token, connection)) = self.connection.new_client() {
            self.clients.insert(client_id, Client::new(connection));
        }

        // Handle database packets.
        while let Some(response) = self.connection.try_recv() {
            match response {
                SimulationResponse::ClientAuthorisationAdd { client_id, token } => {}
                SimulationResponse::ClientAuthorisationRemove { client_id } => {}
                SimulationResponse::ShipEnter {
                    ship_id,
                    ship_data_id,
                    position,
                } => {
                    let hull = self.physics.hulls.insert(ship_data_id.hull_data_id).1;
                    hull.ship_id = Some(ship_id);
                    hull.position = position;
                }
            }
        }

        // Pre-step clients.
        let mut clients = std::mem::take(&mut self.clients);
        clients
            .iter_mut()
            .for_each(|(id, client)| client.step(*id, self));
        self.clients = clients;

        self.physics.step(&mut self.clients);

        // Update clients.
        clients = std::mem::take(&mut self.clients);
        clients.retain(|&client_id, client| client.post_step_retain(client_id, self));
        self.clients = clients;

        // Shrink containers.
        if self.clients.len() < self.clients.capacity() / 4 && self.clients.capacity() > 64 {
            self.clients.shrink_to_fit();
        }

        // Save.
        if self.global_time > self.next_save_global_time {
            self.save();
        }
    }

    fn save(&mut self) {
        self.next_save_global_time = self.global_time + thread_rng().gen_range(SAVE_INTERVAL);

        let ship_saves = self
            .physics
            .hulls
            .iter()
            .filter_map(|(_, hull)| {
                if let Some(ship_id) = hull.ship_id {
                    Some((ship_id, hull.position))
                } else {
                    None
                }
            })
            .collect();

        self.connection
            .queue(SimulationRequest::Save { ship_saves });
    }
}

fn global_time() -> f64 {
    std::time::UNIX_EPOCH
        .elapsed()
        .unwrap_or_default()
        .as_secs_f64()
}

trait AngleTo {
    fn angle_to(self, other: Self) -> f32;
}
impl AngleTo for f32 {
    fn angle_to(self, other: Self) -> f32 {
        let diff = other - self;
        if diff > PI {
            diff - TAU
        } else if diff < -PI {
            diff + TAU
        } else {
            diff
        }
    }
}
