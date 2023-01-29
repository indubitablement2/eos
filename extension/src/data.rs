use std::hash::Hash;

use super::battlescape::entity::*;
use super::metascape::ship::ShipData;
use super::*;
use crate::util::*;
use godot::engine::packed_scene::GenEditState;
use godot::engine::{
    CircleShape2D, CollisionPolygon2D, CollisionShape2D, RectangleShape2D, Script,
};
use godot::prelude::*;
use rapier2d::prelude::SharedShape;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShipDataId(u32);
impl ShipDataId {
    pub fn data(self) -> &'static ShipData {
        &Data::data().ships[self.0 as usize]
    }
}

/// An entity that can be spawned in a battlescape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityDataId(u32);
impl EntityDataId {
    pub fn data(self) -> &'static EntityData {
        &Data::data().entities[self.0 as usize]
    }
}

pub struct ShipDataIter {
    next_id: u32,
    len: u32,
}
impl Iterator for ShipDataIter {
    type Item = (ShipDataId, &'static ShipData);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_id < self.len {
            let id = ShipDataId(self.next_id);
            self.next_id += 1;
            Some((id, id.data()))
        } else {
            None
        }
    }
}
pub fn ship_data_iter() -> ShipDataIter {
    ShipDataIter {
        next_id: 0,
        len: Data::data().ships.len() as u32,
    }
}

pub struct Data {
    ships: IndexMap<String, ShipData, RandomState>,
    entities: IndexMap<String, EntityData, RandomState>,
}
impl Data {
    /// Free all resources.
    /// ## Safety:
    /// Data should not be in use.
    pub fn clear() {
        unsafe {
            DATA = None;
        }
    }

    /// Return None on error that should not occur.
    pub fn try_load_data(path: GodotString) -> Option<()> {
        let data = Data::data_mut();
        let string_path = path.to_string();
        let node =
            try_load::<PackedScene>(path)?.instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)?;

        if node.has_method("_is_ship_data".into()) {
            if data.ships.contains_key(&string_path) {
                node.free();
            } else {
                let (mut ship_data, entity_path) = data.parse_ship_data(node)?;
                let entity_data_idx = data.parse_entity_data(entity_path)?;
                ship_data.entity_data_id = EntityDataId(entity_data_idx as u32);
                log::info!("Added ship from '{}'", string_path);
                data.ships.insert(string_path, ship_data);
            }
        }

