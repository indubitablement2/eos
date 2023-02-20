use super::*;
use crate::{
    battlescape::entity::{script::EntityScriptData, *},
    metascape::ship::ShipData,
    util::*,
};
use godot::{engine::Texture2D, prelude::*};
use rapier2d::prelude::SharedShape;

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct ShipDataBuilder {
    path: String,
    ship_data: ShipData,
    handled: bool,
    #[base]
    base: Base<Resource>,
}
#[godot_api]
impl ShipDataBuilder {
    #[func]
    fn build(&mut self) {
        assert!(!self.handled);
        self.handled = true;

        Data::add_ship(
            std::mem::take(&mut self.path.clone()),
            std::mem::take(&mut self.ship_data),
        );
    }

    #[func]
    fn set_path(&mut self, path: GodotString) {
        self.path = path.to_string();
    }

    #[func]
    fn set_entity_data_path(&mut self, entity_data_path: GodotString) {
        self.ship_data.entity_data_id = Data::entity_data_from_path(entity_data_path.to_string())
            .unwrap()
            .0;
    }

    #[func]
    fn set_display_name(&mut self, display_name: GodotString) {
        self.ship_data.display_name = display_name.to_string();
    }

    #[func]
    fn set_texture(&mut self, texture: Gd<Texture2D>) {
        self.ship_data.texture = texture;
    }
}
#[godot_api]
impl GodotExt for ShipDataBuilder {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            path: Default::default(),
            ship_data: Default::default(),
            handled: false,
            base,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct EntityDataBuilder {
    path: String,
    entity_data: EntityData,
    hulls: Vec<Gd<HullDataBuilder>>,
    handled: bool,
    #[base]
    base: Base<Resource>,
}
#[godot_api]
impl EntityDataBuilder {
    #[func]
    fn build(&mut self) {
        assert!(!self.handled);
        self.handled = true;

        if !self.hulls.is_empty() {
            self.entity_data.hulls = self
                .hulls
                .drain(..)
                .map(|mut hull| hull.bind_mut().finish())
                .collect();
        }

        Data::add_entity(self.path.clone(), std::mem::take(&mut self.entity_data));
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
    fn set_render_scene(&mut self, render_scene: Gd<PackedScene>) {
        self.entity_data.render_scene = render_scene;
    }

    #[func]
    fn set_simulation_script(&mut self, script: Variant) {
        self.entity_data.script = EntityScriptData::new(script);
    }

    #[func]
    fn set_aproximate_radius(&mut self, radius_aprox: f32) {
        self.entity_data.radius_aprox = radius_aprox;
    }

    #[func]
    fn add_hull(&mut self, hull: Gd<HullDataBuilder>) {
        self.hulls.push(hull);
    }
}
#[godot_api]
impl GodotExt for EntityDataBuilder {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            path: Default::default(),
            entity_data: Default::default(),
            hulls: Default::default(),
            handled: false,
            base,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Resource)]
struct HullDataBuilder {
    hull_data: HullData,
    handled: bool,
    #[base]
    base: Base<Resource>,
}
impl HullDataBuilder {
    fn new(render_node_idx: i64) -> Gd<Self> {
        Gd::with_base(|base| Self {
            hull_data: HullData {
                render_node_idx,
                ..Default::default()
            },
            handled: false,
            base,
        })
    }

    fn finish(&mut self) -> HullData {
        assert!(!self.handled);
        self.handled = true;

        std::mem::take(&mut self.hull_data)
    }
}
#[godot_api]
impl HullDataBuilder {
    #[func]
    fn set_render_node_idx(&mut self, render_node_idx: i64) {
        self.hull_data.render_node_idx = render_node_idx;
    }

    #[func]
    fn set_hull(&mut self, hull: i32) {
        self.hull_data.defence.hull = hull;
    }

    #[func]
    fn set_armor(&mut self, armor: i32) {
        self.hull_data.defence.armor = armor;
    }

    #[func]
    fn set_density(&mut self, density: f32) {
        self.hull_data.density = density;
    }

    #[func]
    fn set_initial_position(&mut self, position: Vector2, rotation: f32) {
        self.hull_data.init_position = na::Isometry2::new(position.to_na_descaled(), rotation);
    }

    #[func]
    fn set_shape_circle(&mut self, radius: f32) {
        self.hull_data.shape = SharedShape::ball(radius / GODOT_SCALE);
    }

    #[func]
    fn set_shape_cuboid(&mut self, half_size: Vector2) {
        let half_size = half_size.to_na_descaled();
        self.hull_data.shape = SharedShape::cuboid(half_size.x, half_size.y);
    }

    #[func]
    fn set_shape_polygon(&mut self, points: PackedVector2Array) {
        let vertices = points
            .to_vec()
            .into_iter()
            .map(|v| {
                let v = v.to_na_descaled();
                na::Point2::new(v.x, v.y)
            })
            .collect::<Vec<_>>();

        if vertices.len() < 3 {
            log::warn!("Polygon must have at least 3 vertices");
            return;
        }

        let indices = (0..vertices.len() as u32 - 1)
            .map(|i| [i, i + 1])
            .collect::<Vec<_>>();

        self.hull_data.shape = SharedShape::convex_decomposition(&vertices, indices.as_slice());
    }
}
#[godot_api]
impl GodotExt for HullDataBuilder {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            hull_data: Default::default(),
            handled: false,
            base,
        }
    }
}
