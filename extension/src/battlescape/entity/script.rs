use super::*;
use crate::util::*;
use godot::prelude::{
    utilities::{bytes_to_var, var_to_bytes},
    *,
};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityScriptWrapper {
    serde: Option<()>,
    #[serde(skip)]
    #[serde(default = "default_entity_script")]
    script: Gd<EntityScript>,
    entity_data_id: EntityDataId,
}
impl EntityScriptWrapper {
    pub fn new(entity_data_id: EntityDataId) -> Self {
        let mut script = default_entity_script();
        script.bind_mut().set_script(entity_data_id.data().script.clone());
        Self {
            serde: None,
            script,
            entity_data_id,
        }
    }

    pub fn prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.script.bind_mut().prepare(bs_ptr, entity_idx);
    }

    pub fn start(&mut self) {
        self.wrapper.start();
    }

    pub fn step(&mut self) {
        self.wrapper.step();
    }

    pub fn pre_serialize(&mut self) {
        // TODO: need array
        // self.serde = Some(self.wrapper.serialize());
    }

    /// Create and prepare the script.
    pub fn post_deserialize_prepare(&mut self, bc_ptr: Variant, entity_idx: Variant) {
        let serde = self.serde;
        let entity_data_id = self.entity_data_id;

        *self = Self::new(entity_data_id);
        self.serde = serde;

        self.prepare(bc_ptr, entity_idx);
    }

    /// Deserialize the script custom data.
    /// Should have called `post_deserialize_prepare` on all script before this.
    pub fn post_deserialize_post_prepare(&mut self) {
        if let Some(bytes) = self.serde.take() {
            // TODO: need array
            // self.wrapper.deserialize(bytes);
        }
    }
}
unsafe impl Send for EntityScriptWrapper {}

#[derive(Debug, Serialize, Deserialize)]
pub struct HullScriptWrapper {
    serde: Option<()>,
    #[serde(skip)]
    #[serde(default = "default_hull_script")]
    script: Gd<HullScript>,
    entity_data_id: EntityDataId,
    hull_idx: u32,
}
impl HullScriptWrapper {
    pub fn new(entity_data_id: EntityDataId, hull_idx: u32) -> Self {
        let mut script = default_hull_script();
        script.bind_mut().set_script(entity_data_id.data().hulls[hull_idx as usize].script.clone());
        Self {
            serde: None,
            script,
            entity_data_id,
            hull_idx,
        }
    }

    pub fn prepare(&mut self, bc_ptr: Variant, entity_idx: Variant, hull_idx: Variant) {
        self.wrapper.prepare(&[bc_ptr, entity_idx, hull_idx]);
    }

    pub fn start(&mut self) {
        self.wrapper.start();
    }

    pub fn step(&mut self) {
        self.wrapper.step();
    }

    pub fn pre_serialize(&mut self) {
        // TODO: need array
        // self.serde = Some(self.wrapper.serialize());
    }

    /// Create and prepare the script.
    pub fn post_deserialize_prepare(
        &mut self,
        bc_ptr: Variant,
        entity_idx: Variant,
        hull_idx: u32,
    ) {
        let serde = self.serde;
        let entity_data_id = self.entity_data_id;

        *self = Self::new(entity_data_id, hull_idx);
        self.serde = serde;

        self.prepare(bc_ptr, entity_idx, hull_idx.to_variant());
    }

