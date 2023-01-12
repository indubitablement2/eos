use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EntityAiType {
    /// Whole ai will be removed next update.
    #[default]
    None,
    /// Will try to face target and go forward at max speed.
    /// If target can not be found, will change ai to `Forward`.
    Seek,
    /// Go forward at max speed.
    Forward,
    Fighter,
    Bomber,
    Drone,
    DroneStationaryOffset,
    /// Controlled by a client, by copying its inputs.
    /// Revert to `Ship` if client is not found.
    ShipControlled,
    ShipEntering,
    Ship,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAi {
    target: Option<EntityId>,
    ai: EntityAiType,
}
impl EntityAi {
    pub fn new(
        target: Option<EntityId>,
        ai: EntityAiType,
        entity_index: usize,
        entities: &mut Entities,
    ) -> Self {
        let mut s = Self { target, ai };
        s.ai_changed(entity_index, entities, Default::default());
        s
    }

    /// If this ai can be removed.
    pub fn remove(&self) -> bool {
        if let EntityAiType::None = self.ai {
            true
        } else {
            false
        }
    }

    /// Ruturn `true` if this ai should be removed.
    pub fn update(
        &mut self,
        entity_index: usize,
        entities: &mut Entities,
        physics: &mut Physics,
        clients: &mut Clients,
        fleets: &mut Fleets,
    ) {
        // Remove target if it does not exist.
        let target_index = if let Some(target_id) = self.target {
            if let Some((i, _, _)) = entities.get_full(&target_id) {
                Some(i)
            } else {
                self.target = None;
                None
            }
        } else {
            None
        };

        let mut new_ai: Option<EntityAiType> = None;

        match self.ai {
            EntityAiType::None => {}
            EntityAiType::Seek => {
                if let Some(target_index) = target_index {
                    let position = *physics.body(entities[target_index].rb).translation();
                    entities[entity_index].wish_angvel = WishAngVel::Aim { position };
                } else {
                    new_ai = Some(EntityAiType::Forward);
                }
            }
            EntityAiType::Forward => {}
            EntityAiType::Fighter => todo!(),
            EntityAiType::Bomber => todo!(),
            EntityAiType::Drone => todo!(),
            EntityAiType::DroneStationaryOffset => todo!(),
            EntityAiType::ShipControlled => {
                let e = &mut entities[entity_index];
                if let Some(client) = e
                    .fleet_ship
                    .and_then(|(fleet_id, _)| fleets.get(&fleet_id))
                    .and_then(|fleet| fleet.owner)
                    .and_then(|client_id| clients.get(&client_id))
                {
                    e.wish_angvel = client.client_inputs.wish_angvel;
                    e.wish_linvel = client.client_inputs.wish_linvel;
                } else {
                    // Client not found.
                    new_ai = Some(EntityAiType::Ship);
                }
            }
            EntityAiType::ShipEntering => {
                // TODO:
                // *counter_since_entering += 1;

                // let e = entities.get_mut(&entity_id).unwrap();

                // // TODO: If entered
                // if false {
                //     change = Some(Some(EntityAi::Ship));
                // } else if *counter_since_entering > 600 {
                //     physics.body_mut(e.rb).set_translation(Vector2::zeros(), true);
                //     change = Some(Some(EntityAi::Ship));
                // }
            }
            EntityAiType::Ship => {
                // TODO:
            }
        }

        if let Some(new_ai) = new_ai {
            let previous_ai = std::mem::replace(&mut self.ai, new_ai);
            self.ai_changed(entity_index, entities, previous_ai);
        }
    }

    fn ai_changed(
        &mut self,
        entity_index: usize,
        entities: &mut Entities,
        previous_ai: EntityAiType,
    ) {
        match self.ai {
            EntityAiType::None => {}
            EntityAiType::Seek => {
                entities[entity_index].wish_linvel = WishLinVel::Forward;
            }
            EntityAiType::Forward => {
                entities[entity_index].wish_linvel = WishLinVel::Forward;
                entities[entity_index].wish_angvel = WishAngVel::Cancel;
            }
            EntityAiType::Fighter => {
                entities[entity_index].wish_linvel = WishLinVel::Forward;
            }
            EntityAiType::Bomber => {
                entities[entity_index].wish_linvel = WishLinVel::Forward;
            }
            EntityAiType::Drone => {}
            EntityAiType::DroneStationaryOffset => {}
            EntityAiType::Ship => {}
            EntityAiType::ShipControlled => {}
            EntityAiType::ShipEntering => {
                entities[entity_index].wish_linvel = WishLinVel::Forward;
                // TODO: Face a point forward from spawn position.
            }
        }
    }
}
