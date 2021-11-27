use bevy_ecs::prelude::Entity;
use indexmap::IndexMap;

pub struct SystemId(u32);

pub struct SystemsRes {
    systems: IndexMap<SystemId, System>,
}
impl SystemsRes {
    pub fn new() -> Self {
        Self {
            systems: IndexMap::new(),
        }
    }

    pub fn spawn_fleet(&mut self) {
        todo!()
    }
}

struct System {
    
}