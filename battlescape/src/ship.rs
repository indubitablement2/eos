use crate::user_data::UserData;

use super::*;
use data::ship_data::*;

pub type AuxiliaryHulls = SmallVec<[HullId; 4]>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, PartialOrd, Ord, Default,
)]
pub struct ShipId(pub u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct Ship {
    pub rb: RigidBodyHandle,
    pub mobility: Mobility,
    pub main_hull: HullId,
    pub auxiliary_hulls: AuxiliaryHulls,
}

pub struct ShipBuilder {
    pub ship_data_id: ShipDataId,
    pub pos: na::Isometry2<f32>,
    pub linvel: na::Vector2<f32>,
    pub angvel: f32,
    pub team: u32,
}
impl ShipBuilder {
    pub fn new(ship_data_id: ShipDataId, pos: na::Isometry2<f32>, team: u32) -> Self {
        Self {
            ship_data_id,
            pos,
            linvel: Default::default(),
            angvel: Default::default(),
            team,
        }
    }

    pub fn with_linvel(mut self, linvel: na::Vector2<f32>) -> Self {
        self.linvel = linvel;
        self
    }

    pub fn with_angvel(mut self, angvel: f32) -> Self {
        self.angvel = angvel;
        self
    }

    pub fn queue(self, queue: &mut ShipSpawnQueue) -> ShipId {
        queue.queue(self)
    }
}

pub struct ShipSpawnQueue {
    next_ship_id: ShipId,
    queue: Vec<(ShipBuilder, ShipId)>,
}
impl ShipSpawnQueue {
    pub fn new(bc: &mut Battlescape) -> Self {
        Self {
            next_ship_id: bc.next_ship_id,
            queue: Default::default(),
        }
    }

    pub fn queue(&mut self, builder: ShipBuilder) -> ShipId {
        let hull_id = self.next_ship_id;
        self.next_ship_id.0 += 1;

        self.queue.push((builder, hull_id));

        hull_id
    }

    pub fn process(self, bc: &mut Battlescape) {
        bc.next_ship_id = self.next_ship_id;

        for (builder, ship_id) in self.queue {
            fn add_hull(
                bc: &mut Battlescape,
                hull_data_id: HullDataId,
                team: u32,
                group_ignore: u32,
                parrent_rb: RigidBodyHandle,
            ) -> HullId {
                let hull_data = hull_data_id.data();
                let hull_id = bc.new_hull_id();
                let user_data =
                    UserData::build(team, group_ignore, GenericId::from_hull_id(hull_id), false);
                let coll = build_hull_collider(hull_data_id, GROUPS_SHIP, user_data);
                let coll_handle = bc.physics.insert_collider(parrent_rb, coll);
                bc.hulls.insert(
                    hull_id,
                    Hull {
                        hull_data_id,
                        current_defence: hull_data.defence,
                        collider: coll_handle,
                    },
                );
                hull_id
            }

            let ship_data = builder.ship_data_id.data();
            let group_ignore = bc.physics.new_group_ignore();

            let rb = RigidBodyBuilder::dynamic()
                .position(builder.pos)
                .linvel(builder.linvel)
                .angvel(builder.angvel)
                .user_data(UserData::build(
                    builder.team,
                    group_ignore,
                    GenericId::ShipId(ship_id),
                    false,
                ))
                .build();
            let parrent_rb = bc.physics.bodies.insert(rb);

            // Add hulls.
            let main_hull = add_hull(
                bc,
                ship_data.main_hull,
                builder.team,
                group_ignore,
                parrent_rb,
            );
            let auxiliary_hulls: AuxiliaryHulls = ship_data
                .auxiliary_hulls
                .iter()
                .map(|&hull_data_id| {
                    add_hull(bc, hull_data_id, builder.team, group_ignore, parrent_rb)
                })
                .collect();

            bc.ships.insert(
                ship_id,
                Ship {
                    rb: parrent_rb,
                    mobility: ship_data.mobility,
                    main_hull,
                    auxiliary_hulls,
                },
            );
        }
    }
}
