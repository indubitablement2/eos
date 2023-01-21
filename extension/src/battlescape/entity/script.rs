use super::*;
use crate::util::*;
use godot::prelude::{
    utilities::{bytes_to_var_with_objects, var_to_bytes_with_objects},
    *,
};
use serde::de::Visitor;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptWrapper {
    wrapper: GodotScriptWrapper,
    need_step: bool,
}
impl ScriptWrapper {
    pub fn new_empty_entity() -> Self {
        let script: Gd<EntityScript> = Gd::new_default();
        Self {
            wrapper: GodotScriptWrapper(script.upcast()),
            need_step: false,
        }
    }

    pub fn new_empty_hull() -> Self {
        let script: Gd<HullScript> = Gd::new_default();
        Self {
            wrapper: GodotScriptWrapper(script.upcast()),
            need_step: false,
        }
    }

    pub fn new_entity(mut script: Gd<Resource>) -> Self {
        if script.has_method("__is_entity_script".into()) {
            Self {
                need_step: script.call("_need_step".into(), &[]).to(),
                wrapper: GodotScriptWrapper(script),
            }
        } else {
            Self::new_empty_hull()
        }
    }

    pub fn new_hull(mut script: Gd<Resource>) -> Self {
        if script.has_method("__is_hull_script".into()) {
            Self {
                need_step: script.call("_need_step".into(), &[]).to(),
                wrapper: GodotScriptWrapper(script),
            }
        } else {
            Self::new_empty_hull()
        }
    }

    pub fn prepare_entity(&mut self, bc_ptr: Variant, entity_idx: Variant) {
        self.wrapper
            .0
            .call("__prepare_internal".into(), &[bc_ptr, entity_idx]);
    }

    pub fn prepare_hull(&mut self, bc_ptr: Variant, entity_idx: Variant, hull_idx: Variant) {
        self.wrapper
            .0
            .call("__prepare_internal".into(), &[bc_ptr, entity_idx, hull_idx]);
    }

    pub fn step(&mut self) {
        if self.need_step {
            self.wrapper.0.call("_step".into(), &[]);
        }
    }
}

#[derive(Debug)]
struct GodotScriptWrapper(Gd<Resource>);
impl Clone for GodotScriptWrapper {
    fn clone(&self) -> Self {
        // TODO: Need subresource?
        Self(self.0.duplicate(false).unwrap())
    }
}
unsafe impl Send for GodotScriptWrapper {}
impl Serialize for GodotScriptWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = var_to_bytes_with_objects(self.0.to_variant());
        // TODO: use those bytes when array are implemented.
        serializer.serialize_bytes(&[])
    }
}
impl<'de> Deserialize<'de> for GodotScriptWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Cast to array when implemented
        deserializer
            .deserialize_byte_buf(BufVisitor)
            .map(|buf| GodotScriptWrapper(bytes_to_var_with_objects(todo!()).to()))
    }
}
struct BufVisitor;
impl<'de> Visitor<'de> for BufVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "bytes")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_vec())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }
}

#[derive(GodotClass)]
#[class(base=Resource)]
struct EntityScript {
    bc: BcPtr,
    entity_idx: usize,
    #[base]
    base: Base<Resource>,
}
impl EntityScript {
    fn entity(&mut self) -> &mut Entity {
        &mut self.bc.entities[self.entity_idx]
    }

    fn body(&mut self) -> &mut RigidBody {
        let handle = self.entity().rb;
        &mut self.bc.physics.bodies[handle]
    }
}
#[godot_api]
impl EntityScript {
    #[func]
    fn __prepare_internal(&mut self, bc_ptr: i64, entity_idx: i64) {
        self.bc = BcPtr(bc_ptr as *mut _);
        self.entity_idx = entity_idx as usize;
    }

    #[func]
    fn __is_entity_script(&mut self) {}

    #[func]
    fn _need_step(&mut self) -> bool {
        false
    }

    /// Called by the engine each tick.
    #[func]
    fn _step(&mut self) {}

    #[func]
    fn position(&mut self) -> Vector2 {
        self.body().translation().to_godot()
    }

    #[func]
    fn rotation(&mut self) -> Vector2 {
        let r = self.body().rotation();
        Vector2::new(r.re, r.im)
    }

    #[func]
    fn angle(&mut self) -> f32 {
        self.body().rotation().angle()
    }

    /// Call a function on the corresponding render node, if it exist (rendering may be disabled).
    #[func]
    fn render_call(&mut self, method: StringName, arg_array: Variant) {
        // TODO: send event
    }
}
#[godot_api]
impl GodotExt for EntityScript {
    fn init(base: Base<Resource>) -> Self {
        Self {
            bc: Default::default(),
            entity_idx: Default::default(),
            base,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Resource)]
struct HullScript {
    bc: BcPtr,
    entity_idx: usize,
    hull_idx: usize,
    #[base]
    base: Base<Resource>,
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
    fn __prepare_internal(&mut self, bc_ptr: i64, entity_idx: i64, hull_idx: i64) {
        self.bc = BcPtr(bc_ptr as *mut _);
        self.entity_idx = entity_idx as usize;
        self.hull_idx = hull_idx as usize;
    }

    #[func]
    fn __is_hull_script(&mut self) {}

    #[func]
    fn _need_step(&mut self) -> bool {
        false
    }

    /// Called by the engine each tick.
    #[func]
    fn _step(&mut self) {}

    #[func]
    fn exist(&mut self) -> bool {
        self.hull().is_some()
    }

    #[func]
    fn position(&mut self) -> Vector2 {
        self.collider()
            .and_then(|collider| collider.position_wrt_parent())
            .map(|pos_wrt_parent| pos_wrt_parent.translation.to_godot())
            .unwrap_or_default()
    }

    #[func]
    fn global_position(&mut self) -> Vector2 {
        self.collider()
            .map(|collider| collider.translation().to_godot())
            .unwrap_or_default()
    }

    #[func]
    fn parent_entity(&mut self) -> Gd<EntityScript> {
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
    fn render_call(&mut self, method: StringName, arg_array: Variant) {
        // TODO: send event
    }
}
#[godot_api]
impl GodotExt for HullScript {
    fn init(base: Base<Resource>) -> Self {
        Self {
            bc: Default::default(),
            entity_idx: Default::default(),
            hull_idx: Default::default(),
            base,
        }
    }
}

struct BcPtr(*mut Battlescape);
impl Default for BcPtr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}
impl Deref for BcPtr {
    type Target = Battlescape;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl DerefMut for BcPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
