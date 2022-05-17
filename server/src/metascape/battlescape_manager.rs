use super::{ecs_components::WrappedId, interception_manager::*};
use crate::metascape::ecs_components::*;
use ahash::AHashMap;
use battlescape::replay::BattlescapeReplay;
use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use common::idx::*;

#[derive(Debug)]
pub struct Battlescape {
    pub time: u32,

    pub players: Vec<Entity>,
    pub clients: Vec<u16>,
    pub teams: Vec<Vec<u16>>,

    pub auto_combat: bool,

    pub last_state: Option<(u32, Vec<u8>)>,
    pub replay: Option<BattlescapeReplay>,
}
impl Default for Battlescape {
    fn default() -> Self {
        Self {
            time: 0,
            players: Default::default(),
            clients: Default::default(),
            teams: Default::default(),
            auto_combat: true,
            last_state: None,
            replay: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct BattlescapeManager {
    last_used: u64,
    pub active_battlescape: AHashMap<BattlescapeId, Battlescape>,
}
impl BattlescapeManager {
    /// Try to join the other's battlescape or create a new one.
    /// Initiator and other are asumed to be enemy.
    /// # Panic
    /// Both entities should be valid.
    pub fn join_battlescape(
        &mut self,
        interception_manager: &mut InterceptionManager,
        commands: &mut Commands,
        query_client: Query<&WrappedId<ClientId>>,
        query_intercepted: Query<&WrappedId<InterceptionId>>,
        initiator: Entity,
        other: Entity,
    ) {
        debug_assert!(
            query_intercepted.get(initiator).is_err(),
            "Intercepted fleet can not initiate a battlescape."
        );

        if let Some((wrapped_interception_id, interception)) = query_intercepted
            .get(other)
            .ok()
            .and_then(|wrapped_interception_id| {
                interception_manager
                    .get_interception_mut(wrapped_interception_id.id())
                    .map(|interception| (wrapped_interception_id, interception))
            })
        {
            match interception.reason {
                InterceptedReason::Battle(battlescape_id) => {
                    // Join this battlescape.
                    commands
                        .entity(initiator)
                        .insert(wrapped_interception_id.to_owned());
                }
            }
        } else {
            // Create a new battlescape.
            self.last_used += 1;
            let battlescape_id = BattlescapeId::from_raw(self.last_used);

            // let clients = players.iter().zip((0u16..).into_iter())
            //     .filter_map(|(&entity, player_id)| {
            //         if let Ok(_) = query_client.get(entity) {
            //             commands.entity(entity).insert(BattlescapeInputs::new(battlescape_id)).insert(Intercepted)
            //             Some(player_id)
            //         } else {
            //             None
            //         }
            //     }).collect();

            // let result = self.active_battlescape.insert(
            //     battlescape_id,
            //     Battlescape {
            //         players,
            //         teams,
            //         ..Default::default()
            //     },
            // );
            // debug_assert!(result.is_none());

            // battlescape_id
        }
    }
}
