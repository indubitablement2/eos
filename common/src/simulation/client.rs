use super::*;

pub struct Client {
    connection: Connection,
    known_hulls: AHashSet<HullId>,
}
impl Client {
    pub fn new_init(id: ClientId, connection: Connection, sim: &mut Simulation) -> Self {
        Self {
            connection,
            known_hulls: Default::default(),
        }
    }

    pub fn pre_step_retain(&mut self, id: ClientId, sim: &mut Simulation) -> bool {
        true
    }

    pub fn post_step(&mut self, id: ClientId, sim: &mut Simulation) {
        // TODO: Iterate over what his "team" can see instead

        let mut hull_add = Vec::new();
        let mut hull_remove = Vec::new();
        let mut hull_states = Vec::with_capacity(sim.physics.hulls.len());
        sim.physics.hulls.iter_mut().for_each(|(&hull_id, hull)| {
            // hull.tr

            if self.known_hulls.insert(hull_id) {
                hull_add.push(HullAdd {
                    index: hull_id.index,
                    data_id: hull.data,
                });
            }

            hull_states.push(HullState {
                index: hull_id.index,
                position: hull.position.map(|x| x.round() as i32),
                rotation: (hull.rotation.angle() / std::f32::consts::PI * 8191.0) as i16,
            });
        });

        self.connection.queue(ClientOutbound::State {
            time: sim.sim_time,
            hull_remove,
            hull_add,
            hull_states,
        });

        self.connection.flush();
    }
}

#[derive(Serialize)]
struct HullRemove {
    index: u32,
    // TODO: Reason
}

#[derive(Serialize)]
struct HullAdd {
    index: u32,
    data_id: HullDataId,
    // TODO: Reason
}

#[derive(Serialize)]
struct HullState {
    // 2
    index: u32,
    // 4
    position: Vector2<i32>,
    // 2
    rotation: i16,
}

#[derive(Serialize)]
enum ClientOutbound {
    State {
        time: f64,
        hull_remove: Vec<HullRemove>,
        hull_add: Vec<HullAdd>,
        hull_states: Vec<HullState>,
    },
}

#[derive(Deserialize)]
enum ClientInbound {}
