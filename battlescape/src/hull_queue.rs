use super::*;

pub struct HullSpawnQueue {
    next_hull_id: u32,
    queue: Vec<(HullBuilder, HullId)>,
}
impl HullSpawnQueue {
    pub fn new(bc: &mut Battlescape) -> Self {
        Self {
            next_hull_id: bc.next_hull_id,
            queue: Default::default(),
        }
    }

    pub fn queue(&mut self, hull_builder: HullBuilder) -> HullId {
        let hull_id = HullId(self.next_hull_id);
        self.next_hull_id += 1;

        self.queue.push((hull_builder, hull_id));

        hull_id
    }

    pub fn process(self, bc: &mut Battlescape) {
        bc.next_hull_id = self.next_hull_id;

        for (hull_builder, hull_id) in self.queue {
            let hull_data = hull_data(hull_builder.hull_data_id);

            let rb = bc.physics.add_body(
                hull_builder.pos,
                hull_builder.linvel,
                hull_builder.angvel,
                hull_data.shape.to_shared_shape(),
                hull_data.density,
                hull_data.groups,
                0,
                hull_builder.team,
                false,
                hull_id.0,
                hull_id,
            );
    
            // TODO: Add joined childs.
            let childs = Childs::new();
    
            let hull = Hull {
                hull_data_id: hull_builder.hull_data_id,
                current_mobility: hull_data.mobility,
                current_defence: hull_data.defence,
                rb,
                childs,
                parent: None,
            };
    
            bc.hulls.insert(hull_id, hull);
        }
    }
}