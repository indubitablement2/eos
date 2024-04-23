mod client;
// mod entity;
pub mod hull;
mod physics;

use super::*;
use arena::*;
use client::Client;
use hull::*;
use physics::*;
use rapier2d::prelude::*;
use std::ops::Range;

pub const DT: Duration = Duration::from_millis(100);

/// How long between simulation saves.
/// Add some randomness to stagger saves.
const SAVE_INTERVAL: Range<f64> = (20.0 * 60.0)..(30.0 * 60.0);

const RADIUS: f32 = 100.0;

pub struct Simulation {
    simulation_id: SimulationId,

    database_connection: Connection,

    /// Seconds since unix epoch.
    global_time: f64,
    next_save_global_time: f64,

    /// Time since start of simulation.
    sim_time: f64,

    /// Entities always have 1 rigid body and 1 collider.
    ///
    /// Physics bodies are always an entity.
    /// Colliders are either an entity or a shield.
    physics: Physics,

    new_client: ConnectionListener,
    //IndexMap<ClientId, Strong<Client>, RandomState>
    clients: AHashMap<ClientId, Client>,

    hulls: Arena<HullId, Hull>,
    hulls_idices: Vec<u32>,

    // TODO: Use brocoli + vector.
    // Can only query through brocoli
    // Client do not need to keep track of projectiles.
    projectiles: Vec<()>,
}
impl Simulation {
    pub fn new(
        simulation_id: SimulationId,
        database_connection: Connection,
        new_client: ConnectionListener,
        save: SimulationSave,
    ) -> Self {
        Self {
            database_connection,
            new_client,
            sim_time: 0.0,
            physics: Default::default(),
            hulls: Default::default(),
            hulls_idices: Default::default(),
            clients: Default::default(),
            simulation_id,
            global_time: global_time(),
            next_save_global_time: thread_rng().gen_range(SAVE_INTERVAL),
            projectiles: Default::default(),
        }
    }

    pub fn step(&mut self) {
        self.sim_time += DT.as_secs_f64();
        self.global_time = global_time();

        // Take new clients.
        let mut new_client = self.new_client.clone();
        while let Some((connection, id)) = new_client.try_recv() {
            let id = ClientId::from_u64(id);
            let client = Client::new_init(id, connection, self);
            self.clients.insert(id, client);
        }

        // todo Handle database packets.

        // Pre-step clients.
        let mut clients = std::mem::take(&mut self.clients);
        clients
            .iter_mut()
            .for_each(|(id, client)| client.pre_step(*id, self));
        self.clients = clients;

        self.physics.step();

        // TODO Handle physic events.
        for (a, event) in self.physics.events.0.try_lock().unwrap().iter().copied() {
            // if let Some(entity) = self.entities.get_mut(&a) {
            //     entity.take_contact_event(event);
            // }

            // let b = event.with_entity_id;
            // let event = ContactEvent {
            //     collider_id: event.with_collider_id,
            //     with_entity_id: a,
            //     with_collider_id: event.collider_id,
            //     force_direction: event.force_direction,
            //     force_magnitude: event.force_magnitude,
            // };
            // if let Some(entity) = self.entities.get_mut(&b) {
            //     entity.take_contact_event(event);
            // }
        }

        // Update hulls physics state.
        for hull_idx in self.hulls_idices.iter() {
            let hull = self.hulls.get_mut_index(*hull_idx as usize).unwrap();
            let body = self.physics.get_body(hull.rb);
            hull.pos = *body.position();
            hull.linvel = *body.linvel();
            hull.angvel = body.angvel();
        }

        // Update hulls.
        let mut i = 0;
        while i < self.hulls_idices.len() {
            let idx = self.hulls_idices[i] as usize;
            let (id, mut hull) = self.hulls.leak_take_index(idx);

            if let Some(reason) = hull.update(id, &self.physics, &mut self.hulls) {
                hull.on_remove(reason);

                self.hulls_idices.swap_remove(i);
                self.hulls.unleak_remove_index(idx);

                self.physics.remove_body(hull.rb);
            } else {
                let body = self.physics.get_body_mut(hull.rb);
                body.set_position(hull.pos, true);
                body.set_linvel(hull.linvel, true);
                body.set_angvel(hull.angvel, true);

                self.hulls.unleak_set_index(idx, hull);
                i += 1;
            }
        }

        // Update clients.
        clients = std::mem::take(&mut self.clients);
        clients.retain(|id, client| client.post_step_retain(*id, self));
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

        let simulation_save = SimulationSave {};

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
pub struct SimulationSave {
    // TODO: Debris
    // TODO: items
    // TODO: planets state
}
impl Default for SimulationSave {
    fn default() -> Self {
        Self {}
    }
}
