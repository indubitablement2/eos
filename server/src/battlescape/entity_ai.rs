use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum EntityAi {
    /// Whole ai will be removed next update.
    #[default]
    None,
    /// Will try to face target and go forward at max speed.
    /// If target can not be found, will move forward.
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
impl EntityAi {
    /// Ruturn `true` if this ai should be removed.
    pub fn update(
        &mut self,
        entity_idx: usize,
        entities: &mut Entities,
        physics: &mut Physics,
    ) -> bool {
        // // Remove target if it does not exist.
        // let target_index = if let Some(target_id) = self.target {
        //     if let Some((i, _, _)) = entities.get_full(&target_id) {
        //         Some(i)
        //     } else {
        //         self.target = None;
        //         None
        //     }
        // } else {
        //     None
        // };

        let mut new_ai: Option<EntityAi> = None;

        match self {
            Self::None => {}
            Self::Seek => {
                // if let Some(target_index) = target_index {
                //     let position = *physics.body(entities[target_index].rb).translation();
                //     entities[entity_index].wish_angvel = WishAngVel::Aim { position };
                // } else {
                //     new_ai = Some(Self::Forward);
                // }
            }
            Self::Forward => {}
            Self::Fighter => todo!(),
            Self::Bomber => todo!(),
            Self::Drone => todo!(),
            Self::DroneStationaryOffset => todo!(),
            Self::ShipControlled => {
                // let e = &mut entities[entity_index];
                // if let Some(client) = e
                //     .fleet_ship
                //     .and_then(|(fleet_id, _)| fleets.get(&fleet_id))
                //     .and_then(|fleet| fleet.owner)
                //     .and_then(|client_id| clients.get(&client_id))
                // {
                //     e.wish_angvel = client.client_inputs.wish_angvel;
                //     e.wish_linvel = client.client_inputs.wish_linvel;
                // } else {
                //     // Client not found.
                //     new_ai = Some(Self::Ship);
                // }
            }
            Self::ShipEntering => {
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
            Self::Ship => {
                // TODO:
            }
        }

        if let Some(mut new_ai) = new_ai {
            let previous_ai = std::mem::replace(self, new_ai);
            self.changed(entity_idx, entities, previous_ai);
        }

        *self == Self::None
    }

    pub fn changed(&mut self, entity_idx: usize, entities: &mut Entities, previous_ai: Self) {
        match self {
            Self::None => {}
            Self::Seek => {
                entities[entity_idx].wish_linvel = WishLinVel::ForceRelative(vector![1.0, 0.0]);
            }
            Self::Forward => {}
            Self::Fighter => {}
            Self::Bomber => {}
            Self::Drone => {}
            Self::DroneStationaryOffset => {}
            Self::Ship => {}
            Self::ShipControlled => {}
            Self::ShipEntering => {}
        }
    }
}
