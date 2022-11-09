use crate::constants::GAME_TO_GODOT_RATIO;
use crate::draw::*;
use crate::util::*;
use ahash::AHashMap;
use battlescape::*;
use gdnative::prelude::Node2D;
use gdnative::prelude::*;
use glam::Vec2;

struct HullSnapshot {
    item: DrawApi,

    hull_data_id: HullDataId,
    hull_id: HullId,
    pos: (Vec2, Vec2),
    rot: (f32, f32),
}
impl HullSnapshot {
    pub fn new(
        root: &DrawApi,
        hull_data_id: HullDataId,
        hull_id: HullId,
        pos: Vec2,
        rot: f32,
    ) -> Self {
        let mut item = root.new_child();
        let hull_data = hull_data_id.data();
        item.add_texture(
            hull_data.texture_paths.albedo,
            hull_data.texture_paths.normal,
            true,
        );

        Self {
            item,
            hull_data_id,
            hull_id,
            pos: (pos, pos),
            rot: (rot, rot),
        }
    }

    fn update(&mut self, pos: Vec2, rot: f32) {
        self.pos.0 = self.pos.1;
        self.pos.1 = pos;
        self.rot.0 = self.rot.1;
        self.rot.1 = rot;
    }

    fn draw(&self, weight: f32) {
        let pos = self.pos.0.lerp(self.pos.1, weight);
        let rot = self.rot.0.slerp(self.rot.1, weight);

        self.item
            .set_transform(pos.to_godot() * GAME_TO_GODOT_RATIO, rot);
    }
}

struct ShipSnapshot {
    item: DrawApi,

    /// Used to detect when a ship should be removed.
    tick: u64,

    ship_data_id: ShipDataId,
    pos: (Vec2, Vec2),
    rot: (f32, f32),
    hulls: Vec<HullSnapshot>,
}
impl ShipSnapshot {
    fn update(&mut self, pos: Vec2, rot: f32, tick: u64) {
        self.tick = tick;
        self.pos.0 = self.pos.1;
        self.pos.1 = pos;
        self.rot.0 = self.rot.1;
        self.rot.1 = rot;
    }

    fn draw(&self, weight: f32) {
        let pos = self.pos.0.lerp(self.pos.1, weight);
        let rot = self.rot.0.slerp(self.rot.1, weight);

        self.item
            .set_transform(pos.to_godot() * GAME_TO_GODOT_RATIO, rot);
        for hull in self.hulls.iter() {
            hull.draw(weight);
        }
    }
}

pub struct BattlescapeSnapshot {
    root: DrawApi,

    bound: f32,
    tick: u64,

    ships: AHashMap<ShipId, ShipSnapshot>,
}
impl BattlescapeSnapshot {
    pub fn new(base: &Node2D) -> Self {
        Self {
            root: DrawApi::new_root(base),
            bound: Default::default(),
            tick: Default::default(),
            ships: Default::default(),
        }
    }

    pub fn update(&mut self, bc: &Battlescape) {
        self.bound = bc.bound;
        self.tick = bc.tick;

        for (ship_id, ship) in bc.ships.iter() {
            let rb = &bc.physics.bodies[ship.rb];
            let pos = rb.translation().to_glam();
            let rot = rb.rotation().angle();

            let ship_snapshot = self.ships.entry(*ship_id).or_insert_with(|| {
                let root = self.root.new_child();

                let hulls = std::iter::once(&ship.main_hull)
                    .chain(ship.auxiliary_hulls.iter())
                    .map(|hull_id| {
                        let hull = bc.hulls.get(hull_id).unwrap();
                        let coll = &bc.physics.colliders[hull.collider];
                        let pos = coll.position_wrt_parent().unwrap().translation.to_glam();
                        let rot = coll.position_wrt_parent().unwrap().rotation.angle();
                        HullSnapshot::new(&root, hull.hull_data_id, *hull_id, pos, rot)
                    })
                    .collect();

                ShipSnapshot {
                    item: root,
                    tick: self.tick,
                    ship_data_id: ship.ship_data_id,
                    pos: (pos, pos),
                    rot: (rot, rot),
                    hulls,
                }
            });

            ship_snapshot.update(pos, rot, self.tick);
            ship_snapshot.hulls.drain_filter(|hull_snapshot| {
                if let Some(hull) = bc.hulls.get(&hull_snapshot.hull_id) {
                    let coll = &bc.physics.colliders[hull.collider];
                    let pos = coll.position_wrt_parent().unwrap().translation.to_glam();
                    let rot = coll.position_wrt_parent().unwrap().rotation.angle();
                    hull_snapshot.update(pos, rot);
                    false
                } else {
                    true
                }
            });
        }
    }

    pub unsafe fn draw_lerp(&mut self, weight: f32, _base: &Node2D) {
        self.ships.drain_filter(|_, ship| {
            if ship.tick != self.tick {
                // This was not updated last frame. eg. it's removed.
                true
            } else {
                ship.draw(weight);

                false
            }
        });
    }
}
