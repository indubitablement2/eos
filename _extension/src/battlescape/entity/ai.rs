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
        entity_id: EntityId,
        entities: &mut Entities,
        physics: &mut Physics,
        clients: &mut Clients,
        fleets: &mut Fleets,
        new_ai: &mut Option<EntityAi>,
        rng: &mut SimRng,
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

        let i = entities.get_index_of(&entity_id).unwrap();

        // Check if controlled
        if let Some(client) = entities[i]
            .owner
            .and_then(|owner| clients.get(&owner))
            .and_then(|client| client.control.map(|control| (client, control)))
            .and_then(|(client, control)| (control == entity_id).then_some(client))
        {
            entities[i].wish_linvel = client.client_inputs.wish_linvel;
            entities[i].wish_angvel = client.client_inputs.wish_angvel;
            return;
        }

        // TODO: Replace this with actual ship ai
        if self.tick % 100 == 0 {
            let position = na::Vector2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
            let wish_linvel = if rng.gen_bool(0.5) {
                WishLinVel::PositionOvershot { position }
            } else {
                WishLinVel::Position { position }
            };
            entities[i].wish_linvel = wish_linvel;
        }
        self.tick += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeekAi {
    pub target: EntityId,
}
impl SeekAi {
    fn update(
        &mut self,
        entity_id: EntityId,
        entities: &mut Entities,
        physics: &mut Physics,
        new_ai: &mut Option<EntityAi>,
    ) {
        let i = entities.get_index_of(&entity_id).unwrap();

        entities[i].wish_linvel = WishLinVel::Relative {
            force: na::Vector2::new(0.0, 1.0),
        };

        let target = if let Some(target) = entities.get(&self.target) {
            *physics.body_mut(target.rb).translation()
        } else {
            *new_ai = Some(EntityAi::None);
            return;
        };

        entities[i].wish_angvel = WishAngVel::Aim { position: target };
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
        entity_id: EntityId,
        entities: &mut Entities,
        physics: &mut Physics,
        clients: &mut Clients,
        fleets: &mut Fleets,
        rng: &mut SimRng,
    ) {
        // Check that the entity still exists.
        if !entities.contains_key(&entity_id) {
            *self = EntityAi::None;
            return;
        }

        let mut new_ai: Option<EntityAi> = None;

        match self {
            EntityAi::None => {}
            EntityAi::Seek(ai) => {
                ai.update(entity_id, entities, physics, &mut new_ai);
            }
            EntityAi::Ship(ai) => {
                ai.update(
                    entity_id,
                    entities,
                    physics,
                    clients,
                    fleets,
                    &mut new_ai,
                    rng,
                );
            }
        }

        if let Some(new_ai) = new_ai {
            *self = new_ai;
        }
    }
}
