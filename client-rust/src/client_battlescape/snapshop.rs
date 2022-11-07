use crate::constants::GAME_TO_GODOT_RATIO;
use crate::draw::*;
use crate::util::*;
use battlescape::*;
use gdnative::api::*;
use gdnative::prelude::Node2D;
use gdnative::prelude::*;
use glam::Vec2;

struct HullSnapshot {
    item: DrawApi,

    /// Used to detect when a hull should be removed.
    tick: u64,
    hull_data_id: HullDataId,
    pos: (Vec2, Vec2),
    rot: (f32, f32),
}
impl HullSnapshot {
    pub fn new(root: &DrawApi, hull_data_id: HullDataId, pos: Vec2, rot: f32, tick: u64) -> Self {
        let mut item = root.new_child();
        let hull_data = hull_data(hull_data_id);
        item.add_texture(
            hull_data.texture_paths.albedo,
            hull_data.texture_paths.normal,
            Vector2::ZERO,
            0.0,
        );

        Self {
            item,
            tick,
            hull_data_id,
            pos: (pos, pos),
            rot: (rot, rot),
        }
    }

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

        self.item.set_transform(Transform2D::IDENTITY.translated(pos.to_godot() * 128.0));
    }
}

pub struct BattlescapeSnapshot {
    root: DrawApi,

    bound: f32,
    tick: u64,
    // TODO: Draw order
    hulls: AHashMap<HullId, HullSnapshot>,
}
impl BattlescapeSnapshot {
    pub fn new(base: &Node2D) -> Self {
        Self {
            root: DrawApi::new_root(base),
            bound: Default::default(),
            tick: Default::default(),
            hulls: Default::default(),
        }
    }

    pub fn update(&mut self, bc: &Battlescape) {
        self.bound = bc.bound;
        self.tick = bc.tick;

        for (hull_id, hull) in bc.hulls.iter() {
            if let Some(rb) = bc.physics.bodies.get(hull.rb) {
                let pos: Vec2 = rb.translation().to_glam();
                let rot = rb.rotation().angle();
                self.hulls
                    .entry(*hull_id)
                    .or_insert(HullSnapshot::new(
                        &self.root,
                        hull.hull_data_id,
                        pos,
                        rot,
                        bc.tick,
                    ))
                    .update(pos, rot, bc.tick);
            }
        }
    }

    pub unsafe fn draw_lerp(&mut self, weight: f32, base: &Node2D) {
        self.hulls.drain_filter(|_, hull| {
            if hull.tick != self.tick {
                // This was not updated last frame. eg. it's removed.
                true
            } else {
                hull.draw(weight);

                false
            }
        });
    }
}
