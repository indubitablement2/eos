use super::*;
use crate::battlescape::events::BattlescapeEventHandler;
use crate::battlescape::*;
use crate::util::*;
use crate::EntityDataId;
use glam::Vec2;
use godot::engine::node::InternalMode;
use godot::engine::Sprite2D;
use godot::engine::Texture2D;
use godot::prelude::*;

#[derive(Default)]
pub struct BattlescapeSnapshot {
    tick: u64,
    entity_snapshots: AHashMap<EntityId, EntitySnapshot>,
    new_entities: Vec<(EntityId, EntityRender)>,
    render_handled: bool,
    battle_over: bool,
}
impl BattlescapeSnapshot {
    pub fn take_snapshot(&mut self, bc: &Battlescape) {
        self.tick = bc.tick;

        // Take the position of all entities and their hulls.
        self.entity_snapshots = bc
            .entities
            .iter()
            .map(|(entity_id, entity)| {
                let rb = &bc.physics.bodies[entity.rb];
                (
                    *entity_id,
                    EntitySnapshot {
                        position: Position {
                            pos: rb.translation().to_glam(),
                            rot: rb.rotation().angle(),
                        },
                        hulls: entity
                            .hulls
                            .iter()
                            .map(|hull| {
                                hull.as_ref().map(|hull| {
                                    let p = &bc.physics.colliders[hull.collider]
                                        .position_wrt_parent()
                                        .unwrap();
                                    HullSnapshot {
                                        position: Position {
                                            pos: p.translation.to_glam(),
                                            rot: p.rotation.angle(),
                                        },
                                    }
                                })
                            })
                            .collect(),
                    },
                )
            })
            .collect();
    }
}
impl Drop for BattlescapeSnapshot {
    fn drop(&mut self) {
        for (_, entity_render) in self.new_entities.iter_mut() {
            entity_render.node.queue_free();
        }
    }
}
impl BattlescapeEventHandler for BattlescapeSnapshot {
    fn fleet_added(&mut self, fleet_id: crate::FleetId) {}

    fn ship_destroyed(&mut self, fleet_id: crate::FleetId, index: usize) {}

    fn entity_removed(&mut self, entity_id: EntityId, entity: entity::Entity) {}

    fn entity_added(&mut self, entity_id: EntityId, entity: &entity::Entity) {
        self.new_entities
            .push((entity_id, EntityRender::new(entity)));
    }

    fn battle_over(&mut self) {
        self.battle_over = true;
    }

    fn cast_snapshot(&mut self) -> Option<BattlescapeSnapshot> {
        Some(std::mem::take(self))
    }
}

#[derive(Debug, Clone, Copy)]
struct Position {
    pos: Vec2,
    rot: f32,
}
impl Position {
    fn lerp(&self, to: &Self, weight: f32) -> Self {
        Self {
            pos: self.pos.lerp(to.pos, weight),
            rot: self.rot.slerp(to.rot, weight),
        }
    }
}

struct EntitySnapshot {
    position: Position,
    hulls: SmallVec<[Option<HullSnapshot>; 1]>,
}

struct HullSnapshot {
    position: Position,
}

struct HullRender {
    sprite: Gd<Sprite2D>,
    hidden: bool,
}

impl HullRender {
    fn new(
        hull_index: usize,
        hull: &Option<entity::Hull>,
        entity: &entity::Entity,
        entity_node: &Gd<Node2D>,
    ) -> Self {
        let mut sprite = Sprite2D::new_alloc();
        sprite.set_texture(load("path")); // TODO: Load texture
        add_child(&entity_node, &sprite);

        HullRender {
            sprite,
            hidden: false,
        }
    }
}

struct EntityRender {
    node: Gd<Node2D>,
    entity_data_id: EntityDataId,
    hulls: SmallVec<[HullRender; 1]>,
    hidden: bool,
}
impl EntityRender {
    /// Will not be added to the scene.
    /// `node` need to manualy free if this is drop before a call to `insert_to_scene`.
    fn new(entity: &entity::Entity) -> Self {
        let mut entity_node = Node2D::new_alloc();
        entity_node.set_visible(false);

        let hulls = entity
            .hulls
            .iter()
            .enumerate()
            .map(|(hull_index, hull)| HullRender::new(hull_index, hull, entity, &entity_node))
            .collect();

        EntityRender {
            node: entity_node,
            entity_data_id: entity.entity_data_id,
            hulls,
            hidden: true,
        }
    }

    fn insert_to_scene(&self, draw_node: &Gd<Node2D>) {
        add_child(draw_node, &self.node);
    }
}
// Needed as it is contructed in events.
unsafe impl Send for EntityRender {}

pub struct BattlescapeRender {
    client_id: ClientId,
    client_node: Gd<Node>,
    draw_node: Gd<Node2D>,

    target: Vec2,

    /// We are ready to render any tick in this range.
    ///
    /// It could be empty as we need at least 2 snapshots to interpolate.
    ///
    /// We expect the next call to `take_snapshot()` to have a bc with tick `this.end`.
    pub available_render_tick: std::ops::Range<u64>,

    snapshots: Vec<BattlescapeSnapshot>,
    entity_renders: AHashMap<EntityId, EntityRender>,
}
impl BattlescapeRender {
    pub fn new(client_id: ClientId, client_node: Gd<Node>, bc: &Battlescape) -> Self {
        let mut s = Self {
            client_id,
            client_node,
            draw_node: Node2D::new_alloc(), // Not adding to scene as it will be free right away.
            target: Default::default(),
            available_render_tick: Default::default(),
            snapshots: Default::default(),
            entity_renders: Default::default(),
        };

        s.reset(bc);

        s
    }

