use crate::ecs_input::*;
use crate::ecs_schedue::*;
use bevy_ecs::prelude::*;
use gdnative::api::*;
use gdnative::prelude::*;

/// Layer between godot and ecs.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Sprite)]
#[register_with(Self::register_builder)]
pub struct GodotEcs {
    option_ecs_world: Option<EcsWorld>,
}

#[methods]
impl GodotEcs {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Sprite) -> Self {
        GodotEcs {
            option_ecs_world: Option::None,
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Sprite) {
        // owner.set_texture(self.chunk_info.texture);
    }

    /// Render interpolated between previous and current ecs update.
    #[export]
    unsafe fn _process(&mut self, owner: &Sprite, delta: f32) {
        if let Some(ecs) = &mut self.option_ecs_world {
            ecs.run(delta);
        }
    }

    #[export]
    unsafe fn _physics_process(&mut self, owner: &Sprite, delta: f32) {}
}
