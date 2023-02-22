use super::*;
use crate::battlescape::events::BattlescapeEventHandlerTrait;
use crate::battlescape::*;
use crate::util::*;
use crate::EntityDataId;
use godot::engine::packed_scene::GenEditState;
use godot::engine::Sprite2D;
use godot::prelude::*;

#[derive(Default)]
pub struct RenderBattlescapeEventHandler {
    pub take_full: bool,
    entity_snapshots: AHashMap<EntityId, EntitySnapshot>,
    new_entities: Vec<(EntityId, EntityRender)>,
    removed_entities: Vec<EntityId>,
}
impl BattlescapeEventHandlerTrait for RenderBattlescapeEventHandler {
    fn stepped(&mut self, bc: &Battlescape) {
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
                            pos: rb.translation().to_godot_scaled(),
                            rot: rb.rotation().angle(),
                        },
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

    fn fleet_added(&mut self, _fleet_id: crate::FleetId) {}

    fn ship_state_changed(
        &mut self,
        _fleet_id: FleetId,
        _ship_idx: usize,
        _state: bc_fleet::FleetShipState,
    ) {
    }

    fn entity_removed(&mut self, entity_id: EntityId, _entity: entity::Entity) {
        if !self.take_full {
            self.removed_entities.push(entity_id);
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub pos: Vector2,
    pub rot: f32,
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
}

pub struct EntityRender {
    /// The entity position with its sprite offset.
    pub position: Position,
    sprite: Gd<Sprite2D>,
    pub entity_data_id: EntityDataId,
    pub fleet_ship: Option<FleetShip>,
}
impl EntityRender {
    /// Will not be added to the scene.
    /// `node` need to manualy free if this is drop before a call to `insert_to_scene`.
    fn new(entity: &entity::Entity) -> Self {
        let sprite = entity
            .entity_data_id
            .render_data()
            .render_scene
            .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
            .map(|node| node.cast::<Sprite2D>())
            .unwrap_or_else(|| Sprite2D::new_alloc());

        // TODO: Set hull shader.

        EntityRender {
            sprite,
            position: Default::default(),
            entity_data_id: entity.entity_data_id,
            fleet_ship: entity.fleet_ship,
        }
    }

    fn set_position(&mut self, new_position: Position) {
        self.position.pos = new_position.pos;
        self.position.rot = new_position.rot;

        self.sprite
            .set_position(new_position.pos + self.entity_data_id.render_data().position_offset);
        self.sprite.set_rotation(
            (new_position.rot + self.entity_data_id.render_data().rotation_offset) as f64,
        );
    }

    fn insert_to_scene(&self, draw_node: &Gd<Node2D>) {
        add_child(draw_node, &self.sprite);
    }
}
impl Drop for EntityRender {
    fn drop(&mut self) {
        self.sprite.queue_free()
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
                let new_position = from.position.lerp(&to.position, weight);
                render_entity.set_position(new_position);
            }
        }
    }
}
