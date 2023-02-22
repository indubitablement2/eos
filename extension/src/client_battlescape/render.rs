use super::*;
use crate::battlescape::entity::Entity;
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
    new_entities: Vec<(EntityId, InitEntityRender)>,
    removed_entities: Vec<EntityId>,
}
impl BattlescapeEventHandlerTrait for RenderBattlescapeEventHandler {
    fn stepped(&mut self, bs: &Battlescape) {
        // Take the position of all entities and their hulls.
        self.entity_snapshots = bs
            .entities
            .keys()
            .map(|entity_id| {
                (
                    *entity_id,
                    EntitySnapshot {
                        position: Position::new(bs.entity_position(*entity_id)),
                    },
                )
            })
            .collect();

        if self.take_full {
            // Take a full snapshot of the battlescape.
            self.new_entities = bs
                .entities
                .iter()
                .map(|(entity_id, entity)| {
                    (
                        *entity_id,
                        InitEntityRender::new(
                            entity,
                            Position::new(bs.entity_position(*entity_id)),
                        ),
                    )
                })
                .collect();
        }
    }

    fn fleet_added(&mut self, _fleet_id: crate::FleetId) {}

    fn ship_state_changed(
        &mut self,
        _fleet_id: FleetId,
        _ship_idx: usize,
        _state: bs_fleet::FleetShipState,
    ) {
    }

    fn entity_removed(&mut self, entity_id: EntityId, _entity: entity::Entity) {
        if !self.take_full {
            self.removed_entities.push(entity_id);
        }
    }

    fn entity_added(
        &mut self,
        entity_id: EntityId,
        entity: &entity::Entity,
        translation: na::Vector2<f32>,
        angle: f32,
    ) {
        if !self.take_full {
            self.new_entities.push((
                entity_id,
                InitEntityRender::new(entity, Position::new2(translation, angle)),
            ));
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
    fn new(iso: na::Isometry2<f32>) -> Self {
        Self {
            pos: iso.translation.to_godot_scaled(),
            // TODO: Use hardware acceleration.
            rot: iso.rotation.im.atan2(iso.rotation.re),
        }
    }

    fn new2(translation: na::Vector2<f32>, angle: f32) -> Self {
        Self {
            pos: translation.to_godot_scaled(),
            rot: angle,
        }
    }

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

struct InitEntityRender {
    pub position: Position,
    pub entity_data_id: EntityDataId,
    pub fleet_ship: Option<FleetShip>,
}
impl InitEntityRender {
    fn new(entity: &Entity, position: Position) -> Self {
        Self {
            position,
            entity_data_id: entity.entity_data_id,
            fleet_ship: entity.fleet_ship,
        }
    }
}

pub struct EntityRender {
    /// The entity position with its sprite offset.
    pub position: Position,
    sprite: Gd<Sprite2D>,
    pub entity_data_id: EntityDataId,
    pub fleet_ship: Option<FleetShip>,
}
impl EntityRender {
    fn new(init: InitEntityRender, draw_node: &Gd<Node2D>) -> Self {
        let sprite = init
            .entity_data_id
            .render_data()
            .render_scene
            .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
            .map(|node| node.cast::<Sprite2D>())
            .unwrap_or_else(|| Sprite2D::new_alloc());

        add_child(draw_node, &sprite);

        // TODO: Set hull shader.

        let mut s = EntityRender {
            sprite,
            position: Default::default(),
            entity_data_id: init.entity_data_id,
            fleet_ship: init.fleet_ship,
        };

        s.set_position(init.position);

        s
    }

    fn set_position(&mut self, new_position: Position) {
        self.position = new_position;

        self.sprite
            .set_position(new_position.pos + self.entity_data_id.render_data().position_offset);
        self.sprite.set_rotation(
            (new_position.rot + self.entity_data_id.render_data().rotation_offset) as f64,
        );
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
        for (entity_id, init) in event.render.new_entities.drain(..) {
            let entity_render = EntityRender::new(init, &self.node);
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
