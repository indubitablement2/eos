use common::res_time::TimeRes;
use common::system::Systems;
use gdnative::api::*;
use gdnative::prelude::*;

use crate::generation::generate_systems;
use crate::generation_mask::GenerationMask;
use crate::generation_mask::GenerationParameters;
use crate::util::glam_to_godot;


#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct SystemEditor {
    delta: f32,
    time: TimeRes,
    systems: Systems
}
impl Default for SystemEditor {
    fn default() -> Self {
        Self {
            delta: 0.0,
            time: TimeRes::default(),
            systems: Systems::default(),
        }
    }
}

#[methods]
impl SystemEditor {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        SystemEditor::default()
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {}

    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {}

    #[export]
    unsafe fn _process(&mut self, _owner: &Node2D, delta: f32) {
        // Increment time.
        self.delta += delta / common::UPDATE_INTERVAL.as_secs_f32();
        if self.delta >= 1.0 {
            self.delta -= 1.0;
            self.time.tick += 1;
        }
    }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        debug!("Rendering systems...");
        render_systems(&owner, &self);
    }

    #[export]
    unsafe fn generate(&mut self, _owner: &Node2D) {
        self.systems = generate_systems(&GenerationParameters::default());
    }
}

fn render_systems(owner: &Node2D, system_editor: &SystemEditor) {
    for system in system_editor.systems.0.iter() {
        owner.draw_circle(glam_to_godot(system.position), system.radius.into(), Color { r: 1.0, g: 0.0, b: 1.0, a: 0.5 });
    }
}