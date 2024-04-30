mod client;
mod hull;

use super::*;
use client::*;
use connection::*;
use hull::*;
use ids::*;
use physics::*;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Point2, UnitComplex, Vector2};
use rapier2d::prelude::*;
use std::ops::Range;
use system::SystemId;

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
    system_id: SystemId,

    database_connection: Connection,

    /// Seconds since unix epoch.
    global_time: f64,
    next_save_global_time: f64,

    /// Time since start of simulation.
    sim_time: f64,

    physics: Physics,

    new_client: ConnectionListener,
    clients_auth: Vec<ClientAuth>,
    clients: Clients,

    // TODO: Use brocoli + vector.
    // Can only query through brocoli
    // Client do not need to keep track of projectiles.
    projectiles: Vec<()>,
}
impl Simulation {
    pub fn new(
        database_connection: Connection,
        new_client: ConnectionListener,
        system_id: SystemId,
        save: Option<&[u8]>,
    ) -> Self {
        let save = if let Some(save) = save {
            match bin_decode(save) {
                Ok(save) => save,
                Err(err) => {
                    log::error!("Failed to decode save: {}", err);
                    SimulationSave::default()
                }
            }
        } else {
            SimulationSave::default()
        };

        Self {
            system_id,
            database_connection,
            new_client,
            clients_auth: Default::default(),
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
        while let Some(connection) = self.new_client.try_recv() {
            self.clients_auth.push(ClientAuth::new(connection));
        }

        // todo Handle database packets.

        // Authenticate clients.
        self.clients_auth.retain_mut(|auth| match auth.step() {
            Some(Ok((id, client))) => {
                self.clients.insert(id, client);
                false
            }
            Some(Err(())) => false,
            None => true,
        });

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

    pub fn save(&mut self) {
        self.next_save_global_time = thread_rng().gen_range(SAVE_INTERVAL);

        // let simulation_save = SimulationSave {};

        // self.database_outbound
        //     .queue(DatabaseRequest::SaveSimulation {
        //         simulation_id: self.simulation_id,
        //         simulation_save,
        //     });

        // TODO: Save ships
        // TODO: Save planets?
    }
}

fn global_time() -> f64 {
    std::time::UNIX_EPOCH
        .elapsed()
        .unwrap_or_default()
        .as_secs_f64()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct SimulationSave {
    // TODO: Debris
    // TODO: items
    // TODO: planets state
}
impl Default for SimulationSave {
    fn default() -> Self {
        Self {}
    }
}
