use std::hash::Hash;
use std::iter::once;

use crate::constants::GAME_TO_GODOT_RATIO;
use crate::draw::*;
use crate::util::*;
use ahash::AHashMap;
use battlescape::*;
use common::*;
use gdnative::prelude::Node2D;
use glam::Vec2;

struct HullRender {
    item: DrawApi,
    hull_data_id: HullDataId,
    hull_id: HullId,
}
impl HullRender {
    pub fn new(root: &DrawApi, hull_data_id: HullDataId, hull_id: HullId) -> Self {
        let item = root.new_child();
        item.add_texture(
            hull_data_id.data().texture_paths.albedo,
            hull_data_id.data().texture_paths.normal,
            true,
        );
        Self {
            item,
            hull_data_id,
            hull_id,
        }
    }
}

struct ShipRender {
    item: DrawApi,
    ship_data_id: ShipDataId,
    hulls: Vec<HullRender>,
}

struct ItemSnapshot {
    pos: Vec2,
    rot: f32,
}

struct _BattlescapeSnapshot {
    follow: Option<ShipId>,
    ships: AHashMap<ShipId, ItemSnapshot>,
    hulls: AHashMap<HullId, ItemSnapshot>,
}

pub struct BattlescapeSnapshot {
    client_id: ClientId,
    root: DrawApi,

    target: Vec2,

    bound: f32,

    /// We are ready to render any tick in this range.
    /// 
    /// It could be empty as we need at least 2 snapshots to interpolate.
    /// 
    /// We expect the next call to `take_snapshot()` to have a bc with tick `this.end`.
    pub available_render_tick: std::ops::Range<u64>,

    snapshots: Vec<_BattlescapeSnapshot>,
    ship_renders: AHashMap<ShipId, ShipRender>,
    events: Vec<()>,
}
impl BattlescapeSnapshot {
    pub fn new(client_id: ClientId, base: &Node2D, bc: &Battlescape) -> Self {
        let mut s = Self {
            client_id,
            root: DrawApi::new_root(base),
            target: Default::default(),
            bound: Default::default(),
            available_render_tick: Default::default(),
            snapshots: Default::default(),
            ship_renders: Default::default(),
            events: Default::default(),
        };

        s.reset(bc);

        s
    }

    /// Clear all internal and set it with the bc's current state.
    pub fn reset(&mut self, bc: &Battlescape) {
        // Take initial ship state.
        self.ship_renders = AHashMap::from_iter(bc.ships.iter().map(|(ship_id, ship)| {
            let ship_root = self.root.new_child();

            let hulls = once(&ship.main_hull)
                .chain(ship.auxiliary_hulls.iter())
                .map(|hull_id| {
                    let hull = bc.hulls.get(hull_id).unwrap();
                    HullRender::new(&ship_root, hull.hull_data_id, *hull_id)
                })
                .collect();

            (
                *ship_id,
                ShipRender {
                    item: ship_root,
                    ship_data_id: ship.ship_data_id,
                    hulls,
                },
            )
        }));

        self.bound = bc.bound;
        
        self.snapshots.clear();
        self.events.clear();

        self.take_snapshot_internal(bc);

        // We only have a single snapshot, so an empty range.
        self.available_render_tick = bc.tick..bc.tick;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.root.set_visible(visible);
    }

    pub fn next_expected_tick(&self) -> u64 {
        self.available_render_tick.end
    }

    /// The last tick we are ready to render.
    pub fn max_tick(&self) -> Option<u64> {
        // self.available_render_tick.last(), but range are not copy.
        if self.available_render_tick.is_empty() {
            None
        } else {
            Some(self.available_render_tick.end - 1)
        }
    }

    /// If we are ready to draw that tick.
    pub fn can_draw(&self, tick: u64) -> bool {
        self.available_render_tick.contains(&tick)
    }

