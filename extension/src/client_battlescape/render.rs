use super::*;
use crate::battlescape::events::BattlescapeEventHandlerTrait;
use crate::battlescape::*;
use crate::util::*;
use crate::EntityDataId;
use glam::Vec2;
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::engine::Sprite2D;
use godot::prelude::*;

#[derive(Default)]
pub struct ClientBattlescapeEventHandler {
    tick: u64,
    entity_snapshots: AHashMap<EntityId, EntitySnapshot>,
    new_entities: Vec<(EntityId, EntityRender)>,
    render_handled: bool,
    battle_over: bool,
}
impl Drop for ClientBattlescapeEventHandler {
    fn drop(&mut self) {
        for (_, entity_render) in self.new_entities.iter_mut() {
            entity_render.node.queue_free();
        }
    }
}
impl BattlescapeEventHandlerTrait for ClientBattlescapeEventHandler {
    fn stepped(&mut self, bc: &Battlescape) {
        self.tick = bc.tick;

        // Take the position of all entities and their hulls.
        self.entity_snapshots = bc
            .entities
            .iter()
            .map(|(entity_id, entity)| {
                let rb = &bc.physics.bodies[entity.rb];

                let hulls = entity
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
                    .collect();

                (
                    *entity_id,
                    EntitySnapshot {
                        position: Position {
                            pos: rb.translation().to_glam(),
                            rot: rb.rotation().angle(),
                        },
                        hulls,
                    },
                )
            })
            .collect();
    }

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
}

impl HullRender {
    fn new(
        hull_index: usize,
        hull: &Option<entity::Hull>,
        entity: &entity::Entity,
        entity_node: &Gd<Node2D>,
    ) -> Self {
        let entity_data = entity.entity_data_id.data();
        let sprite = entity_node
            .get_child(entity_data.hulls[hull_index].render_node_idx, false)
            .unwrap()
            .cast();
        // TODO: Set hull shader.

        HullRender { sprite }
    }
}

struct EntityRender {
    node: Gd<Node2D>,
    entity_data_id: EntityDataId,
    hulls: SmallVec<[HullRender; 1]>,
}
impl EntityRender {
    /// Will not be added to the scene.
    /// `node` need to manualy free if this is drop before a call to `insert_to_scene`.
    fn new(entity: &entity::Entity) -> Self {
        let entity_data = entity.entity_data_id.data();

        let entity_node = entity_data
            .render_node
            .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
            .map(|node| node.cast::<Node2D>())
            .unwrap_or_else(|| Node2D::new_alloc());

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
        }
    }

    fn insert_to_scene(&self, draw_node: &Gd<Node2D>) {
        add_child(draw_node, &self.node);
    }
}
// Needed as it is contructed in events.
unsafe impl Send for EntityRender {}

pub struct BattlescapeRender {
    pub client_id: ClientId,
    client_node: Gd<Node>,
    draw_node: Gd<Node2D>,

    target: Vec2,

    /// We are ready to render any tick in this range.
    ///
    /// It could be empty as we need at least 2 snapshots to interpolate.
    ///
    /// We expect the next call to `take_snapshot()` to have a bc with tick `this.end`.
    pub available_render_tick: std::ops::Range<u64>,

    events: Vec<ClientBattlescapeEventHandler>,
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
            events: Default::default(),
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

        self.events.clear();
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

    pub fn hide(&mut self, hide: bool) {
        self.draw_node.set_visible(!hide);
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

    pub fn current_tick(&self) -> u64 {
        self.available_render_tick.start
    }

    /// ## Warning
    /// We expect bc to be at tick `self.next_expected_tick()`.
    /// Otherwise we will reset the snapshot to the received tick.
    ///
    /// Return if states was reset.
    pub fn take_snapshot(
        &mut self,
        bc: &Battlescape,
        events: ClientBattlescapeEventHandler,
    ) -> bool {
        if events.tick == self.next_expected_tick() {
            self.available_render_tick.end = events.tick;
            self.events.push(events);
            false
        } else {
            log::info!(
                "Render reset as events tick is {} while expecting {}",
                events.tick,
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
        self.events.drain(..advance);

        // The snapshot we will interpolate between.
        let from = &self.events[0].entity_snapshots;
        let to = &self.events[1].entity_snapshots;

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
                        let position = from.position.lerp(&to.position, weight);
                        render_hull.sprite.set_position(position.pos.to_godot());
                        render_hull.sprite.set_rotation(position.rot as f64);
                    }
                }
            }
        }
    }

    fn apply_snapshot_events(&mut self, snapshot_index: usize) {
        let snapshot = &mut self.events[snapshot_index];

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
