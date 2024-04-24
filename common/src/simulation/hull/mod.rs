pub mod data_json;
pub mod update;

use super::*;
use nalgebra::UnitComplex;
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy)]
pub struct HullId {
    pub generation: NonZeroU32,
    pub index: u32,
}

/// A ship, drone, missile or debris.
/// May only have one shield.
pub struct Hull {
    pub data: HullDataId,
    pub owner: Option<ClientId>,

    pub tracking_clients: AHashSet<ClientId>,

    /// See `pos`/`linvel`/`angvel`/`collision_group_ignore`.
    /// These properties are kept in sync with physics.
    pub rb: RigidBodyHandle,
    pub position: Vector2<f32>,
    pub rotation: UnitComplex<f32>,
    pub linvel: Vector2<f32>,
    pub angvel: f32,
    /// Ignore collision with hulls in the same group.
    pub collision_group_ignore: u64,

    hull_max: f32,
    hull: f32,

    armor_max: f32,
    armor_cells: (),

    linacc: f32,
    angacc: f32,
    linvel_max: f32,
    angvel_max: f32,

    pub wish_angvel: WishAngVel,
    pub wish_linvel: WishLinVel,

    // pub wish_aim: (),
    pub controlled: bool,

    pub target: Option<HullId>,

    modifiers: SmallVec<[Modifier; 4]>,
}

#[derive(Debug, Clone, Copy)]
pub enum RemoveReason {
    Destroyed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishAngVel {
    #[default]
    None,
    /// Keep current angvel unless above max.
    Keep,
    Stop,
    /// Set angvel to face world space position without overshot.
    AimSmooth(Vector2<f32>),
    /// Turn left or right.
    Force(f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishLinVel {
    #[default]
    None,
    /// Keep current linvel unless above max.
    Keep,
    Stop,
    PositionSmooth(Vector2<f32>),
    PositionOvershoot(Vector2<f32>),
    /// A force in world space. -y is up.
    ForceAbsolute(Vector2<f32>),
    /// A force in local space. +y is forward, +x is right.
    ForceRelative(Vector2<f32>),
}

/// Something that modify the hull (ai, buff, etc).
#[derive(Debug, Default)]
enum Modifier {
    /// Does nothing. Modifier will be removed.
    /// Use this to remove a modifier.
    #[default]
    Nothing,
    AiShip,
    /// Will try to face hull's target and go forward at max speed.
    /// If hull has no target just move forward untill a target is set.
    AiSeek,
}

// ####################################################################################
// ################################### EVENTS #########################################
// ####################################################################################

impl Hull {
    pub fn on_remove(&mut self, reason: RemoveReason) {
        // TODO: Tracking clients notification.
    }
}

// ####################################################################################
// ################################### DATA ###########################################
// ####################################################################################

// TODO: weapon slot
// TODO: built-in weapon (take a slot #)
// TODO: Engine placement
// TODO: Shields
pub struct HullData {
    pub id: u32,

    hull_max: f32,

    armor_max: f32,
    armor_cells_translation: Vector2<f32>,
    armor_cells_size: Vector2<i32>,
    /// The maximum value a cell can have.
    armor_cells: (),

    pub shape_translation: Vector2<f32>,
    pub shape: SharedShape,
    pub mprops: MassProperties,
    pub groups: InteractionGroups,

    linear_acceleration: f32,
    angular_acceleration: f32,
    max_linear_velocity: f32,
    max_angular_velocity: f32,

    on_new: Vec<HullEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
enum HullEvent {
    AddAiShip,
    AddAiSeek,
}

static DATA: std::sync::OnceLock<Vec<HullData>> = std::sync::OnceLock::new();

fn data() -> &'static [HullData] {
    DATA.get().unwrap()
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "u32")]
#[serde(into = "u32")]
pub struct HullDataId(pub &'static HullData);
impl Default for HullDataId {
    fn default() -> Self {
        Self(data().first().unwrap())
    }
}
impl std::ops::Deref for HullDataId {
    type Target = HullData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl TryFrom<u32> for HullDataId {
    type Error = TryFromHullDataIdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        data()
            .get(value as usize)
            .map(Self)
            .ok_or(TryFromHullDataIdError(value))
    }
}
impl From<HullDataId> for u32 {
    fn from(idx: HullDataId) -> Self {
        idx.id
    }
}
impl std::fmt::Debug for HullDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

pub struct TryFromHullDataIdError(pub u32);
impl std::fmt::Display for TryFromHullDataIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid hull data id: {} out of bound", self.0)
    }
}

// ####################################################################################
// ################################### SAVE ###########################################
// ####################################################################################

// TODO: Inventory
// TODO: Turret
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct HullSave {
    data: u64,

    owner: Option<ClientId>,

    position: Isometry2<f32>,
    linvel: Vector2<f32>,
    angvel: f32,

    hull: f32,
    armor_cells: (),

    modifiers: SmallVec<[ModifierSave; 4]>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
enum ModifierSave {
    // TODO: This handle bad enum when deserializing?
    #[default]
    RemoveThis,
}

impl Hull {
    pub fn new(save: HullSave, group_ignore: u64, target: Option<HullId>) -> Self {
        todo!()
        // let rb = physics.add_body(
        //     save.position,
        //     save.linvel,
        //     save.angvel,
        //     save.data,
        //     entity_id,
        //     group_ignore,
        // );

        // let mut s = Self {
        //     data: save.data,
        //     owner: save.owner,
        //     tracking_clients: AHashSet::new(),
        //     rb,
        //     hull_max: save.data.hull_max,
        //     hull: save.hull,
        //     armor_max: save.data.armor_max,
        //     armor_cells: save.armor_cells,
        //     linear_acceleration: save.data.linear_acceleration,
        //     angular_acceleration: save.data.angular_acceleration,
        //     max_linear_velocity: save.data.max_linear_velocity,
        //     max_angular_velocity: save.data.max_angular_velocity,
        //     wish_angvel: WishAngVel::None,
        //     wish_linvel: WishLinVel::None,
        //     controlled: false,
        //     target,
        //     modifiers: SmallVec::new(),
        //     pos: todo!(),
        //     linvel: todo!(),
        //     angvel: todo!(),
        // };

        // for new_event in save.data.on_new.iter() {
        //     match new_event {
        //         EntityEvent::AddAiShip => {
        //             s.modifiers.push(Modifier::AiShip);
        //         }
        //         EntityEvent::AddAiSeek => {
        //             s.wish_linvel = WishLinVel::ForceRelative(Vector2::new(1.0, 0.0));
        //             s.modifiers.push(Modifier::AiSeek);
        //         }
        //     }
        // }

        // s
    }

    pub fn save(&self) -> HullSave {
        todo!()
        // let modifier_saves = self
        //     .modifiers
        //     .iter()
        //     .filter_map(|m| match m {
        //         Modifier::Nothing => None,
        //         Modifier::AiShip => None,
        //         Modifier::AiSeek => None,
        //     })
        //     .collect();

        // EntitySave {
        //     data: self.data.id,
        //     owner: self.owner,
        //     position: self.position(),
        //     linvel: self.linvel(),
        //     angvel: self.angvel(),
        //     hull: self.hull,
        //     armor_cells: self.armor_cells.clone(),
        //     modifiers: modifier_saves,
        // }
    }
}
