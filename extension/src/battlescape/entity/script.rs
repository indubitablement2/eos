use super::*;
use crate::util::*;
use godot::prelude::{
    utilities::{bytes_to_var, var_to_bytes},
    *,
};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityScriptWrapper {
    serde: Option<Vec<u8>>,
    #[serde(skip)]
    #[serde(default = "default_entity_script")]
    script: Gd<EntityScript>,
    entity_data_id: EntityDataId,
}
impl EntityScriptWrapper {
    pub fn new(entity_data_id: EntityDataId) -> Self {
        let mut s = Self {
            serde: None,
            script: default_entity_script(),
            entity_data_id,
        };
        s.set_script();
        s
    }

    pub fn prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.script.bind_mut().prepare(bs_ptr, entity_idx);
    }

    pub fn start(&mut self) {
        if self.script_data().has_start {
            self.script.bind_mut().call("start".into(), &[]);
        }
    }

    pub fn destroyed(&mut self) {
        // if self.script_data().has_destroyed {
        //     self.script.bind_mut().call("destroyed".into(), &[]);
        // }
    }

    pub fn step(&mut self) {
        if self.script_data().has_step {
            self.script.bind_mut().call("step".into(), &[]);
        }
    }

    pub fn pre_serialize(&mut self) {
        if self.script_data().has_serialize {
            self.serde = Some(
                var_to_bytes(self.script.bind_mut().call("pre_serialize".into(), &[])).to_vec(),
            );
        }
    }

    /// Create and prepare the script.
    pub fn post_deserialize_prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.set_script();
        self.prepare(bs_ptr, entity_idx);
    }

    /// Deserialize the script custom data.
    /// Should have called `post_deserialize_prepare` on all script before this.
    pub fn post_deserialize_post_prepare(&mut self) {
        if let Some(bytes) = self.serde.take() {
            if self.script_data().has_deserialize {
                self.script.bind_mut().call(
                    "deserialize".into(),
                    &[bytes_to_var(PackedByteArray::from(bytes.as_slice()))],
                );
            }
        }
    }

    fn set_script(&mut self) {
        if !self.script_data().script.is_nil() {
            let gdscript = self.script_data().script.clone();
            self.script.bind_mut().set_script(gdscript);
        }
    }

    fn script_data(&self) -> &EntityDataScript {
        &self.entity_data_id.data().script
    }
}
unsafe impl Send for EntityScriptWrapper {}

#[derive(Debug)]
pub struct EntityDataScript {
    pub script: Variant,
    pub has_start: bool,
    pub has_step: bool,
    pub has_serialize: bool,
    pub has_deserialize: bool,
}
impl EntityDataScript {
    pub fn new(script: Variant) -> Self {
        if let Ok(gd_script) = script.try_to::<Gd<godot::engine::Script>>() {
            let base_type = gd_script.get_instance_base_type().to_string();
            if "EntityScript" != base_type.as_str() {
                log::warn!(
                    "Expected simulation script to extend 'EntityScript', got '{}' instead. Removing...",
                    base_type
                );
                return Default::default();
            }
        } else {
            log::warn!("Hull simulation script is not a script. Ignoring...");
            return Default::default();
        }

        let mut obj: Gd<EntityScript> = Gd::new_default();
        let mut bind = obj.bind_mut();
        bind.set_script(script.clone());

        let s = Self {
            script,
            has_start: bind.has_method("start".into()),
            has_step: bind.has_method("step".into()),
            has_serialize: bind.has_method("serialize".into()),
            has_deserialize: bind.has_method("deserialize".into()),
        };

        drop(bind);
        obj.free();

        log::debug!("{:#?}", &s);

        s
    }
}
impl Default for EntityDataScript {
    fn default() -> Self {
        Self {
            script: Variant::nil(),
            has_start: false,
            has_step: false,
            has_serialize: false,
            has_deserialize: false,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Object)]
struct EntityScript {
    bs: BsPtr,
    entity_idx: usize,
    #[base]
    base: Base<Object>,
}
impl EntityScript {
    fn prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.bs = bs_ptr;
        self.entity_idx = entity_idx;
    }

