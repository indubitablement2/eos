use super::*;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

#[derive(Default)]
pub struct Character {
    pub removed: bool,
    pub data: CharacterDataId,

    health_relative: f32,

    level: u32,
    on_level_modifier: DynamicModifiers,

    pub position: Vec2,

    pub radius: Mulf32,

    max_health_flat: Addf32,
    max_health_increase: Addf32,
    max_health_multiplier: Mulf32,

    movement_speed_increase: Addf32,

    modifiers: Vec<ModifierOrigin>,
    ai: Ai,
}
impl Character {
    pub fn new(data: CharacterDataId, health_relative: f32, level: u32) -> CharacterRef {
        let mut char = Self {
            data,
            health_relative,
            level,
            ..Default::default()
        };
        char.reset();
        CharacterRef {
            inner: Rc::new(RefCell::new(char)),
        }
    }

    pub fn reset(&mut self) {
        self.on_level_modifier.clear();

        self.radius = self.data.radius.into();

        self.max_health_flat = (20.0 + self.level as f32 * 10.0).into();
        self.max_health_increase = 1.0.into();
        self.max_health_multiplier = self.data.max_health_multiplier.into();

        self.movement_speed_increase = 1.0.into();

        let mut modifiers = std::mem::take(&mut self.modifiers);
        for modifier in modifiers.iter_mut() {
            modifier.apply(self);
        }
        self.modifiers = modifiers;
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn set_level(&mut self, level: u32) {
        let on_level_modifier = self.on_level_modifier.take().remove(self);
        self.level = level;
        self.on_level_modifier = on_level_modifier.apply(self);
    }

    pub fn max_health(&self) -> f32 {
        self.max_health_flat.value()
            * self.max_health_increase.value()
            * self.max_health_multiplier.value()
    }

    pub fn movement_speed(&self) -> f32 {
        self.data.movement_speed * self.movement_speed_increase.value()
    }
}

#[derive(Debug, Default)]
enum Ai {
    #[default]
    None,
    Seek {
        target: Option<u32>,
    },
}

#[derive(Debug)]
enum ModifierOrigin {
    // TODO: dynamic item
    Item {},
}
impl ModifierOrigin {
    fn apply(&mut self, char: &mut Character) {
        // TODO: Emit events
        // self.on_level_modifier = self.on_level_modifier.take().apply(self);

        match self {
            ModifierOrigin::Item {} => {}
        }
    }

    fn remove(&mut self, char: &mut Character) {
        match self {
            ModifierOrigin::Item {} => {}
        }
    }
}

// ####################################################################################
// ################################### MODIFIER #######################################
// ####################################################################################

#[derive(Default)]
struct DynamicModifiers {
    modifiers: Vec<(DynamicModifier, f32)>,
}
impl DynamicModifiers {
    fn clear(&mut self) {
        self.modifiers.clear();
    }

    fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    fn apply(mut self, char: &mut Character) -> Self {
        for (modifier, value) in self.modifiers.iter_mut() {
            modifier.apply(*value, char);
        }
        self
    }

