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
    client_id: ClientId,

    team: Option<Team>,
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

        // Try to get our team.
        for fleet in bs.fleets.values() {
            if fleet.owner.is_some_and(|owner| owner == self.client_id) {
                self.team = Some(fleet.team);
                break;
            }
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
        position: na::Isometry2<f32>,
    ) {
        if !self.take_full {
            self.new_entities.push((
                entity_id,
                InitEntityRender::new(entity, Position::new(position)),
            ));
        }
    }

    fn battle_over(&mut self) {}
}
impl RenderBattlescapeEventHandler {
    pub fn new(take_full: bool, client_id: ClientId) -> Self {
        Self {
            take_full,
            client_id,
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
            // Use hardware acceleration.
            rot: iso.rotation.im.atan2(iso.rotation.re),
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
    team: Team,
    owner: Option<ClientId>,
    position: Position,
    entity_data_id: EntityDataId,
    fleet_ship: Option<FleetShip>,
}
impl InitEntityRender {
    fn new(entity: &Entity, position: Position) -> Self {
        Self {
            team: entity.team,
            owner: entity.owner,
            position,
            entity_data_id: entity.entity_data_id,
            fleet_ship: entity.fleet_ship,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityRelation {
    Owned,
    Friendly,
    Neutral,
    Enemy,
}

pub struct EntityRender {
    pub relation: EntityRelation,
    pub team: Team,
    pub owner: Option<ClientId>,
    base: Gd<Node2D>,
    sprite: Gd<Sprite2D>,
    draw: Gd<EntityDraw>,
    pub entity_data_id: EntityDataId,
    pub fleet_ship: Option<FleetShip>,
}
impl EntityRender {
    fn new(init: InitEntityRender, draw_node: &Gd<Node2D>) -> Self {
        let base = init
            .entity_data_id
            .render_data()
            .render_scene
            .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
            .map(|node| node.cast::<Node2D>())
            .unwrap();

        add_child(draw_node, &base);

        // TODO: Set hull shader.

        let draw = EntityDraw::new(
            &base,
            init.entity_data_id,
            Color::from_rgba(1.0, 0.0, 0.0, 0.1),
            true,
        );

        let mut s = EntityRender {
            relation: EntityRelation::Neutral,
            team: init.team,
            owner: init.owner,
            sprite: base
                .get_child(init.entity_data_id.render_data().child_sprite_idx, false)
                .unwrap()
                .cast(),
            base,
            draw,
            entity_data_id: init.entity_data_id,
            fleet_ship: init.fleet_ship,
        };

        s.set_position(init.position);

        s
    }

    fn set_position(&mut self, new_position: Position) {
        self.base.set_position(new_position.pos);
        self.base.set_rotation(new_position.rot as f64);
    }

    fn update_relation(&mut self, client_id: ClientId, team: Team) {
        fn update_col(draw: &mut Gd<EntityDraw>, color: Color) {
            let mut b = draw.bind_mut();
            b.draw_color = color;
            b.queue_redraw();
        }

        if let Some(owner) = self.owner {
            if owner == client_id {
                self.relation = EntityRelation::Owned;
                update_col(&mut self.draw, Color::from_rgb(0.0, 1.0, 0.0));
            } else if self.team == team {
                self.relation = EntityRelation::Friendly;
                update_col(&mut self.draw, Color::from_rgb(1.0, 0.5, 0.0));
            } else {
                self.relation = EntityRelation::Enemy;
                update_col(&mut self.draw, Color::from_rgb(1.0, 0.0, 0.0));
            }
        } else {
            self.relation = EntityRelation::Neutral;
            update_col(&mut self.draw, Color::from_rgb(0.5, 0.5, 0.5));
        }
    }

    pub fn position(&self) -> Vector2 {
        self.draw.bind().get_global_position()
    }
}
impl Drop for EntityRender {
    fn drop(&mut self) {
        self.base.queue_free()
    }
}
// Needed as it is contructed in events.
unsafe impl Send for EntityRender {}

pub struct BattlescapeRender {
    client_id: ClientId,
    team: Option<Team>,
    node: Gd<Node2D>,

    pub entity_renders: AHashMap<EntityId, EntityRender>,
}
impl BattlescapeRender {
    pub fn new(client_id: ClientId, node: Gd<Node2D>) -> Self {
        Self {
            client_id,
            team: None,
            node,
            entity_renders: Default::default(),
        }
    }

    pub fn handle_event(&mut self, event: &mut ClientBattlescapeEventHandler) {
        if let Some(team) = event.render.team {
            self.team = Some(team);
            for entity in self.entity_renders.values_mut() {
                entity.update_relation(self.client_id, team);
            }
        }

        for (entity_id, init) in event.render.new_entities.drain(..) {
            let mut entity_render = EntityRender::new(init, &self.node);
            if let Some(team) = self.team {
                entity_render.update_relation(self.client_id, team);
            }
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

#[derive(GodotClass)]
#[class(base=Node2D)]
struct EntityDraw {
    entity_data_id: EntityDataId,
    draw_rect: bool,
    draw_color: Color,
    #[base]
    base: Base<Node2D>,
}
impl EntityDraw {
    fn new(
        parent: &Gd<Node2D>,
        entity_data_id: EntityDataId,
        draw_color: Color,
        draw_rect: bool,
    ) -> Gd<Self> {
        Gd::<Self>::with_base(|mut base| {
            add_child(parent, &base);

            let obj = base.share();
            base.connect(
                "draw".into(),
                Callable::from_object_method(obj, "__draw"),
                0,
            );

            Self {
                entity_data_id,
                draw_rect,
                draw_color,
                base,
            }
        })
    }
}
#[godot_api]
impl EntityDraw {
    // TODO: Remove when _draw() is implemented.
    #[func]
    fn __draw(&mut self) {
        let r = self.entity_data_id.render_data().radius_aprox;

        if self.draw_rect {
            // self.base.draw_set_transform(
            //     -self.entity_data_id.render_data().position_offset,
            //     -self.entity_data_id.render_data().rotation_offset as f64,
            //     Vector2::ONE,
            // );

            self.base
                .draw_circle(Vector2::ZERO, r as f64, self.draw_color.with_alpha(0.1));
            // let rect = Rect2{ opaque: todo!() }
            // self.draw_rect(rect, self.color, false, 2.0);
        }
    }
}
