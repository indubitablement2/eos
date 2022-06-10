use crate::_metascape2::*;

/// Insert factions that were queued.
pub fn handle_faction_queue(s: &mut Metascape) {
    while let Some((faction_id, faction_builder)) = FACTION_QUEUE.pop() {
        let faction = Faction {
            name: faction_builder.name,
            reputations: faction_builder.reputations,
            fallback_reputation: faction_builder.fallback_reputation,
            clients: faction_builder.clients,
            fleets: faction_builder.fleets,
            colonies: faction_builder.colonies,
        };

        if s.factions.insert(faction_id, faction).1.is_some() {
            log::error!("{:?} overwritten.", faction_id);
        };
    }
}
