use super::*;

/// Update factions & fleets allied/enemy masks.
pub fn update_masks(s: &mut Metascape) {
    let factions = &mut s.factions;
    let fleets_index_map = &s.fleets.index_map;
    let fleets_masks = &mut s.fleets.container.masks;

    let new_masks = factions.reputations.compute_masks();

    let mut faction_changed = AHashSet::default();
    for ((faction, faction_id), new_masks) in factions
        .factions
        .iter_mut()
        .zip((0..64).map(|id| FactionId::new(id)))
        .zip(new_masks)
    {
        if faction.masks != new_masks {
            // Find the faction(s) that changed.
            for other_faction_id in (0..64).map(|id| FactionId::new(id)) {
                let other_mask = other_faction_id.mask();
                if faction.masks.allied & other_mask != new_masks.allied & other_mask
                    || faction.masks.enemy & other_mask != new_masks.enemy & other_mask
                {
                    // Mask between these faction changed.
                    faction_changed.insert(faction_id);
                    faction_changed.insert(other_faction_id);
                }
            }
        }
    }

    // Update fleet's masks that are in the factions that changed.
    for faction_id in faction_changed {
        let faction = factions.get_faction(faction_id);
        for fleet_id in faction.fleets.iter() {
            if let Some(&fleet_index) = fleets_index_map.get(&fleet_id) {
                fleets_masks[fleet_index] = faction.masks;
            }
        }
    }

    // TODO: Recompute client masks that are in neutral faction.
}