    fn id(&self) -> EntityId {
        *self.bs.entities.get_index(self.entity_idx).unwrap().0
    }

    fn entity(&self) -> &Entity {
        &self.bs.entities[self.entity_idx]
    }

    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.bs.entities[self.entity_idx]
    }

    fn body(&mut self) -> &mut RigidBody {
        let handle = self.entity().rb;
        &mut self.bs.physics.bodies[handle]
    }
}
#[godot_api]
impl EntityScript {
    // ---------- API

    /// Only intended for serialization.
    #[func]
    fn get_id(&self) -> i64 {
        self.id().0 as i64
    }

    /// Only intended for deserialization.
    #[func]
    fn get_entity_from_id(&mut self, id: i64) -> Gd<EntityScript> {
        self.bs
            .entities
            .get(&EntityId(id as u32))
            .map(|entity| entity.script.script.share())
            .expect("entity should exist")
    }

    // ---------- CALLBACKS
    
    #[func]
    fn cb_on_hull_destroyed(&mut self, hull_idx: i64, entity: Gd<EntityScript>, method: StringName, arg_array: Array) {
        entity.bind().get_id();
        StringName::from("asd");
        match arg_array.get(0).get_type() {
            VariantType::Nil => todo!(),
            VariantType::Bool => todo!(),
            VariantType::Int => todo!(),
            VariantType::Float => todo!(),
            VariantType::String => todo!(),
            VariantType::Vector2 => todo!(),
            VariantType::Vector2i => todo!(),
            VariantType::Rect2 => todo!(),
            VariantType::Rect2i => todo!(),
            VariantType::Vector3 => todo!(),
            VariantType::Vector3i => todo!(),
            VariantType::Transform2D => todo!(),
            VariantType::Vector4 => todo!(),
            VariantType::Vector4i => todo!(),
            VariantType::Plane => todo!(),
            VariantType::Quaternion => todo!(),
            VariantType::Aabb => todo!(),
            VariantType::Basis => todo!(),
            VariantType::Transform3D => todo!(),
            VariantType::Projection => todo!(),
            VariantType::Color => todo!(),
            VariantType::StringName => todo!(),
            VariantType::NodePath => todo!(),
            // VariantType::Rid => todo!(),
            // VariantType::Object => todo!(),
            // VariantType::Callable => todo!(),
            // VariantType::Signal => todo!(),
            VariantType::Dictionary => todo!(),
            VariantType::Array => todo!(),
            VariantType::PackedByteArray => todo!(),
            VariantType::PackedInt32Array => todo!(),
            VariantType::PackedInt64Array => todo!(),
            VariantType::PackedFloat32Array => todo!(),
            VariantType::PackedFloat64Array => todo!(),
            VariantType::PackedStringArray => todo!(),
            VariantType::PackedVector2Array => todo!(),
            VariantType::PackedVector3Array => todo!(),
            VariantType::PackedColorArray => todo!(),
            _ => todo!(),
        }
    }

    // ---------- SCRIPT

    #[func]
    fn get_position(&mut self) -> Vector2 {
        self.body().translation().to_godot()
    }

    #[func]
    fn get_rotation(&mut self) -> Vector2 {
        let r = self.body().rotation();
        Vector2::new(r.re, r.im)
    }

    #[func]
    fn get_angle(&mut self) -> f32 {
        self.body().rotation().angle()
    }

    // ---------- ANGVEL
    #[func]
    fn set_wish_angvel_keep(&mut self) {
        self.entity_mut().wish_angvel = WishAngVel::Keep;
    }

    #[func]
    fn set_wish_angvel_cancel(&mut self) {
        self.entity_mut().wish_angvel = WishAngVel::Cancel;
    }

    #[func]
    fn set_wish_angvel_aim_at(&mut self, position: Vector2) {
        self.entity_mut().wish_angvel = WishAngVel::Aim {
            position: position.to_na_descaled(),
        };
    }

