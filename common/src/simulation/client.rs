use super::*;

pub struct Client {
    connection: Connection,
}
impl Client {
    pub fn new_init(id: ClientId, connection: Connection, sim: &mut Simulation) -> Self {
        Self { connection }
    }

    pub fn pre_step(&mut self, id: ClientId, sim: &mut Simulation) {}

    pub fn post_step_retain(&mut self, id: ClientId, sim: &mut Simulation) -> bool {
        // TODO:
        true
    }
}

#[derive(Serialize)]
struct EntityState {
    network_id: u32,
    relative_translation: Vector2<f32>,
    rotation: u16,
}

#[derive(Serialize)]
enum ClientOutbound {
    EnteredSystem {
        client_id: ClientId,
        system_id: SimulationId,
    },
    State {
        time: f64,
        origin: Vector2<f32>,
        entitie_states: Vec<EntityState>,
    },
    AddEntity {
        hull_idx: u32,
        network_id: u32,
        entity_data_id: HullDataId,
    },
    RemoveEntity {
        network_id: u32,
    },
    RemoveSeenEntity {
        network_id: u32,
    },
    AddSeenEntity {
        network_id: u32,
    },
}

#[derive(Deserialize)]
enum ClientInbound {}
