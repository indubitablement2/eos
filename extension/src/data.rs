use std::hash::Hash;

use crate::battlescape::entity::script::ScriptWrapper;
use crate::util::*;
use super::battlescape::entity::*;
use super::metascape::ship::ShipData;
use super::*;
use godot::engine::packed_scene::GenEditState;
use godot::engine::{
    CircleShape2D, CollisionPolygon2D, CollisionShape2D, RectangleShape2D, Sprite2D, Texture2D,
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

pub struct Data {
    error_texture: Gd<Texture2D>,
    ships: IndexMap<String, ShipData, RandomState>,
    entities: IndexMap<String, EntityData, RandomState>,
}
impl Data {
    /// ## Safety:
    /// Data should not be in use.
    pub fn reset() {
        unsafe {
            DATA = Some(Default::default());
        }
    }

    // TODO: Call this when client exit
    /// Free all resources.
    pub fn clear() {
        unsafe {
            DATA = None;
        }
    }

    /// Return None on error that should not occur.
    pub fn try_load_data(path: GodotString) -> Option<()> {
        let data = Data::data_mut();
        let string_path = path.to_string();
        let node = try_load::<PackedScene>(path)?
            .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)?;

        if node.has_method("_is_ship_data".into()) {
            if !data.ships.contains_key(&string_path) {
                let (mut ship_data, entity_path) = data.parse_ship_data(node)?;
                let entity_data_idx = data.parse_entity_data(entity_path)?;
                ship_data.entity_data_id = EntityDataId(entity_data_idx as u32);
                log::info!("Added ship at '{}'.", string_path);
                data.ships.insert(string_path, ship_data);
            }
        }

        Some(())
    }

    fn parse_ship_data(&mut self, node: Gd<Node>) -> Option<(ShipData, String)> {
        let entity_path = node
            .get("entity_path".into())
            .try_to::<GodotString>()
            .ok()?
            .to_string();
        
        Some((
            ShipData {
                display_name: node.get("display_name".into()).try_to::<GodotString>().ok()?.to_string(),
                render: node.try_cast()?,
                entity_data_id: EntityDataId(0),
            },
            entity_path
        ))
    }

    /// Return the index of the entity data.
    fn parse_entity_data(&mut self, entity_path: String) -> Option<usize> {
        self.entities.get_index_of(&entity_path).or_else(|| {
            let mut node = try_load::<PackedScene>(entity_path.to_string())?
                .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)?;
            
            // Find the hulls nodes.
            let mut hulls: SmallVec<[HullData; 1]> = SmallVec::new();
            for (mut child_node, render_node_idx) in node.children_iter().zip(0i64..) {
                if !child_node.has_method("_is_hull_data".into()) {
                    continue;
                }

                let mut shape = SharedShape::ball(0.5);
                let mut init_position = rapier2d::prelude::Isometry::default();
                for child_child_node in child_node.children_iter() {
                    if let Some(collision_node) = child_child_node.share().try_cast::<CollisionShape2D>() {
                        let shape_node = collision_node.get_shape()?;
                        
                        init_position = rapier2d::prelude::Isometry::new(
                            collision_node.get_position().to_na_descaled(),
                            collision_node.get_rotation() as f32,
                        );

                        if let Some(circle_shape) = shape_node.share().try_cast::<CircleShape2D>() {
                            let radius = circle_shape.get_radius() as f32 / GODOT_SCALE;
                            shape = SharedShape::ball(radius);
                        } else if let Some(rectangle_shape) = shape_node.try_cast::<RectangleShape2D>() {
                            let size = rectangle_shape.get_size().to_na_descaled();
                            shape = SharedShape::cuboid(size.x, size.y);
                        }
                        
                        // Remove collision shape node.
                        child_node.remove_child(collision_node.upcast());

                        break;
                    } else if let Some(collision_poly) = child_child_node.try_cast::<CollisionPolygon2D>() {
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
                        child_node.remove_child(collision_poly.upcast());

                        break;
                    }
                }

                hulls.push(HullData {
                    defence: Defence {
                        hull: child_node.get("hull".into()).try_to().ok()?,
                        armor: child_node.get("armor".into()).try_to().ok()?,
                    },
                    shape,
                    init_position,
                    density: child_node.get("density".into()).try_to().ok()?,
                    render_node_idx,
                    script: ScriptWrapper::new_hull(child_node.get("simulation_script".into()).try_to().ok()?),
                });

                let render_script = child_node.get("render_script".into());
                child_node.set_script(render_script);
            }

            let entity_data = EntityData {
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
                node: node.share().try_cast()?,
                script: ScriptWrapper::new_entity(node.get("simulation_script".into()).try_to().ok()?),
            };

            let render_script = node.get("render_script".into());
            node.set_script(render_script);
            
            Some(self.entities.insert_full(entity_path, entity_data).0)
        })
    }

    pub fn load_data(path: &str) {
        // let data = Data::data_mut();
        // let s = load::<PackedScene>(path);
        // let mut node = s
        //     .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
        //     .unwrap();

        // if node.has_method("_is_ship_data".into()) {
        //     let display_name = node
        //         .get("get_display_name".into())
        //         .to::<GodotString>()
        //         .to_string();
            
        //     log::debug!("0");

        //     if let Some(entity_data_id) = data.add_entity_data(&mut node, &display_name) {
        //         log::debug!("Added ShipData: '{}'.", &display_name);

        //         todo!()
        //         // data.ships.push(ShipData {
        //         //     display_name,
        //         //     texture: node
        //         //         .share()
        //         //         .cast::<Sprite2D>()
        //         //         .get_texture()
        //         //         .unwrap_or_else(|| data.error_texture.share()),
        //         //     entity_data_id,
        //         // });
        //     }
        // } else {
        //     log::warn!("Unhandled data node. Ignoring...");
        // }
    }

    // /// Find child EntityData node, detach it and add EntityData to data.
    // /// Return none if there are no EntityData node.
    // ///
    // /// `ship_name` is only to give better error messages.
    // fn add_entity_data(
    //     &mut self,
    //     parent_node: &mut Gd<Node>,
    //     ship_name: &str,
    // ) -> Option<EntityDataId> {
    //     let id = EntityDataId(self.entities.len() as u32);

    //     log::debug!("1"); // TODO: CRASH HERE <-----------

    //     // Find the entity data node.
    //     let mut entity_data_node = if let Some(entity_data_node) = parent_node
    //         .children_iter()
    //         .find(|child| child.has_method("_is_entity_data".into()))
    //     {
    //         entity_data_node
    //     } else {
    //         // We did not find entity data node.
    //         log::warn!(
    //             "ShipData '{}' does not have child EntityData node. Ignoring...",
    //             ship_name
    //         );
    //         return None;
    //     };

    //     log::debug!("2");

    //     // TODO: Replace entity data script with entity script.
    //     // Detach entity data child node.
    //     parent_node.remove_child(entity_data_node.share());

    //     log::debug!("3");

    //     // Find the hulls nodes.
    //     let hulls: SmallVec<[HullData; 1]> = entity_data_node.children_iter()
    //     .zip(0i64..)
    //     .filter_map(|(mut hull_data, node_idx)| {
    //         if hull_data.has_method("_is_hull_data".into()) {
    //             // TODO: Replace hull data script with hull script.

    //             // // Detach hull data child node.
    //             // for child in hulls_data_nodes.iter() {
    //             //     entity_data_node.remove_child(child.share());
    //             // }
    //             log::debug!("4");
    //             // Get the hull shape.
    //             let shape = hull_data
    //             .children_iter()
    //             .find_map(|child| {
    //                 child.share().try_cast::<CollisionShape2D>().map(|collision_shape| {
    //                     // Remove collision shape node.
    //                     hull_data.remove_child(child.share());

    //                     collision_shape.get_shape().map(|shape| {
    //                         shape.share().try_cast::<CircleShape2D>().map(|circle_shape| {
    //                             let radius = circle_shape.get_radius() as f32 / GODOT_SCALE;
    //                             SharedShape::ball(radius)
    //                         }).or_else(|| shape.try_cast::<RectangleShape2D>().map(|rectangle_shape| {
    //                             let size = rectangle_shape.get_size().inner() / GODOT_SCALE;
    //                             SharedShape::cuboid(size.x, size.y)
    //                         })).unwrap_or_else(|| {
    //                             log::warn!("ShipData '{}' has a hull with a CollisionShape2D with a shape other than a CircleShape2D or RectangleShape2D. Using a default circle...", ship_name);
    //                             SharedShape::ball(1.0)
    //                         })
    //                     }).unwrap_or_else(|| {
    //                         log::warn!("ShipData '{}' has a hull with a CollisionShape2D with no shape. Using a default circle...", ship_name);
    //                         SharedShape::ball(1.0)
    //                     })
    //                 }).or_else(|| {
    //                     child.share().try_cast::<CollisionPolygon2D>().map(|collision_poly| {
    //                         // Remove collision poly node.
    //                         entity_data_node.remove_child(child);

    //                         // TODO: Handle poly when array are supported.
    //                         // TODO: (GODOT_SCALE)
    //                         // TODO: empty poly
    //                         collision_poly.get_polygon();

    //                         // let vertices = vertices
    //                         //     .iter()
    //                         //     .map(|v| na::point![v.x, v.y])
    //                         //     .collect::<Vec<_>>();

    //                         // let indices = (0..vertices.len() as u32 - 1)
    //                         //     .map(|i| [i, i + 1])
    //                         //     .collect::<Vec<_>>();
    //                         // SharedShape::convex_decomposition(&vertices, indices.as_slice())

    //                         todo!("poly not supported yet")
    //                     })
    //                 })
    //             }).unwrap_or_else(|| {
    //                 log::warn!("ShipData '{}' has a hull with neither CollisionShape2D nor CollisionPolygon2D. Using a default circle...", ship_name);
    //                 SharedShape::ball(1.0)
    //             });
    //             log::debug!("5");
    //             Some(HullData {
    //                 defence: Defence {
    //                     hull: hull_data.get("hull".into()).to::<i32>(),
    //                     armor: hull_data.get("armor".into()).to::<i32>(),
    //                 },
    //                 shape,
    //                 density: hull_data.get("get_density".into()).to::<f32>(),
    //                 node_idx,
    //                 script: ScriptWrapper::new_empty_hull(),
    //             })
    //         } else {
    //             None
    //         }
    //     }).collect();

    //     if hulls.is_empty() {
    //         log::warn!("ShipData '{}' has no hull. Ignoring...", ship_name);
    //         return None;
    //     }
    //     log::debug!("6");
    //     let entity_data = EntityData {
    //         mobility: Mobility {
    //             linear_acceleration: entity_data_node
    //                 .get("linear_acceleration".into())
    //                 .to::<f32>()
    //                 / GODOT_SCALE,
    //             angular_acceleration: entity_data_node
    //                 .get("angular_acceleration".into())
    //                 .to::<f32>()
    //                 / GODOT_SCALE,
    //             max_linear_velocity: entity_data_node
    //                 .get("max_linear_velocity".into())
    //                 .to::<f32>()
    //                 / GODOT_SCALE,
    //             max_angular_velocity: entity_data_node
    //                 .get("max_angular_velocity".into())
    //                 .to::<f32>()
    //                 / GODOT_SCALE,
    //         },
    //         hulls,
    //         ai: None, // TODO: Initial ai
    //         node: entity_data_node.cast(),
    //         script: ScriptWrapper::new_empty_entity(),
    //     };
    //     // self.entities.push(entity_data);

    //     Some(id)
    // }

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
            error_texture: load("res://debug/error.png"),
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
        log::debug!("1.1");
        let childs = self.get_children(false);
        log::debug!("1.2");
        let i = 0;
        log::debug!("1.3");
        let len = self.get_child_count(false);
        log::debug!("1.4");

        ChildIter {
            childs: self.get_children(false),
            i: 0,
            len: self.get_child_count(false),
        }
    }
}