    /// ## Warning
    /// We expect bc to be at tick `self.next_expected_tick()`. 
    /// Otherwise we will reset the snapshot the received tick.
    /// 
    /// Return if states was reset.
    pub fn take_snapshot(&mut self, bc: &Battlescape) -> bool {
        if bc.tick != self.next_expected_tick() {
            log::info!("Snapshot reset as bc tick is {} while expecting {}", bc.tick, self.next_expected_tick());
            self.reset(bc);
            true
        } else {
            self.take_snapshot_internal(bc);
            false
        }
    }

    fn take_snapshot_internal(&mut self, bc: &Battlescape) {
        let ships = AHashMap::from_iter(bc.ships.iter().map(|(ship_id, ship)| {
            let rb = &bc.physics.bodies[ship.rb];
            let pos = rb.translation().to_glam();
            let rot = rb.rotation().angle();

            (*ship_id, ItemSnapshot { pos, rot })
        }));

        let hulls = AHashMap::from_iter(bc.hulls.iter().map(|(hull_id, hull)| {
            let coll = &bc.physics.colliders[hull.collider];
            let pos = coll.position_wrt_parent().unwrap().translation.to_glam();
            let rot = coll.position_wrt_parent().unwrap().rotation.angle();

            (*hull_id, ItemSnapshot { pos, rot })
        }));

        // TODO: take events.

        // Take followed ship.
        let follow = if let Some(client) = bc.clients.get(&self.client_id) {
            client.control
        } else {
            None
        };

        self.snapshots.push(
            _BattlescapeSnapshot {
                follow,
                ships,
                hulls,
            },
        );

        self.available_render_tick.end = bc.tick;
    }

    /// Used to interpolate time dilation independent things like camera smoothing.
    pub fn update(&mut self, delta: f32) {
        // TODO:
    }

    pub fn draw_lerp(&mut self, tick: u64, weight: f32, _base: &Node2D) {
        if !self.available_render_tick.contains(&tick) {
            log::warn!("Can not draw tick {}. Available {:?}", tick, self.available_render_tick);
            return;
        }

        // The number of tick we will consume. 
        let advance = (tick - self.available_render_tick.start) as usize;
        self.available_render_tick.start = tick;

        // Apply previous events and remove old snapshots.
        for _ in 0..advance {
            let events = self.events.remove(0);
            self.apply_events(events);
            self.snapshots.remove(0);
        }

        // The snapshot we will interpolate between.
        let from = &self.snapshots[0];
        let to = &self.snapshots[1];

        let prev_target = self.target;
        // Set target on followed ship.
        if let Some(ship_id) = from.follow {
            if let Some((pos, _)) = get_snapshot_lerp(&ship_id, &from.ships, &to.ships, weight) {
                self.target = pos;
            }
        }
        log::debug!(
            "{:.3} : {:.3}",
            (prev_target - self.target).length(),
            weight
        );
        self.root
            .set_transform(self.target.to_godot() * -GAME_TO_GODOT_RATIO, 0.0);

        // Draw ships.
        for (ship_id, render_ship) in self.ship_renders.iter() {
            if let Some((pos, rot)) = get_snapshot_lerp(ship_id, &from.ships, &to.ships, weight) {
                // Update ship position.
                render_ship
                    .item
                    .set_transform(pos.to_godot() * GAME_TO_GODOT_RATIO, rot);

                // Update hulls position.
                for render_hull in render_ship.hulls.iter() {
                    if let Some((pos, rot)) =
                        get_snapshot_lerp(&render_hull.hull_id, &from.hulls, &to.hulls, weight)
                    {
                        render_hull
                            .item
                            .set_transform(pos.to_godot() * GAME_TO_GODOT_RATIO, rot);
                    }
                }
            }
        }
    }

    fn apply_events(&mut self, events: ()) {
        // TODO:
    }
}

fn get_snapshot_lerp<I: Hash + Eq>(
    id: &I,
    from: &AHashMap<I, ItemSnapshot>,
    to: &AHashMap<I, ItemSnapshot>,
    weight: f32,
) -> Option<(Vec2, f32)> {
    from.get(id).and_then(|a| {
        to.get(&id)
            .map(|b| (a.pos.lerp(b.pos, weight), a.rot.slerp(b.rot, weight)))
    })
}
