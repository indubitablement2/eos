use crate::collision::IntersectionPipeline;
use crate::packets::ServerAddresses;
use crate::res_clients::ClientsRes;
use crate::res_factions::FactionsRes;
use crate::res_fleets::FleetsRes;
use crate::res_parameters::ParametersRes;
use crate::res_times::TimeRes;
use bevy_ecs::prelude::*;

pub struct Metascape {
    world: World,
    schedule: Schedule,
}
impl Metascape {
    pub fn new(local: bool, parameters: ParametersRes) -> std::io::Result<Self> {
        let mut world = World::new();
        world.insert_resource(TimeRes::new());
        world.insert_resource(parameters);
        world.insert_resource(IntersectionPipeline::new());
        world.insert_resource(ClientsRes::new(local));
        world.insert_resource(FactionsRes::new());
        world.insert_resource(FleetsRes::new());

        let mut schedule = Schedule::default();

        Ok(Self { world, schedule })
    }

    pub fn update(&mut self) {
        self.schedule.run_once(&mut self.world);
    }

    /// Get this server addressses.
    pub fn get_addresses(&self) -> ServerAddresses {
        self.world.get_resource::<ClientsRes>().unwrap().get_addresses()
    }
}