        Some(())
    }

    fn parse_ship_data(&mut self, node: Gd<Node>) -> Option<(ShipData, String)> {
        log::debug!("Parsing ship data");

        let entity_path = node
            .get("entity_path".into())
            .try_to::<GodotString>()
            .ok()?
            .to_string();

        Some((
            ShipData {
                display_name: node
                    .get("display_name".into())
                    .try_to::<GodotString>()
                    .ok()?
                    .to_string(),
                render_node: node.try_cast()?,
                entity_data_id: EntityDataId(0),
            },
            entity_path,
        ))
    }

    /// Return the index of the entity data.
    fn parse_entity_data(&mut self, entity_path: String) -> Option<usize> {
        if let Some(idx) = self.entities.get_index_of(&entity_path) {
            return Some(idx);
        }

        log::debug!("Parsing entity data at '{}'", entity_path);

        let mut node = try_load::<PackedScene>(entity_path.to_string())?
            .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)?;

        // Find the hulls nodes.
        let mut hulls: SmallVec<[HullData; 1]> = SmallVec::new();
        for (mut child_node, render_node_idx) in node.children_iter().zip(0i64..) {
            if !child_node.has_method("_is_hull_data".into()) {
                continue;
            }

            log::debug!("Parsing hull data at Node #{}", render_node_idx);

            let mut shape = SharedShape::ball(0.5);
            let mut init_position = rapier2d::prelude::Isometry::default();
            for child_child_node in child_node.children_iter() {
                if let Some(collision_node) =
                    child_child_node.share().try_cast::<CollisionShape2D>()
                {
                    log::debug!("Parsing hull's CollisionShape2D");

                    let shape_node = collision_node.get_shape()?;

                    init_position = rapier2d::prelude::Isometry::new(
                        collision_node.get_position().to_na_descaled(),
                        collision_node.get_rotation() as f32,
                    );
                    log::trace!("Got initial position of {:?}", &init_position);

                    if let Some(circle_shape) = shape_node.share().try_cast::<CircleShape2D>() {
                        let radius = circle_shape.get_radius() as f32 / GODOT_SCALE;
                        log::trace!("Got CircleShape2D with radius of {}", radius);
                        shape = SharedShape::ball(radius);
                    } else if let Some(rectangle_shape) = shape_node.try_cast::<RectangleShape2D>()
                    {
                        let size = rectangle_shape.get_size().to_na_descaled();
                        log::trace!("Got RectangleShape2D with size of {:?}", size);
                        shape = SharedShape::cuboid(size.x, size.y);
                    }

                    // Remove collision shape node.
                    log::debug!("Removing child CollisionShape2D");
                    child_node.remove_child(collision_node.share().upcast());
                    collision_node.free();

                    break;
                } else if let Some(collision_poly) =
                    child_child_node.try_cast::<CollisionPolygon2D>()
                {
                    log::debug!("Parsing hull's CollisionPolygon2D");

                    // TODO: Handle poly when array are supported.
                    // TODO: (GODOT_SCALE)
                    // TODO: empty poly
                    collision_poly.get_polygon();

                    // let vertices = vertices
                    //     .iter()
                    //     .map(|v| na::point![v.x, v.y])
                    //     .collect::<Vec<_>>();

                    // let indices = (0..vertices.len() as u32 - 1)
                    //     .map(|i| [i, i + 1])
                    //     .collect::<Vec<_>>();
                    // SharedShape::convex_decomposition(&vertices, indices.as_slice())

                    log::warn!("poly not supported yet");

                    // Remove collision poly node.
                    log::debug!("Removing child CollisionPolygon2D");
                    child_node.remove_child(collision_poly.share().upcast());
                    collision_poly.free();

                    break;
                }
            }

            let script = validate_script(child_node.get("simulation_script".into()), "HullScript");

            hulls.push(HullData {
                defence: Defence {
                    hull: child_node.get("hull".into()).try_to().ok()?,
                    armor: child_node.get("armor".into()).try_to().ok()?,
                },
                shape,
                init_position,
                density: child_node.get("density".into()).try_to().ok()?,
                render_node_idx,
                script,
            });

            log::debug!("Replacing hull data script with render script");
            let render_script = child_node.get("render_script".into());
            child_node.set_script(render_script);
        }

        if hulls.is_empty() {
            node.free();
            log::warn!("Entity data without hull not supported. Ignoring...");
            return None;
        }

        let script = validate_script(node.get("simulation_script".into()), "EntityScript");

        let mut entity_data = EntityData {
            mobility: Mobility {
                linear_acceleration: node
                    .get("linear_acceleration".into())
                    .try_to::<f32>()
                    .ok()?
                    / GODOT_SCALE,
                angular_acceleration: node
                    .get("angular_acceleration".into())
                    .try_to::<f32>()
                    .ok()?
                    / GODOT_SCALE,
                max_linear_velocity: node
                    .get("max_linear_velocity".into())
                    .try_to::<f32>()
                    .ok()?
                    / GODOT_SCALE,
                max_angular_velocity: node
                    .get("max_angular_velocity".into())
                    .try_to::<f32>()
                    .ok()?
                    / GODOT_SCALE,
            },
            hulls,
            ai: None, // TODO: Initial ai
            render_node: PackedScene::new(),
            script,
        };

        log::debug!("Replacing entity data script with render script");
        let render_script = node.get("render_script".into());
        node.set_script(render_script);

        // TODO: Check that this is Ok. Otherwise return None
        entity_data.render_node.pack(node.share());
        node.free();

        Some(self.entities.insert_full(entity_path, entity_data).0)
    }

    fn data() -> &'static Data {
        unsafe { DATA.get_or_insert_default() }
    }

    fn data_mut() -> &'static mut Data {
        unsafe { DATA.get_or_insert_default() }
    }
}
impl Default for Data {
    fn default() -> Self {
        Self {
            ships: Default::default(),
            entities: Default::default(),
        }
    }
}

static mut DATA: Option<Data> = None;

struct ChildIter {
    childs: TypedArray<Gd<Node>>,
    i: i64,
    len: i64,
}
impl Iterator for ChildIter {
    type Item = Gd<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.len {
            let r = self.childs.get(self.i);
            self.i += 1;
            r
        } else {
            None
        }
    }
}
trait ChildIterTrait {
    fn children_iter(&self) -> ChildIter;
}
impl ChildIterTrait for Gd<Node> {
    fn children_iter(&self) -> ChildIter {
        log::trace!("Iterating over children");

        ChildIter {
            childs: self.get_children(false),
            i: 0,
            len: self.get_child_count(false),
        }
    }
}

fn validate_script(script: Variant, extend: &str) -> Variant {
    if script.is_nil() {
        log::debug!("Can not validate nil script. TODO: remove thing");
        return script;
    }

    if let Ok(gd_script) = script.try_to::<Gd<Script>>() {
        let base_type = gd_script.get_instance_base_type().to_string();
        if base_type.as_str() == extend {
            log::debug!("Simulation script validated");
            script
        } else {
            log::warn!(
                "Expected simulation script to extend '{}', got '{}' instead. Removing...",
                extend,
                base_type
            );
            Variant::nil()
        }
    } else {
        Variant::nil()
    }
}