    /// Deserialize the script custom data.
    /// Should have called `post_deserialize_prepare` on all script before this.
    pub fn post_deserialize_post_prepare(&mut self) {
        if let Some(bytes) = self.serde.take() {
            // TODO: need array
            // self.wrapper.deserialize(bytes);
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

    fn entity(&mut self) -> &mut Entity {
        &mut self.bs.entities[self.entity_idx]
    }

    fn body(&mut self) -> &mut RigidBody {
        let handle = self.entity().rb;
        &mut self.bs.physics.bodies[handle]
    }
}
#[godot_api]
impl EntityScript {
    #[func]
    fn start(&mut self) {}

    #[func]
    fn step(&mut self) {}

    #[func]
    fn serialize(&mut self) -> Variant {
        Variant::nil()
    }

    #[func]
    fn deserialize(&mut self, _var: Variant) {}

    // ---------- API

    #[func]
    fn get_id(&mut self) -> i64 {
        self.bs.entities.get_index(self.entity_idx).unwrap().0 .0 as i64
    }

    #[func]
    fn get_entity_from_id(&mut self, id: i64) -> Gd<EntityScript> {
        self.bs.entities
            .get(&EntityId(id as u32))
            .map(|entity| entity.script.wrapper.0.share().cast())
            .unwrap_or_else(|| {
                log::warn!("Tried to get entity from id {}, but it does not exist. Returning null instance...", id);
                let new: Gd<EntityScript> = Gd::new_default();
                new.share().free();
                new
            })
    }

    #[func]
    fn get_hull_from_id(&mut self, id: i64) -> Gd<HullScript> {
        self.bc.entities
            .get(&EntityId(id as u32))
            .and_then(|entity| entity.hulls[(id >> 32) as usize].as_ref())
            .map(|hull| hull.script.wrapper.0.share().cast())
            .unwrap_or_else(|| {
                log::warn!("Tried to get hull from id {}, but it does not exist. Returning null instance...", id);
                let new: Gd<HullScript> = Gd::new_default();
                new.share().free();
                new
            })
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
        self.entity().wish_angvel = WishAngVel::Keep;
    }

    #[func]
    fn set_wish_angvel_cancel(&mut self) {
        self.entity().wish_angvel = WishAngVel::Cancel;
    }

    #[func]
    fn set_wish_angvel_aim_at(&mut self, position: Vector2) {
        self.entity().wish_angvel = WishAngVel::Aim {
            position: position.to_na_descaled(),
        };
    }

    /// Call a function on the corresponding render node, if it exist (rendering may be disabled).
    #[func]
    fn add_render_call(&mut self, method: StringName, arg_array: Variant) {
        // TODO: send event
    }
}
#[godot_api]
impl GodotExt for EntityScript {
    fn init(base: Base<Object>) -> Self {
        Self {
            bc: Default::default(),
            entity_idx: Default::default(),
            base,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Object)]
struct HullScript {
    bc: BcPtr,
    entity_idx: usize,
    hull_idx: usize,
    #[base]
    base: Base<Object>,
}
impl HullScript {
    fn entity(&mut self) -> &mut Entity {
        &mut self.bc.entities[self.entity_idx]
    }

    fn hull(&mut self) -> Option<&mut Hull> {
        let hull_idx = self.hull_idx;
        self.entity().hulls[hull_idx].as_mut()
    }

    fn collider(&mut self) -> Option<&mut Collider> {
        self.hull()
            .map(|hull| hull.collider)
            .map(|handle| &mut self.bc.physics.colliders[handle])
    }
}
#[godot_api]
impl HullScript {
    #[func]
    fn _prepare(&mut self, bc_ptr: i64, entity_idx: i64, hull_idx: i64) {
        self.bc = BcPtr(bc_ptr as *mut _);
        self.entity_idx = entity_idx as usize;
        self.hull_idx = hull_idx as usize;
    }

    // ---------- VIRTUAL

    #[func]
    fn start(&mut self) {}

    #[func]
    fn step(&mut self) {}

    #[func]
    fn serialize(&mut self) -> Variant {
        Variant::nil()
    }

    #[func]
    fn deserialize(&mut self, _var: Variant) {}

    // ---------- API

    #[func]
    fn get_id(&mut self) -> i64 {
        let entity_id = self.bc.entities.get_index(self.entity_idx).unwrap().0 .0 as i64;
        let hull_idx = self.hull_idx as i64;
        entity_id + (hull_idx << 32)
    }

    #[func]
    fn get_entity_from_id(&mut self, id: i64) -> Gd<EntityScript> {
        self.bc.entities
            .get(&EntityId(id as u32))
            .map(|entity| entity.script.wrapper.0.share().cast())
            .unwrap_or_else(|| {
                log::warn!("Tried to get entity from id {}, but it does not exist. Returning null instance...", id);
                let new: Gd<EntityScript> = Gd::new_default();
                new.share().free();
                new
            })
    }

    #[func]
    fn get_hull_from_id(&mut self, id: i64) -> Gd<HullScript> {
        self.bc.entities
            .get(&EntityId(id as u32))
            .and_then(|entity| entity.hulls[(id >> 32) as usize].as_ref())
            .map(|hull| hull.script.wrapper.0.share().cast())
            .unwrap_or_else(|| {
                log::warn!("Tried to get hull from id {}, but it does not exist. Returning null instance...", id);
                let new: Gd<HullScript> = Gd::new_default();
                new.share().free();
                new
            })
    }

    // ---------- SCRIPT

    #[func]
    fn get_local_position(&mut self) -> Vector2 {
        self.collider()
            .and_then(|collider| collider.position_wrt_parent())
            .map(|pos_wrt_parent| pos_wrt_parent.translation.to_godot())
            .unwrap_or_default()
    }

    #[func]
    fn get_global_position(&mut self) -> Vector2 {
        self.collider()
            .map(|collider| collider.translation().to_godot())
            .unwrap_or_default()
    }

    #[func]
    fn get_parent_entity(&mut self) -> Gd<EntityScript> {
        self.entity().script.wrapper.0.share().cast()
    }

    // #[func]
    // fn rotation(&mut self) -> Vector2 {
    //     let r = self.body().rotation();
    //     Vector2::new(r.re, r.im)
    // }

    // #[func]
    // fn angle(&mut self) -> f32 {
    //     self.body().rotation().angle()
    // }

    /// Call a function on the corresponding render node, if it exist (rendering may be disabled).
    #[func]
    fn add_render_call(&mut self, method: StringName, arg_array: Variant) {
        // TODO: send event
    }
}
#[godot_api]
impl GodotExt for HullScript {
    fn init(base: Base<Object>) -> Self {
        Self {
            bc: Default::default(),
            entity_idx: Default::default(),
            hull_idx: Default::default(),
            base,
        }
    }
}

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

fn default_hull_script() -> Gd<HullScript> {
    Gd::new_default()
}