    /// Call a function on the corresponding render node, if it exist (rendering may be disabled).
    #[func]
    fn add_render_call(&mut self, method: StringName, arg_array: Array) {
        // TODO: send event
    }
}
#[godot_api]
impl GodotExt for EntityScript {
    fn init(base: Base<Object>) -> Self {
        Self {
            bs: Default::default(),
            entity_idx: Default::default(),
            base,
        }
    }
}

// #[derive(GodotClass)]
// #[class(base=Object)]
// struct HullScript {
//     bs: BsPtr,
//     entity_idx: usize,
//     hull_idx: usize,
//     #[base]
//     base: Base<Object>,
// }
// impl HullScript {
//     fn prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize, hull_idx: usize) {
//         self.bs = bs_ptr;
//         self.entity_idx = entity_idx;
//         self.hull_idx = hull_idx;
//     }

//     fn entity(&mut self) -> &mut Entity {
//         &mut self.bs.entities[self.entity_idx]
//     }

//     fn hull(&mut self) -> Option<&mut Hull> {
//         let hull_idx = self.hull_idx;
//         self.entity().hulls[hull_idx].as_mut()
//     }

//     fn collider(&mut self) -> Option<&mut Collider> {
//         self.hull()
//             .map(|hull| hull.collider)
//             .map(|handle| &mut self.bs.physics.colliders[handle])
//     }
// }
// #[godot_api]
// impl HullScript {
//     // ---------- API

//     #[func]
//     fn get_id(&mut self) -> i64 {
//         let entity_id = self.bs.entities.get_index(self.entity_idx).unwrap().0 .0 as i64;
//         let hull_idx = self.hull_idx as i64;
//         entity_id + (hull_idx << 32)
//     }

//     #[func]
//     fn get_entity_from_id(&mut self, id: i64) -> Gd<EntityScript> {
//         self.bs
//             .entities
//             .get(&EntityId(id as u32))
//             .map(|entity| entity.script.script.share())
//             .expect("entity should exist")
//     }

//     #[func]
//     fn get_hull_from_id(&mut self, id: i64) -> Gd<HullScript> {
//         self.bs
//             .entities
//             .get(&EntityId(id as u32))
//             .and_then(|entity| entity.hulls[(id >> 32) as usize].as_ref())
//             .map(|hull| hull.script.script.share())
//             .expect("hull should exist")
//     }

//     // ---------- SCRIPT

//     #[func]
//     fn get_local_position(&mut self) -> Vector2 {
//         self.collider()
//             .and_then(|collider| collider.position_wrt_parent())
//             .map(|pos_wrt_parent| pos_wrt_parent.translation.to_godot())
//             .unwrap_or_default()
//     }

//     #[func]
//     fn get_global_position(&mut self) -> Vector2 {
//         self.collider()
//             .map(|collider| collider.translation().to_godot())
//             .unwrap_or_default()
//     }

//     #[func]
//     fn get_parent_entity(&mut self) -> Gd<EntityScript> {
//         self.entity().script.script.share()
//     }

//     // #[func]
//     // fn rotation(&mut self) -> Vector2 {
//     //     let r = self.body().rotation();
//     //     Vector2::new(r.re, r.im)
//     // }

//     // #[func]
//     // fn angle(&mut self) -> f32 {
//     //     self.body().rotation().angle()
//     // }

//     /// Call a function on the corresponding render node, if it exist (rendering may be disabled).
//     #[func]
//     fn add_render_call(&mut self, method: StringName, arg_array: Variant) {
//         // TODO: send event
//     }
// }
// #[godot_api]
// impl GodotExt for HullScript {
//     fn init(base: Base<Object>) -> Self {
//         Self {
//             bs: Default::default(),
//             entity_idx: Default::default(),
//             hull_idx: Default::default(),
//             base,
//         }
//     }
// }

#[derive(Clone, Copy)]
pub struct BsPtr(*mut Battlescape);
impl BsPtr {
    pub fn new(bs: &mut Battlescape) -> Self {
        Self(bs as *mut _)
    }
}
impl Default for BsPtr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}
impl Deref for BsPtr {
    type Target = Battlescape;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl DerefMut for BsPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

fn default_entity_script() -> Gd<EntityScript> {
    Gd::new_default()
}
