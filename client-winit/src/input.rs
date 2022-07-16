use glam::Vec2;
use winit::{
    dpi::PhysicalPosition,
    event::{MouseButton, VirtualKeyCode},
};

pub enum InputEvent {
    KeyboardPressed(VirtualKeyCode),
    KeyboardReleased(VirtualKeyCode),
    Character(char),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    /// Cursor position relative to the top-left corner of the window in pixel.
    CursorMoved(PhysicalPosition<f64>),
    /// Indicate movement forward (away from the user).
    MouseWheelUp,
    /// Indicate movement backward (toward the user).
    MouseWheelDown,
}

#[derive(Debug, Clone, Default)]
pub struct Inputs {
    /// Mouse position relative to the top left windows corner.
    /// Range from 0 to 1.
    pub mouse_pos: Vec2,
    /// Used for gamepad. Emulated with left/right/up/down.
    /// Clamped to a unit vertor.
    pub dir_strenght: Vec2,
    /// Used for gamepad only.
    /// Clamped to a unit vertor.
    pub look_strenght: Vec2,

    pub pressed: InputsAction,
    pub just_pressed: InputsAction,
}

#[derive(Debug, Clone, Default)]
pub struct InputsAction {
    pub primary: bool,
    pub secondary: bool,
    pub tertiary: bool,

    pub back: bool,
    pub enter: bool,

    pub zoom_in: bool,
    pub zoom_out: bool,

    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}
