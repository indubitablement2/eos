use super::*;
use crate::battlescape::events::BattlescapeEventHandlerTrait;
use crate::battlescape::*;
use crate::util::*;
use crate::EntityDataId;
use glam::Vec2;
use godot::engine::packed_scene::GenEditState;
use godot::engine::Sprite2D;
use godot::prelude::*;

#[derive(Default)]
pub struct RenderBattlescapeEventHandler {
    pub take_full: bool,
    entity_snapshots: AHashMap<EntityId, EntitySnapshot>,
    new_entities: Vec<(EntityId, EntityRender)>,
    removed_entities: Vec<EntityId>,
    removed_hull: Vec<(EntityId, usize)>,
}
impl BattlescapeEventHandlerTrait for RenderBattlescapeEventHandler {
    fn stepped(&mut self, bc: &Battlescape) {
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

        if self.take_full {
            // Take a full snapshot of the battlescape.
            self.new_entities = bc
                .entities
                .iter()
                .map(|(entity_id, entity)| (*entity_id, EntityRender::new(entity)))
                .collect();
        }
    }

    fn fleet_added(&mut self, fleet_id: crate::FleetId) {}

    fn ship_destroyed(&mut self, fleet_id: crate::FleetId, ship_index: usize) {}

    fn entity_removed(&mut self, entity_id: EntityId, entity: entity::Entity) {
        if !self.take_full {
            self.removed_entities.push(entity_id);
        }
    }

    fn hull_removed(&mut self, entity_id: EntityId, hull_index: usize) {
        if !self.take_full {
            self.removed_hull.push((entity_id, hull_index));
        }
    }

    fn entity_added(&mut self, entity_id: EntityId, entity: &entity::Entity) {
        if !self.take_full {
            self.new_entities
                .push((entity_id, EntityRender::new(entity)));
        }
    }

    fn battle_over(&mut self) {}
}
impl RenderBattlescapeEventHandler {
    pub fn new(take_full: bool) -> Self {
        Self {
            take_full,
            ..Default::default()
        }
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

pub struct EntityRender {
    pub node: Gd<Node2D>,
    pub entity_data_id: EntityDataId,
    hulls: SmallVec<[Option<HullRender>; 1]>,
    pub fleet_ship: Option<FleetShip>,
}
impl EntityRender {
    /// Will not be added to the scene.
    /// `node` need to manualy free if this is drop before a call to `insert_to_scene`.
    fn new(entity: &entity::Entity) -> Self {
        let entity_data = entity.entity_data_id.data();

        let entity_node = entity_data
            .render_scene
            .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
            .map(|node| node.cast::<Node2D>())
            .unwrap_or_else(|| Node2D::new_alloc());

        let hulls = entity
            .hulls
            .iter()
            .enumerate()
            .map(|(hull_index, hull)| Some(HullRender::new(hull_index, hull, entity, &entity_node)))
            .collect();

        EntityRender {
            node: entity_node,
            entity_data_id: entity.entity_data_id,
            hulls,
            fleet_ship: entity.fleet_ship,
        }
    }

    fn insert_to_scene(&self, draw_node: &Gd<Node2D>) {
        add_child(draw_node, &self.node);
    }
}
impl Drop for EntityRender {
    fn drop(&mut self) {
        self.node.queue_free()
    }
}
// Needed as it is contructed in events.
unsafe impl Send for EntityRender {}

pub struct BattlescapeRender {
    pub client_id: ClientId,
    node: Gd<Node2D>,

    pub entity_renders: AHashMap<EntityId, EntityRender>,
}
impl BattlescapeRender {
    pub fn new(client_id: ClientId, node: Gd<Node2D>) -> Self {
        Self {
            client_id,
            node,
            entity_renders: Default::default(),
        }
    }

    pub fn handle_event(&mut self, event: &mut ClientBattlescapeEventHandler) {
        for (entity_id, entity_render) in event.render.new_entities.drain(..) {
            entity_render.insert_to_scene(&self.node);
            self.entity_renders.insert(entity_id, entity_render);
        }

        for entity_id in event.render.removed_entities.drain(..) {
            self.entity_renders.remove(&entity_id);
        }

        for (entity_id, hull_index) in event.render.removed_hull.drain(..) {
            if let Some(entity_render) = self.entity_renders.get_mut(&entity_id) {
                entity_render.hulls[hull_index] = None;
            }
        }
    }

    pub fn draw_lerp(
        &mut self,
        from: &ClientBattlescapeEventHandler,
        to: &ClientBattlescapeEventHandler,
        weight: f32,
    ) {
        let from = &from.render.entity_snapshots;
        let to = &to.render.entity_snapshots;

        // Update positions.
        for (entity_id, render_entity) in self.entity_renders.iter_mut() {
            if let Some((from, to)) = from
                .get(entity_id)
                .and_then(|from| to.get(entity_id).map(|to| (from, to)))
            {
                let position = from.position.lerp(&to.position, weight);
                render_entity
                    .node
                    .set_position(position.pos.to_godot_scaled());
                render_entity.node.set_rotation(position.rot as f64);

                for ((render_hull, from), to) in render_entity
                    .hulls
                    .iter_mut()
                    .zip(&from.hulls)
                    .zip(&to.hulls)
                {
                    let render_hull = if let Some(render_hull) = render_hull {
                        render_hull
                    } else {
                        continue;
                    };

                    let from = if let Some(from) = from {
                        from
                    } else {
                        continue;
                    };

                    let to = if let Some(to) = to {
                        to
                    } else {
                        continue;
                    };

                    let position = from.position.lerp(&to.position, weight);
                    render_hull
                        .sprite
                        .set_position(position.pos.to_godot_scaled());
                    render_hull.sprite.set_rotation(position.rot as f64);
                }
            }
        }
    }
}