    /// Clear all internal and set it with the bc's current state.
    pub fn reset(&mut self, bc: &Battlescape) {
        // Free previous node tree.
        self.draw_node.queue_free();
        self.draw_node = Node2D::new_alloc();
        add_child_node(&mut self.client_node, &self.draw_node);

        self.snapshots.clear();
        self.entity_renders.clear();

        // Take initial entity state.
        for (entity_id, entity) in bc.entities.iter() {
            let render_entity = EntityRender::new(entity);
            render_entity.insert_to_scene(&self.draw_node);
            self.entity_renders.insert(*entity_id, render_entity);
        }

        // We only have a single snapshot, so an empty range.
        self.available_render_tick = bc.tick..bc.tick;
    }

    pub fn hide(&mut self, visible: bool) {
        self.draw_node.set_visible(visible);
    }

    pub fn next_expected_tick(&self) -> u64 {
        self.available_render_tick.end + 1
    }

    /// The last tick we are ready to render.
    pub fn max_tick(&self) -> Option<u64> {
        self.available_render_tick.clone().last()
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
    pub fn take_snapshot(&mut self, bc: &Battlescape, snapshot: BattlescapeSnapshot) -> bool {
        if snapshot.tick == self.next_expected_tick() {
            self.available_render_tick.end = snapshot.tick;
            self.snapshots.push(snapshot);
            false
        } else {
            log::info!(
                "Render reset as snapshot tick is {} while expecting {}",
                snapshot.tick,
                self.next_expected_tick()
            );
            self.reset(bc);
            true
        }
    }

    /// Used to interpolate time dilation independent things like camera smoothing.
    ///
    /// Delta in real seconds.
    pub fn update(&mut self, delta: f32) {
        // TODO:
    }

    pub fn draw_lerp(&mut self, tick: u64, weight: f32) {
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

        // Apply previous events.
        for snapshot_index in 0..advance + 1 {
            self.apply_snapshot_events(snapshot_index);
        }
        // Remove old snapshots.
        self.snapshots.drain(..advance);

        // The snapshot we will interpolate between.
        let from = &self.snapshots[0].entity_snapshots;
        let to = &self.snapshots[1].entity_snapshots;

        // // Set target on followed ship.
        // if let Some(ship_id) = from.follow {
        //     if let Some((pos, _)) = get_snapshot_lerp(&ship_id, &from.ships, &to.ships, weight) {
        //         self.target = pos;
        //     }
        // }

        // Update positions.
        for (entity_id, render_entity) in self.entity_renders.iter_mut() {
            if let Some((from, to)) = from
                .get(entity_id)
                .and_then(|from| to.get(entity_id).map(|to| (from, to)))
            {
                if render_entity.hidden {
                    render_entity.hidden = false;
                    render_entity.node.set_visible(true);
                }

                let position = from.position.lerp(&to.position, weight);
                render_entity.node.set_position(position.pos.to_godot());
                render_entity.node.set_rotation(position.rot as f64);

                for ((render_hull, from), to) in render_entity
                    .hulls
                    .iter_mut()
                    .zip(&from.hulls)
                    .zip(&to.hulls)
                {
                    if let Some((from, to)) = from
                        .as_ref()
                        .and_then(|from| to.as_ref().map(|to| (from, to)))
                    {
                        if render_hull.hidden {
                            render_hull.hidden = false;
                            render_hull.sprite.set_visible(true);
                        }

                        let position = from.position.lerp(&to.position, weight);
                        render_hull.sprite.set_position(position.pos.to_godot());
                        render_hull.sprite.set_rotation(position.rot as f64);
                    } else {
                        // Missing at least one snapshot to interpolate hull.
                        if !render_hull.hidden {
                            render_hull.hidden = true;
                            render_hull.sprite.set_visible(false);
                        }
                    }
                }
            } else {
                // Missing at least one snapshot to interpolate entity.
                if !render_entity.hidden {
                    render_entity.hidden = true;
                    render_entity.node.set_visible(false);
                }
            }
        }
    }

    fn apply_snapshot_events(&mut self, snapshot_index: usize) {
        let snapshot = &mut self.snapshots[snapshot_index];

        if snapshot.render_handled {
            return;
        }
        snapshot.render_handled = true;

        for (entity_id, entity_render) in snapshot.new_entities.drain(..) {
            entity_render.insert_to_scene(&self.draw_node);
            self.entity_renders.insert(entity_id, entity_render);
        }
    }
}

fn add_child<A, B>(parent: &Gd<A>, child: &Gd<B>)
where
    A: Inherits<Node> + godot::prelude::GodotClass,
    B: Inherits<Node> + godot::prelude::GodotClass,
{
    parent.share().upcast().add_child(
        child.share().upcast(),
        false,
        InternalMode::INTERNAL_MODE_DISABLED,
    );
}

fn add_child_node<B>(parent: &mut Gd<Node>, child: &Gd<B>)
where
    B: Inherits<Node> + godot::prelude::GodotClass,
{
    parent.add_child(
        child.share().upcast(),
        false,
        InternalMode::INTERNAL_MODE_DISABLED,
    );
}
