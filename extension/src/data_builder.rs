use super::*;
use crate::{
    battlescape::{
        entity::{script::EntityScriptData, *},
        physics::builder::*,
    },
    client_battlescape::EntityRenderData,
    metascape::ship::ShipData,
    util::*,
};
use godot::{engine::Texture2D, prelude::*};

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct EntityDataBuilder {
    path: String,
    entity_data: EntityData,
    entity_render_data: EntityRenderData,
    ship_data: Option<ShipData>,
    #[base]
    base: Base<Resource>,
}
#[godot_api]
impl EntityDataBuilder {
    #[func]
    fn build(&mut self) {
        let entity_data_id = Data::add_entity(
            self.path.clone(),
            std::mem::take(&mut self.entity_data),
            std::mem::take(&mut self.entity_render_data),
        );

        if let Some(mut ship_data) = self.ship_data.take() {
            ship_data.entity_data_id = entity_data_id;
            Data::add_ship(self.path.clone(), ship_data);
        }
    }

    #[func]
    fn set_path(&mut self, path: GodotString) {
        self.path = path.to_string();
    }

    #[func]
    fn set_linear_acceleration(&mut self, linear_acceleration: f32) {
        self.entity_data.mobility.linear_acceleration = linear_acceleration;
    }

    #[func]
    fn set_angular_acceleration(&mut self, angular_acceleration: f32) {
        self.entity_data.mobility.angular_acceleration = angular_acceleration;
    }

    #[func]
    fn set_max_linear_velocity(&mut self, max_linear_velocity: f32) {
        self.entity_data.mobility.max_linear_velocity = max_linear_velocity;
    }

    #[func]
    fn set_max_angular_velocity(&mut self, max_angular_velocity: f32) {
        self.entity_data.mobility.max_angular_velocity = max_angular_velocity;
    }

    #[func]
    fn set_simulation_script(&mut self, script: Variant) {
        self.entity_data.script = EntityScriptData::new(script);
    }

    #[func]
    fn set_hull(&mut self, hull: i32) {
        self.entity_data.defence.hull = hull;
    }

    #[func]
    fn set_armor(&mut self, armor: i32) {
        self.entity_data.defence.armor = armor;
    }

    #[func]
    fn set_render_scene(
        &mut self,
        render_scene: Gd<PackedScene>,
        position_offset: Vector2,
        rotation_offset: f32,
    ) {
        self.entity_render_data.render_scene = render_scene;
        self.entity_render_data.position_offset = position_offset;
        self.entity_render_data.rotation_offset = rotation_offset;
    }

    #[func]
    fn set_aproximate_radius(&mut self, radius_aprox: f32) {
        self.entity_render_data.radius_aprox = radius_aprox;
    }

    #[func]
    fn set_shape_circle(&mut self, radius: f32, density: f32, entity_type: i64) {
        self.entity_data.collider = ball_collider(radius, density, groups(entity_type));
    }

    #[func]
    fn set_shape_cuboid(&mut self, half_size: Vector2, density: f32, entity_type: i64) {
        let half_size = half_size.to_na_descaled();
        self.entity_data.collider =
            cuboid_collider(half_size.x, half_size.y, density, groups(entity_type));
    }

    #[func]
    fn set_shape_polygon(&mut self, points: PackedVector2Array, density: f32, entity_type: i64) {
        let vertices = points
            .to_vec()
            .into_iter()
            .map(|v| {
                let v = v.to_na_descaled();
                na::Point2::new(v.x, v.y)
            })
            .collect::<Vec<_>>();
        self.entity_data.collider = polygon_collider(&vertices, density, groups(entity_type));
    }

    #[func]
    fn set_ship_display_name(&mut self, display_name: GodotString) {
        self.ship_data.get_or_insert_default().display_name = display_name.to_string();
        self.entity_data.is_ship = true;
    }

    #[func]
    fn set_ship_texture(&mut self, texture: Gd<Texture2D>) {
        self.ship_data.get_or_insert_default().texture = texture;
    }
}
#[godot_api]
impl GodotExt for EntityDataBuilder {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            path: Default::default(),
            entity_data: Default::default(),
            entity_render_data: Default::default(),
            ship_data: None,
            base,
        }
    }
}

// TODO: Export enum.
fn groups(entity_type: i64) -> Groups {
    match entity_type {
        0 => Groups::Ship,
        _ => unreachable!(),
    }
}
