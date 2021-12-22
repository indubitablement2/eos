use common::res_time::TimeRes;
use common::system::*;
use gdnative::api::*;
use gdnative::prelude::*;
use rand::random;
use glam::Vec2;

use crate::generation::generate_systems;
use crate::util::glam_to_godot;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct SystemEditor {
    viewport_rect: Rect2,
    time_multiplier: f32,
    delta: f32,
    time: TimeRes,
    systems: Systems,
    editor_systems: Vec<(Vec2, Systems)>
}
impl Default for SystemEditor {
    fn default() -> Self {
        Self {
            viewport_rect: Rect2 { position: Vector2::ZERO, size: Vector2::ZERO },
            time_multiplier: 1.0,
            delta: 0.0,
            time: TimeRes::default(),
            systems: Systems::default(),
            editor_systems: Vec::new(),
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
    unsafe fn _process(&mut self, owner: &Node2D, delta: f32) {
        // Increment time.
        self.delta += delta * self.time_multiplier / common::UPDATE_INTERVAL.as_secs_f32();
        if self.delta >= 1.0 {
            let more = self.delta.floor();
            self.delta -= more;
            self.time.tick += more as u32;
        }

        owner.update();
    }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        render_systems(&owner, &self);
    }

    #[export]
    unsafe fn get_tick(&mut self, _owner: &Node2D) -> i64 {
        self.time.tick as i64
    }

    #[export]
    unsafe fn set_time_multiplier(&mut self, _owner: &Node2D, time_multiplier: f32) {
        self.time_multiplier = time_multiplier;
    }

    #[export]
    unsafe fn set_viewport_rect(&mut self, _owner: &Node2D, rect: Rect2) {
        self.viewport_rect = rect;
    }

    /// Select a systems or system for editing.
    #[export]
    unsafe fn select(&mut self, owner: &Node2D) {
        // Get a systems.
        for systems in self.editor_systems.iter() {
            // if systems.bound.length_squared()
        }
    }

    #[export]
    unsafe fn generate(
        &mut self,
        _owner: &Node2D,
        seed: i64,
        bound: f32,
        radius_min: f32,
        radius_max: f32,
        min_distance: f32,
        system_density: f32,
        system_size: f32,
    ) {
        let seed = if seed.is_negative() {
            random::<u64>()
        } else {
            seed as u64
        };

        self.systems = generate_systems(
            seed,
            bound,
            radius_min,
            radius_max,
            min_distance,
            system_density,
            system_size,
        );
    }
}

fn render_systems(owner: &Node2D, system_editor: &SystemEditor) {
    let time = system_editor.time.tick as f32 + system_editor.delta;
    let rect = system_editor.viewport_rect;


    for system in system_editor.systems.0.iter() {
        if system.position.x + system.radius < rect.position.x ||
        system.position.x - system.radius > rect.position.x + rect.size.x ||
        system.position.y + system.radius < rect.position.y ||
        system.position.y - system.radius > rect.position.y + rect.size.y {
            continue;
        }


        // Draw system.
        owner.draw_circle(
            glam_to_godot(system.position),
            system.radius.into(),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.1,
            },
        );

        for (pos, body) in system.get_bodies_world_position(time).into_iter().zip(system.bodies.iter()) {
            // Draw bodies.
            owner.draw_circle(
                glam_to_godot(pos),
                body.radius.into(),
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 1.0,
                    a: 0.5,
                },
            );
        }
    }
}
