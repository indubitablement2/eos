use crate::constants::GAME_TO_GODOT_RATIO;
use crate::draw::*;
use crate::util::*;
use ahash::AHashMap;
use battlescape::*;
use common::*;
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

    fn position(&self, weight: f32) -> Vec2 {
        self.pos.0.lerp(self.pos.1, weight)
    }

    fn draw(&self, weight: f32) {
        let pos = self.position(weight);
        let rot = self.rot.0.slerp(self.rot.1, weight);

        self.item
            .set_transform(pos.to_godot() * GAME_TO_GODOT_RATIO, rot);
        for hull in self.hulls.iter() {
            hull.draw(weight);
        }
    }
}

pub struct BattlescapeSnapshot {
    client_id: ClientId,
    root: DrawApi,

    follow: Option<ShipId>,
    target: Vec2,

    bound: f32,
    tick: u64,

    ships: AHashMap<ShipId, ShipSnapshot>,
}
impl BattlescapeSnapshot {
    pub fn new(client_id: ClientId, base: &Node2D) -> Self {
        Self {
            client_id,
            root: DrawApi::new_root(base),
            follow: None,
            target: Vec2::ZERO,
            bound: Default::default(),
            tick: Default::default(),
            ships: Default::default(),
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.root.set_visible(visible);
    }

    pub fn update(&mut self, bc: &Battlescape) {        
        self.bound = bc.bound;
        self.tick = bc.tick;
        log::debug!("update");

        if let Some(client) = bc.clients.get(&self.client_id) {
            if let Some(ship_id) = client.control {
                // Follow controlled ship.
                self.follow = Some(ship_id);
            } else {
                // TODO: Follow any ship in our fleet.
            }
        }

        // Fallback to following any ship.
        if self.follow.is_none() {
            self.follow = bc.ships.keys().next().copied();
        }

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

    pub fn draw_lerp(&mut self, weight: f32, _base: &Node2D) {
        let prev_target = self.target;
        
        // Set target on followed ship.
        if let Some(ship_id) = self.follow {
            if let Some(ship) = self.ships.get(&ship_id) {
                self.target = ship.position(weight);
            } else {
                // Followed ship is gone.
                self.follow = None;
            }
        }

        // TODO: Move ship removal to update.
        self.ships.drain_filter(|_, ship| {
            if ship.tick != self.tick {
                // This was not updated last frame. eg. it's removed.
                true
            } else {
                ship.draw(weight);

                false
            }
        });

        log::debug!("{:.3} : {:.3}", (prev_target - self.target).length(), weight);
        self.root.set_transform(self.target.to_godot() * -GAME_TO_GODOT_RATIO, 0.0);
    }
}
