pub mod data_json;
pub mod physics;
pub mod update;

use super::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct HullId(pub std::num::NonZeroU64);
impl Default for HullId {
    fn default() -> Self {
        Self(std::num::NonZeroU64::MIN)
    }
}
impl Id for HullId {
    fn next(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn try_from_u64(value: u64) -> Option<Self> {
        std::num::NonZeroU64::new(value).map(Self)
    }
}

/// A ship, drone, missile or debris.
/// May only have one shield.
#[derive(Default)]
pub struct Hull {
    pub hull_data_id: HullDataId,

    pub owner: Option<ClientId>,

    /// Best not to touch this.
    /// See `pos`/`linvel`/`angvel`/`collision_group_ignore`.
    /// These properties are kept in sync with physics.
    // TODO: Shield rotation and arc
    rb: RigidBodyHandle,
    pub position: Vector2<f32>,
    pub rotation: UnitComplex<f32>,
    pub linvel: Vector2<f32>,
    pub angvel: f32,
    /// Ignore collision with anything in the same group.
    pub collision_group_ignore: u64,

    hull_max_percent_increase: i32,
    hull_max_flat_increase: i32,
    /// Relative to hull max. Usually in the range `0..1`.
    hull_relative: f32,

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

    modifiers: SmallVec<[Modifier; 2]>,
}
impl Hull {
    pub fn hull(&self) -> f32 {
        self.hull_relative
            * (self.hull_data_id.hull_max + self.hull_max_flat_increase as f32)
            * (self.hull_max_percent_increase + 100) as f32
            / 100.0
    }
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

#[derive(Debug)]
enum Modifier {
    RemoveThis,
}

// ####################################################################################
// ################################### EVENTS #########################################
// ####################################################################################

impl Hull {
    fn on_new(&mut self) {
        for &event in self.hull_data_id.0.on_new.iter() {
            match event {}
        }

        match self.hull_data_id.0.ai {
            HullAi::None => {}
            HullAi::Ship => {}
            HullAi::Seek => {
                self.wish_linvel = WishLinVel::ForceRelative(vector![0.0, 1.0]);
            }
        }
    }

    fn on_remove(&mut self, reason: RemoveReason) {
        for &event in self.hull_data_id.0.on_remove.iter() {
            match event {}
        }
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
    id: u32,

    hull_max: f32,

    armor_max: f32,
    armor_cells_translation: Vector2<f32>,
    armor_cells_size: Vector2<i32>,
    /// The maximum value a cell can have.
    armor_cells: (),

    shape_translation: Vector2<f32>,
    shape: SharedShape,
    groups: InteractionGroups,
    mprops: MassProperties,

    linear_acceleration: f32,
    angular_acceleration: f32,
    max_linear_velocity: f32,
    max_angular_velocity: f32,

    ai: HullAi,

    on_new: Vec<HullEvent>,
    on_remove: Vec<HullEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum HullEvent {}

#[derive(Debug, Default, Serialize, Deserialize)]
enum HullAi {
    #[default]
    None,
    Ship,
    /// Will try to face hull's target and go forward at max speed.
    /// If hull has no target just move forward untill a target is set.
    Seek,
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
    pub hull_data_id: HullDataId,

    pub owner: Option<ClientId>,

    pub position: Vector2<f32>,
    pub rotation: f32,
    pub linvel: Vector2<f32>,
    pub angvel: f32,

    pub hull_relative: f32,
    pub armor_cells: (),

    pub modifiers: SmallVec<[ModifierSave; 4]>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum ModifierSave {
    // TODO: This handle bad enum when deserializing?
    #[default]
    RemoveThis,
}
impl ModifierSave {
    fn apply(self, hull: &mut Hull) {
        match self {
            ModifierSave::RemoveThis => {}
        }
    }
}

impl Hull {
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