    fn remove(mut self, char: &mut Character) -> Self {
        for (modifier, value) in self.modifiers.iter_mut() {
            modifier.remove(*value, char);
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DynamicModifier {
    MovementSpeedPerLevel,
}
impl DynamicModifier {
    fn apply(self, value: f32, char: &mut Character) {
        match self {
            DynamicModifier::MovementSpeedPerLevel => {
                char.movement_speed_increase += value * char.level as f32
            }
        }
    }

    fn remove(self, value: f32, char: &mut Character) {
        match self {
            DynamicModifier::MovementSpeedPerLevel => {
                char.movement_speed_increase -= value * char.level as f32
            }
        }
    }
}

// ####################################################################################
// ################################### EVENTS #########################################
// ####################################################################################

impl Character {
    pub fn step(&mut self, sim: &mut Simulation) {
        match &mut self.ai {
            Ai::None => {}
            Ai::Seek { target } => {
                *target = None;
            }
        }

        // self.modifiers.retain_mut(|(_, modifier)| match modifier {
        //     Modifier::RemoveThis => false,
        // });

        if self.health_relative <= 0.0 {
            self.on_death();
            // retain = false;
        }
    }

    fn on_death(&mut self) {
        for &event in self.data.on_death.iter() {
            match event {}
        }
    }

    // fn on_hit(&mut self, mut dmg: f32, local_pos: Vec2) {

    //     self.hull_relative -= dmg / self.hull_max();
    // }
}

// ####################################################################################
// ################################### DATA ###########################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterData {
    id: u32,

    radius: f32,

    max_health_multiplier: f32,

    movement_speed: f32,

    on_death: Vec<CharacterDataEvent>,
}
impl CharacterData {
    pub fn load_data() {
        let read = std::fs::read("../client/tool/server_data/characters.json").unwrap();
        let data: Vec<CharacterData> = serde_json::from_slice(read.as_slice()).unwrap();
        DATA.set(data).ok().unwrap();
    }

    pub fn data() -> &'static [CharacterData] {
        DATA.get().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum CharacterDataEvent {}

static DATA: std::sync::OnceLock<Vec<CharacterData>> = std::sync::OnceLock::new();

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "u32")]
#[serde(into = "u32")]
pub struct CharacterDataId(pub &'static CharacterData);
impl Default for CharacterDataId {
    fn default() -> Self {
        Self(CharacterData::data().first().unwrap())
    }
}
impl std::ops::Deref for CharacterDataId {
    type Target = CharacterData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl TryFrom<u32> for CharacterDataId {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        CharacterData::data()
            .get(value as usize)
            .map(Self)
            .ok_or(value)
    }
}
impl From<CharacterDataId> for u32 {
    fn from(id: CharacterDataId) -> Self {
        id.id
    }
}
impl std::fmt::Debug for CharacterDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

// ####################################################################################
// ################################### OTHER ##########################################
// ####################################################################################

#[derive(Clone)]
pub struct CharacterRef {
    inner: Rc<RefCell<Character>>,
}
impl CharacterRef {
    pub fn get(&self) -> Option<Ref<Character>> {
        self.inner
            .try_borrow()
            .ok()
            .and_then(|r| if r.removed { None } else { Some(r) })
    }

    pub fn get_mut(&self) -> Option<RefMut<Character>> {
        self.inner
            .try_borrow_mut()
            .ok()
            .and_then(|r| if r.removed { None } else { Some(r) })
    }
}
impl std::hash::Hash for CharacterRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Rc::as_ptr(&self.inner), state);
    }
}
impl PartialEq for CharacterRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for CharacterRef {}

#[derive(Debug, Clone, Copy, Default)]
pub struct Mulf32 {
    value: f32,
}
impl Mulf32 {
    pub fn value(&self) -> f32 {
        self.value
    }
}
impl From<f32> for Mulf32 {
    fn from(value: f32) -> Self {
        Self { value }
    }
}
impl std::ops::Mul<f32> for Mulf32 {
    type Output = Mulf32;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            value: self.value * rhs,
        }
    }
}
impl std::ops::MulAssign<f32> for Mulf32 {
    fn mul_assign(&mut self, rhs: f32) {
        self.value *= rhs;
    }
}
impl std::ops::Div<f32> for Mulf32 {
    type Output = Mulf32;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            value: self.value / rhs,
        }
    }
}
impl std::ops::DivAssign<f32> for Mulf32 {
    fn div_assign(&mut self, rhs: f32) {
        self.value /= rhs;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Addf32 {
    value: f32,
}
impl Addf32 {
    pub fn value(&self) -> f32 {
        self.value
    }
}
impl From<f32> for Addf32 {
    fn from(value: f32) -> Self {
        Self { value }
    }
}
impl std::ops::Add<f32> for Addf32 {
    type Output = Addf32;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            value: self.value + rhs,
        }
    }
}
impl std::ops::AddAssign<f32> for Addf32 {
    fn add_assign(&mut self, rhs: f32) {
        self.value += rhs;
    }
}
impl std::ops::Sub<f32> for Addf32 {
    type Output = Addf32;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            value: self.value - rhs,
        }
    }
}
impl std::ops::SubAssign<f32> for Addf32 {
    fn sub_assign(&mut self, rhs: f32) {
        self.value -= rhs;
    }
}
