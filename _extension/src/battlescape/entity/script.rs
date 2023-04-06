use super::*;
use crate::util::*;
use godot::{
    engine::Engine,
    prelude::{
        utilities::{bytes_to_var, var_to_bytes},
        *,
    },
};
use std::ops::{Deref, DerefMut};

// prepare -> cmds -> script_step -> entity_step -> despawn

#[derive(Debug)]
pub struct EntityScriptData {
    script: Variant,
    has_script: bool,
    has_start: bool,
    has_step: bool,
    has_serialize: bool,
    has_deserialize: bool,
}
impl EntityScriptData {
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
            has_script: true,
            has_start: bind.has_method("start".into()),
            has_step: bind.has_method("step".into()),
            has_serialize: bind.has_method("serialize".into()),
            has_deserialize: bind.has_method("deserialize".into()),
        };

        drop(bind);
        obj.free();

        s
    }
}
impl Default for EntityScriptData {
    fn default() -> Self {
        Self {
            script: Variant::nil(),
            has_script: false,
            has_start: false,
            has_step: false,
            has_serialize: false,
            has_deserialize: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
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

    pub fn start(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.prepare(bs_ptr, entity_idx);

        if self.script_data().has_start {
            hack().start(self.script.to_variant());
        }
    }

    pub fn step(&mut self) {
        if self.script_data().has_step {
            hack().step(self.script.to_variant());
        }
    }

    pub fn prepare_serialize(&mut self) {
        if self.script_data().has_serialize {
            self.serde = Some(var_to_bytes(hack().serialize(self.script.to_variant())).to_vec());
        }
    }

    pub fn post_deserialize(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.set_script();
        self.prepare(bs_ptr, entity_idx);
    }

    pub fn post_post_deserialize(&mut self) {
        if self.script_data().has_deserialize {
            if let Some(serde) = self.serde.take() {
                hack().deserialize(
                    self.script.to_variant(),
                    bytes_to_var(PackedByteArray::from(serde.as_slice())),
                )
            }
        }
    }

    fn set_script(&mut self) {
        if self.script_data().has_script {
            let gdscript = self.script_data().script.clone();
            self.script.bind_mut().set_script(gdscript);
        }
    }

    fn script_data(&self) -> &EntityScriptData {
        &self.entity_data_id.data().script
    }
}
impl Drop for EntityScriptWrapper {
    fn drop(&mut self) {
        self.script.share().free();
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Callbacks {
    next_id: i64,
    callbacks: Vec<Callback>,
}
impl Callbacks {
    pub fn emit(&mut self) {
        let mut i = 0usize;
        while i < self.callbacks.len() {
            if self.callbacks[i].emit() {
                i += 1;
            } else {
                self.callbacks.swap_remove(i);
            }
        }
    }

    fn push(&mut self, method_args: Option<Variant>, callable: Variant) -> i64 {
        let id = self.next_id;
        self.next_id += 1;

        self.callbacks.push(Callback {
            method_args,
            callable,
            id,
        });

        id
    }

    fn remove(&mut self, callback_id: i64) {
        let mut i = 0usize;
        while i < self.callbacks.len() {
            if self.callbacks[i].id == callback_id {
                self.callbacks.swap_remove(i);
                return;
            }
            i += 1;
        }
    }
}

// TODO: Serialize/Deserialize callbacks
#[derive(Serialize, Deserialize)]
struct Callback {
    #[serde(skip)]
    method_args: Option<Variant>,
    #[serde(skip)]
    callable: Variant,
    id: i64,
}
impl Callback {
    /// Return if the callback could be sent.
    fn emit(&mut self) -> bool {
        if self.is_valid() {
            hack().callback(self);
            true
        } else {
            false
        }
    }

    fn is_valid(&self) -> bool {
        // TODO: Check if callable is valid.
        true
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
    // ---------- CALLBACKS

    /// Return nothing when destroyed.
    #[func]
    fn cb_on_destroyed(&mut self, callable: Variant) -> i64 {
        self.entity_mut().cb_destroyed.push(None, callable)
    }

    #[func]
    fn rcb_on_destroyed(&mut self, callback_id: i64) {
        self.entity_mut().cb_destroyed.remove(callback_id);
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

struct Hack {
    node: Gd<Object>,
    step: StringName,
    start: StringName,
    serialize: StringName,
    deserialize: StringName,
    callback: StringName,
    callback_empty: StringName,
}
impl Hack {
    fn new() -> Self {
        Self {
            node: Engine::singleton().get_singleton("Hack".into()).unwrap(),
            start: "start".into(),
            step: "step".into(),
            serialize: "serialize".into(),
            deserialize: "deserialize".into(),
            callback: "callback".into(),
            callback_empty: "callback_empty".into(),
        }
    }

    fn start(&mut self, on: Variant) {
        self.node.call(self.start.clone(), &[on]);
    }

    fn step(&mut self, on: Variant) {
        self.node.call(self.step.clone(), &[on]);
    }

    fn serialize(&mut self, on: Variant) -> Variant {
        self.node.call(self.serialize.clone(), &[on])
    }

    fn deserialize(&mut self, on: Variant, data: Variant) {
        self.node.call(self.deserialize.clone(), &[on, data]);
    }

    fn callback(&mut self, callback: &Callback) {
        if let Some(method_args) = &callback.method_args {
            self.node.call(
                self.callback.clone(),
                &[method_args.clone(), callback.callable.clone()],
            );
        } else {
            self.node
                .call(self.callback_empty.clone(), &[callback.callable.clone()]);
        }
    }
}
fn hack() -> &'static mut Hack {
    unsafe { HACK.get_or_insert_with(|| Hack::new()) }
}
static mut HACK: Option<Hack> = None;
