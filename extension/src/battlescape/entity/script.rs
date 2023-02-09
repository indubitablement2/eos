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
        let mut script = default_entity_script();
        script
            .bind_mut()
            .set_script(entity_data_id.data().script.clone());
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
        self.script.bind_mut().start();
    }

    pub fn destroyed(&mut self) {
        self.script.bind_mut().destroyed();
    }

    pub fn step(&mut self) {
        self.script.bind_mut().step();
    }

    pub fn pre_serialize(&mut self) {
        self.serde = Some(var_to_bytes(self.script.bind_mut().serialize()).to_vec());
    }

    /// Create and prepare the script.
    pub fn post_deserialize_prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        let serde = self.serde.take();
        let entity_data_id = self.entity_data_id;

        *self = Self::new(entity_data_id);
        self.serde = serde;

        self.prepare(bs_ptr, entity_idx);
    }

    /// Deserialize the script custom data.
    /// Should have called `post_deserialize_prepare` on all script before this.
    pub fn post_deserialize_post_prepare(&mut self) {
        if let Some(bytes) = self.serde.take() {
            self.script
                .bind_mut()
                .deserialize(bytes_to_var(PackedByteArray::from(bytes.as_slice())));
        }
    }
}
unsafe impl Send for EntityScriptWrapper {}

#[derive(Debug, Serialize, Deserialize)]
pub struct HullScriptWrapper {
    serde: Option<Vec<u8>>,
    #[serde(skip)]
    #[serde(default = "default_hull_script")]
    script: Gd<HullScript>,
    entity_data_id: EntityDataId,
}
impl HullScriptWrapper {
    pub fn new(entity_data_id: EntityDataId, hull_idx: usize) -> Self {
        let mut script = default_hull_script();
        script
            .bind_mut()
            .set_script(entity_data_id.data().hulls[hull_idx].script.clone());
        Self {
            serde: None,
            script,
            entity_data_id,
        }
    }

    pub fn prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize, hull_idx: usize) {
        self.script.bind_mut().prepare(bs_ptr, entity_idx, hull_idx);
    }

    pub fn start(&mut self) {
        self.script.bind_mut().start();
    }

    pub fn destroyed(&mut self) {
        self.script.bind_mut().destroyed();
    }

    pub fn step(&mut self) {
        self.script.bind_mut().step();
    }

    pub fn pre_serialize(&mut self) {
        self.serde = Some(var_to_bytes(self.script.bind_mut().serialize()).to_vec());
    }

    /// Create and prepare the script.
    pub fn post_deserialize_prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize, hull_idx: usize) {
        let serde = self.serde.take();
        let entity_data_id = self.entity_data_id;

        *self = Self::new(entity_data_id, hull_idx);
        self.serde = serde;

        self.prepare(bs_ptr, entity_idx, hull_idx);
    }

    /// Deserialize the script custom data.
    /// Should have called `post_deserialize_prepare` on all script before this.
    pub fn post_deserialize_post_prepare(&mut self) {
        if let Some(bytes) = self.serde.take() {
            self.script
                .bind_mut()
                .deserialize(bytes_to_var(PackedByteArray::from(bytes.as_slice())));
        }
    }
}
unsafe impl Send for HullScriptWrapper {}

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

    fn start(&mut self) {
        self.call("i_start".into(), &[]);
    }

    fn destroyed(&mut self) {
        self.emit_signal("destroyed".into(), &[]);
    }

    fn step(&mut self) {
        self.call("i_step".into(), &[]);
    }

    fn serialize(&mut self) -> Variant {
        self.call("i_serialize".into(), &[])
    }

    fn deserialize(&mut self, var: Variant) {
        self.call("i_deserialize".into(), &[var]);
    }
}
#[godot_api]
impl EntityScript {
    #[signal]
    fn destroyed();

    // ---------- INTERFACE

    #[func]
    fn i_start(&mut self) {}

    #[func]
    fn i_step(&mut self) {}

    #[func]
    fn i_serialize(&mut self) -> Variant {
        Variant::nil()
    }

    #[func]
    fn i_deserialize(&mut self, _var: Variant) {}

    // ---------- API

    /// Only intended for serialization.
    #[func]
    fn get_id(&mut self) -> i64 {
        self.bs.entities.get_index(self.entity_idx).unwrap().0 .0 as i64
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

    /// Only intended for deserialization.
    #[func]
    fn get_hull_from_id(&mut self, id: i64) -> Gd<HullScript> {
        self.bs
            .entities
            .get(&EntityId(id as u32))
            .and_then(|entity| entity.hulls[(id >> 32) as usize].as_ref())
            .map(|hull| hull.script.script.share())
            .expect("hull should exist")
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
            bs: Default::default(),
            entity_idx: Default::default(),
            base,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Object)]
struct HullScript {
    bs: BsPtr,
    entity_idx: usize,
    hull_idx: usize,
    #[base]
    base: Base<Object>,
}
impl HullScript {
    fn prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize, hull_idx: usize) {
        self.bs = bs_ptr;
        self.entity_idx = entity_idx;
        self.hull_idx = hull_idx;
    }

    fn entity(&mut self) -> &mut Entity {
        &mut self.bs.entities[self.entity_idx]
    }

    fn hull(&mut self) -> Option<&mut Hull> {
        let hull_idx = self.hull_idx;
        self.entity().hulls[hull_idx].as_mut()
    }

    fn collider(&mut self) -> Option<&mut Collider> {
        self.hull()
            .map(|hull| hull.collider)
            .map(|handle| &mut self.bs.physics.colliders[handle])
    }

    fn start(&mut self) {
        self.call("i_start".into(), &[]);
    }

    fn destroyed(&mut self) {
        self.emit_signal("destroyed".into(), &[]);
    }

    fn step(&mut self) {
        self.call("i_step".into(), &[]);
    }

    fn serialize(&mut self) -> Variant {
        self.call("i_serialize".into(), &[])
    }

    fn deserialize(&mut self, var: Variant) {
        self.call("i_deserialize".into(), &[var]);
    }
}
#[godot_api]
impl HullScript {
    #[signal]
    fn destroyed();

    // ---------- INTERFACE

    #[func]
    fn i_start(&mut self) {}

    #[func]
    fn i_step(&mut self) {}

    #[func]
    fn i_serialize(&mut self) -> Variant {
        Variant::nil()
    }

    #[func]
    fn i_deserialize(&mut self, _var: Variant) {}

    // ---------- API

    #[func]
    fn get_id(&mut self) -> i64 {
        let entity_id = self.bs.entities.get_index(self.entity_idx).unwrap().0 .0 as i64;
        let hull_idx = self.hull_idx as i64;
        entity_id + (hull_idx << 32)
    }

    #[func]
    fn get_entity_from_id(&mut self, id: i64) -> Gd<EntityScript> {
        self.bs
            .entities
            .get(&EntityId(id as u32))
            .map(|entity| entity.script.script.share())
            .expect("entity should exist")
    }

    #[func]
    fn get_hull_from_id(&mut self, id: i64) -> Gd<HullScript> {
        self.bs
            .entities
            .get(&EntityId(id as u32))
            .and_then(|entity| entity.hulls[(id >> 32) as usize].as_ref())
            .map(|hull| hull.script.script.share())
            .expect("hull should exist")
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
        self.entity().script.script.share()
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
            bs: Default::default(),
            entity_idx: Default::default(),
            hull_idx: Default::default(),
            base,
        }
    }
}

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

fn default_hull_script() -> Gd<HullScript> {
    Gd::new_default()
}
