mod character;
mod client;
mod grid;

use super::*;
use character::*;
use client::*;
use connection::*;
use database_packet::{SimulationConnection, SimulationResponse};
use grid::Characters;
use ids::*;
use rand::prelude::*;
use std::f32::consts::{PI, TAU};

/// 1/60th of a second.
pub const DT: Duration = Duration::from_micros(16667);
pub const CELL_SIZE: f32 = 128.0;

pub fn load_data() {
    character::CharacterData::load_data();
}

pub struct Simulation {
    connection: SimulationConnection,

    clients: HashMap<ClientId, Client>,

    ally_characters: Characters,
    enemy_characters: Characters,
}
impl Simulation {
    pub fn new(connection: SimulationConnection, save: Option<&[u8]>) -> Self {
        Self {
            connection,
            clients: Default::default(),
            ally_characters: Default::default(),
            enemy_characters: Default::default(),
        }
    }

    pub fn step(&mut self) {
        // Handle database packets.
        while let Some(response) = self.connection.try_recv() {
            match response {
                SimulationResponse::ClientLeft { client_id } => {
                    self.clients.remove(&client_id);
                }
            }
        }

        // Step clients.
        let mut clients = std::mem::take(&mut self.clients);
        clients
            .iter_mut()
            .for_each(|(id, client)| client.step(*id, self));
        self.clients = clients;

        // Step clients characters.
        self.ally_characters.update_query_structure();
        self.enemy_characters.update_query_structure();
        let mut idx = 0;
        while idx < self.ally_characters.characters.len() {
            let char_ref = self.ally_characters.characters[idx].clone();
            let Some(mut char) = char_ref.get_mut() else {
                self.ally_characters.characters.swap_remove(idx);
                continue;
            };
            char.step(self);
            if char.removed {
                self.ally_characters.characters.swap_remove(idx);
            } else {
                idx += 1;
            }
        }

        // Step monsters characters.
        std::mem::swap(&mut self.ally_characters, &mut self.enemy_characters);
        idx = 0;
        while idx < self.ally_characters.characters.len() {
            let char_ref = self.ally_characters.characters[idx].clone();
            let Some(mut char) = char_ref.get_mut() else {
                self.ally_characters.characters.swap_remove(idx);
                continue;
            };
            char.step(self);
            if char.removed {
                self.ally_characters.characters.swap_remove(idx);
            } else {
                idx += 1;
            }
        }
        std::mem::swap(&mut self.ally_characters, &mut self.enemy_characters);

        // Post-step clients.
        clients = std::mem::take(&mut self.clients);
        clients.retain(|&client_id, client| client.post_step_retain(client_id, self));
        self.clients = clients;

        // Shrink containers.
        if self.clients.len() < self.clients.capacity() / 4 && self.clients.capacity() > 64 {
            self.clients.shrink_to_fit();
        }
    }
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
