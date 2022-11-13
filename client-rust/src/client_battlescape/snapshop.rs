use std::hash::Hash;
use std::iter::once;

use crate::constants::GAME_TO_GODOT_RATIO;
use crate::draw::*;
use crate::util::*;
use ahash::AHashMap;
use battlescape::bc_event::BattlescapeEvents;
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
impl ShipRender {
    pub fn new(ship_id: &ShipId, root: &DrawApi, bc: &Battlescape) -> Self {
        let ship = bc.ships.get(ship_id).unwrap();

        let ship_root = root.new_child();

        let hulls = once(&ship.main_hull)
            .chain(ship.auxiliary_hulls.iter())
            .map(|hull_id| {
                let hull = bc.hulls.get(hull_id).unwrap();
                HullRender::new(&ship_root, hull.hull_data_id, *hull_id)
            })
            .collect();

        Self {
            item: ship_root,
            ship_data_id: ship.ship_data_id,
            hulls,
        }
    }
}

struct ItemSnapshot {
    pos: Vec2,
    rot: f32,
}

#[derive(Default)]
pub struct RenderEvents {
    add_ships: Vec<(ShipId, ShipRender)>,
}
impl RenderEvents {
    pub fn from_bc_events(root: &DrawApi, bc_events: &BattlescapeEvents, bc: &Battlescape) -> Self {
        let add_ships = bc_events
            .add_ship
            .iter()
            .map(|ship_id| (*ship_id, ShipRender::new(ship_id, root, bc)))
            .collect::<Vec<_>>();

        for (_, ship) in add_ships.iter() {
            ship.item.set_visible(false);
        }

        Self { add_ships }
    }
}

struct _BattlescapeSnapshot {
    follow: Option<ShipId>,
    ships: AHashMap<ShipId, ItemSnapshot>,
    hulls: AHashMap<HullId, ItemSnapshot>,
    /// None if they were already consumed.
    events: Option<RenderEvents>,
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
        };

        s.reset(bc);

        s
    }

    /// Clear all internal and set it with the bc's current state.
    pub fn reset(&mut self, bc: &Battlescape) {
        // Take initial ship state.
        self.ship_renders = AHashMap::from_iter(
            bc.ships
                .keys()
                .map(|ship_id| (*ship_id, ShipRender::new(ship_id, &self.root, bc))),
        );

        self.bound = bc.bound;

        self.snapshots.clear();

        self.take_snapshot_internal(bc, &Default::default());

        // We only have a single snapshot, so an empty range.
        self.available_render_tick = bc.tick..bc.tick;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.root.set_visible(visible);
    }

    pub fn next_expected_tick(&self) -> u64 {
        self.available_render_tick.end + 1
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
    /// Otherwise we will reset the snapshot to the received tick.
    ///
    /// Return if states was reset.
    pub fn take_snapshot(&mut self, bc: &Battlescape, events: &BattlescapeEvents) -> bool {
        if bc.tick == self.next_expected_tick() {
            self.take_snapshot_internal(bc, events);
            false
        } else if self.can_draw(bc.tick) || bc.tick == self.available_render_tick.end {
            // Already have that tick.
            false
        } else {
            log::info!(
                "Snapshot reset as bc tick is {} while expecting {}",
                bc.tick,
                self.next_expected_tick()
            );
            self.reset(bc);
            true
        }
    }

    fn take_snapshot_internal(&mut self, bc: &Battlescape, events: &BattlescapeEvents) {
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

        // Take followed ship.
        let follow = if let Some(client) = bc.clients.get(&self.client_id) {
            client.control
        } else {
            None
        };

        // Take events.
        let events = RenderEvents::from_bc_events(&self.root, events, bc);

        self.snapshots.push(_BattlescapeSnapshot {
            follow,
            ships,
            hulls,
            events: Some(events),
        });

        self.available_render_tick.end = bc.tick;
    }

    /// Used to interpolate time dilation independent things like camera smoothing.
    pub fn update(&mut self, delta: f32) {
        // TODO:
    }

    pub fn draw_lerp(&mut self, tick: u64, weight: f32, _base: &Node2D) {
        if !self.can_draw(tick) {
            log::warn!(
                "Can not draw tick {}. Available {:?}",
                tick,
                self.available_render_tick
            );
            return;
        }

        // The number of tick we will consume.
        let advance = (tick - self.available_render_tick.start) as usize;
        self.available_render_tick.start = tick;

        // Apply previous events and remove old snapshots.
        for _ in 0..advance {
            if let Some(events) = self.snapshots.remove(0).events {
                self.apply_events(events);
            }
        }

        // Apply current envets.
        if let Some(events) = self.snapshots[0].events.take() {
            self.apply_events(events);
        }

        // The snapshot we will interpolate between.
        let from = &self.snapshots[0];
        let to = &self.snapshots[1];

        // Set target on followed ship.
        if let Some(ship_id) = from.follow {
            if let Some((pos, _)) = get_snapshot_lerp(&ship_id, &from.ships, &to.ships, weight) {
                self.target = pos;
            }
        }
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

    fn apply_events(&mut self, events: RenderEvents) {
        for (ship_id, ship_render) in events.add_ships {
            ship_render.item.set_visible(true);
            self.ship_renders.insert(ship_id, ship_render);
        }
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
