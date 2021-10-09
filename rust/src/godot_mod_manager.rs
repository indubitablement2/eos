use gdnative::prelude::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_builder)]
pub struct ModManager {}

#[methods]
impl ModManager {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        ModManager {}
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node) {}

    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node) {}
}
