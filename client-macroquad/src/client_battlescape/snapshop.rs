use crate::{prelude::*, utils::ToGlam};
use battlescape::*;

#[derive(Default)]
struct HullSnapshot {
    /// Used to detect when a hull should be removed.
    tick: u64,
    hull_data_id: HullDataId,
    pos: (Vec2, Vec2),
    rot: (f32, f32),
}
impl HullSnapshot {
    pub fn new(hull_data_id: HullDataId, pos: Vec2, rot: f32, tick: u64) -> Self {
        Self {
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
}

#[derive(Default)]
pub struct BattlescapeSnapshot {
    bound: f32,
    tick: u64,
    hulls: AHashMap<HullId, HullSnapshot>,
}
impl BattlescapeSnapshot {
    pub fn update_snapshot(&mut self, bc: &Battlescape) {
        self.bound = bc.bound;
        self.tick = bc.tick;

        for (hull_id, hull) in bc.hulls.iter() {
            if let Some(rb) = bc.physics.bodies.get(hull.rb) {
                let pos: Vec2 = rb.translation().to_glam();
                let rot = rb.rotation().angle();
                self.hulls
                    .entry(*hull_id)
                    .or_insert(HullSnapshot::new(hull.hull_data_id, pos, rot, bc.tick))
                    .update(pos, rot, bc.tick);
            }
        }
    }

    pub fn draw_lerp(&mut self, weight: f32, rendering: &mut Rendering) {
        self.hulls.drain_filter(|_, snapshot| {
            if snapshot.tick != self.tick {
                // This was not updated last frame. eg. it's removed.
                true
            } else {
                let pos = snapshot.pos.0.lerp(snapshot.pos.1, weight);
                let rot = snapshot.rot.0.slerp(snapshot.rot.1, weight);

                rendering.shaded_draw(
                    hull_data(snapshot.hull_data_id).texture_paths.shaded,
                    pos * 128.0, // TODO: Add bc scale constant.
                    rot,
                    0,
                );

                false
            }
        });
    }
}
