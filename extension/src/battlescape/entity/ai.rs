use super::*;

// #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
// pub enum EntityAiType {
//     /// Whole ai will be removed next update.
//     #[default]
//     None,
//     /// Will try to face target and go forward at max speed.
//     /// If target can not be found, will remove ai.
//     Seek,
//     Fighter,
//     Bomber,
//     Drone,
//     Ship,
// }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipAi {
    target: Option<EntityId>,
    entering: bool,
    tick: u32,
}
impl ShipAi {
    fn update(
        &mut self,
        entity_idx: usize,
        entities: &mut Entities,
        physics: &mut Physics,
        clients: &mut Clients,
        fleets: &mut Fleets,
        new_ai: &mut Option<EntityAi>,
    ) {
        // Remove target if it does not exist.
        let target_index = if let Some(target_id) = self.target {
            if let Some(i) = entities.get_index_of(&target_id) {
                Some(i)
            } else {
                self.target = None;
                None
            }
        } else {
            None
        };

        // TODO: Check if controlled

        // TODO: Make ai use position wish vel to test if it work.
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeekAi {
    pub target: EntityId,
}
impl SeekAi {
    fn update(
        &mut self,
        entity_idx: usize,
        entities: &mut Entities,
        physics: &mut Physics,
        new_ai: &mut Option<EntityAi>,
    ) {
        entities[entity_idx].wish_linvel = WishLinVel::Relative {
            force: na::Vector2::new(0.0, 1.0),
        };

        let target = if let Some(target) = entities.get(&self.target) {
            *physics.body_mut(target.rb).translation()
        } else {
            *new_ai = Some(EntityAi::None);
            return;
        };

        entities[entity_idx].wish_angvel = WishAngVel::Aim { position: target };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EntityAi {
    #[default]
    None,
    Seek(SeekAi),
    Ship(ShipAi),
}
impl EntityAi {
    pub fn new_ship() -> Self {
        Self::Ship(ShipAi::default())
    }

    pub fn new_seek(target: EntityId) -> Self {
        Self::Seek(SeekAi { target })
    }

    /// If this ai can be removed.
    pub fn can_remove(&self) -> bool {
        match self {
            EntityAi::None => true,
            _ => false,
        }
    }

    pub fn update(
        &mut self,
        entity_idx: usize,
        entities: &mut Entities,
        physics: &mut Physics,
        clients: &mut Clients,
        fleets: &mut Fleets,
    ) {
        let mut new_ai: Option<EntityAi> = None;

        match self {
            EntityAi::None => {}
            EntityAi::Seek(ai) => {
                ai.update(entity_idx, entities, physics, &mut new_ai);
            }
            EntityAi::Ship(ai) => {
                ai.update(entity_idx, entities, physics, clients, fleets, &mut new_ai);
            }
        }

        if let Some(new_ai) = new_ai {
            *self = new_ai;
        }
    }
}
