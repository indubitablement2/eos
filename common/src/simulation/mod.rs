mod client;
mod hull;
mod save;

use super::*;
use client::*;
use connection::*;
use database_packet::{ClientUpdate, SimulationConnection, SimulationRequest, SimulationResponse};
use hull::*;
use ids::*;
use physics::{Hulls, Physics};
use rand::prelude::*;
use std::{
    f32::consts::{PI, TAU},
    ops::Range,
};

type Clients = HashMap<ClientId, Client>;

pub const DT: Duration = Duration::from_millis(100);

/// How long between simulation saves.
/// Add some randomness to stagger saves.
const SAVE_INTERVAL: Range<f64> = (20.0 * 60.0)..(30.0 * 60.0);

const RADIUS: f32 = 10000.0;

pub fn load_data() {
    hull::data_json::load_hull_data();
}

struct Faction {
    faction_id: (),
    hulls: HashSet<HullId>,
    tracking_clients: Vec<ClientId>,
}

pub struct Simulation {
    connection: SimulationConnection,

    /// Seconds since unix epoch.
    global_time: f64,
    next_save_global_time: f64,

    /// Time since start of simulation.
    sim_time: f64,

    physics: Physics,

    clients: Clients,

    // TODO: Use brocoli + vector.
    // Can only query through brocoli
    // Client do not need to keep track of projectiles.
    projectiles: Vec<()>,
}
impl Simulation {
    pub fn new(connection: SimulationConnection, save: Option<&[u8]>) -> Self {
        let builder = if let Some(save) = save {
            match bin_decode::<save::SimulationSave>(save) {
                Ok(save) => save.to_builder(),
                Err(err) => {
                    log::error!("Failed to decode simulation save: {}", err);
                    save::SimulationBuilder::default()
                }
            }
        } else {
            save::SimulationBuilder::default()
        };

        Self {
            connection,
            sim_time: 0.0,
            physics: Default::default(),
            clients: Default::default(),
            global_time: global_time(),
            next_save_global_time: thread_rng().gen_range(SAVE_INTERVAL),
            projectiles: Default::default(),
        }
    }

    pub fn step(&mut self) {
        self.sim_time += DT.as_secs_f64();
        self.global_time = global_time();

        // Take new clients.
        while let Some((client_id, connection)) = self.connection.new_client() {
            self.clients.insert(client_id, Client::new(connection));
        }

        // Handle database packets.
        while let Some(response) = self.connection.try_recv() {
            match response {
                SimulationResponse::ClientUpdate { client_id, update } => todo!(),
                SimulationResponse::ClientLogoff { client_id } => {
                    self.clients.remove(&client_id);
                }
                SimulationResponse::ShipEnter { ship_id, hull_save } => {
                    let mut builder = bin_decode::<hull::save::HullSave>(&hull_save)
                        .unwrap_or_default()
                        .to_hull_builder();
                    builder.ship_id = Some(ship_id);

                    self.physics.hulls.insert(builder);
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
        clients.retain(|_, client| client.post_step_retain(self));
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
        self.next_save_global_time = thread_rng().gen_range(SAVE_INTERVAL);

        self.connection.queue(SimulationRequest::Save {
            simulation_save: bin_encode(save::SimulationSave::from_sim(self)),
        });
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
