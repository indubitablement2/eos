use common::intersection::AccelerationStructure;
use common::intersection::Collider;
use common::res_time::TimeRes;
use common::system::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;
use indexmap::IndexMap;
use rand::random;

use crate::generation::generate_systems;
use crate::util::glam_to_godot;
use crate::util::godot_to_glam;

enum Selected {
    Systems {
        editor_systems_index: usize,
    },
    System {
        editor_systems_index: usize,
        system_index: usize,
    },
    Nothing,
}

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct SystemEditor {
    camera: Option<Ref<Camera2D>>,
    time_multiplier: f32,
    delta: f32,
    time: TimeRes,
    editor_systems: Vec<(Vec2, Systems, f32)>,
    selected: Selected,
    /// moving, new_pos_valid
    moving_selected: (bool, bool),
}
impl Default for SystemEditor {
    fn default() -> Self {
        Self {
            camera: None,
            time_multiplier: 1.0,
            delta: 0.0,
            time: TimeRes::default(),
            editor_systems: Vec::new(),
            selected: Selected::Nothing,
            moving_selected: (false, true),
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

        // First editor systems is always at (0, 0).
        if let Some((editor_systems_pos, _, _)) = self.editor_systems.first_mut() {
            *editor_systems_pos = Vec2::ZERO;
        }

        if self.moving_selected.0 {
            // Check if new position is valid.
            match self.selected {
                Selected::Systems {
                    editor_systems_index: _,
                } => self.moving_selected.1 = true,
                Selected::System {
                    editor_systems_index,
                    system_index,
                } => {
                    let mouse_pos = godot_to_glam(owner.get_global_mouse_position());
                    let (systems_pos, editor_systems, _) = &self.editor_systems[editor_systems_index];
                    let collider = Collider::new_idless(editor_systems.0[system_index].bound, mouse_pos);
                    self.moving_selected.1 = true;

                    // Check that new position does not intersect any other system.
                    for (id, system) in editor_systems.0.iter().enumerate() {
                        if id == system_index {
                            continue;
                        }

                        let other = Collider::new_idless(system.bound, system.position + *systems_pos);

                        if collider.intersection_test(other) {
                            self.moving_selected.1 = false;
                            break;
                        }
                    }
                }
                Selected::Nothing => {
                    self.moving_selected = (false, true);
                }
            }
        }

        owner.update();
    }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        render_systems(&self, &owner);
    }

    #[export]
    unsafe fn set_tick(&mut self, _owner: &Node2D, tick: u32) {
        self.time.tick = tick;
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
    unsafe fn set_camera(&mut self, _owner: &Node2D, camera: Ref<Camera2D>) {
        self.camera = Some(camera);
    }

    /// Select a systems or system for editing.
    #[export]
    unsafe fn select(&mut self, owner: &Node2D) {
        let mouse_pos = godot_to_glam(owner.get_global_mouse_position());

        for (editor_systems_index, (systems_pos, systems, systems_bound)) in
            self.editor_systems.iter().enumerate().rev()
        {
            let systems_relative_mouse = mouse_pos - *systems_pos;

            // Check if mouse overlap this systems.
            if (systems_relative_mouse).length_squared() <= systems_bound.powi(2) {
                for (system_index, system) in systems.0.iter().enumerate() {
                    // Check if mouse overlap this system.
                    if (systems_relative_mouse).distance_squared(system.position) <= system.bound.powi(2) {
                        // We select a system.
                        self.selected = Selected::System {
                            editor_systems_index,
                            system_index,
                        };
                        godot_print!("selected a system.");
                        return;
                    }
                }
                // We select a systems.
                self.selected = Selected::Systems { editor_systems_index };
                godot_print!("selected an editor systems.");
                return;
            }
        }
        // We select nothing.
        godot_print!("selected nothing!");
        self.selected = Selected::Nothing;
    }

    #[export]
    unsafe fn delete_selected(&mut self, _owner: &Node2D) {
        match self.selected {
            Selected::Systems { editor_systems_index } => {
                self.editor_systems.remove(editor_systems_index);
            }
            Selected::System {
                editor_systems_index,
                system_index,
            } => {
                self.editor_systems[editor_systems_index].1 .0.remove(system_index);
                // Update systems bound.
                let new_bound = self.editor_systems[editor_systems_index].1.get_bound();
                self.editor_systems[editor_systems_index].2 = new_bound;

                // Update systems bound.
                let new_bound = self.editor_systems[editor_systems_index].1.get_bound();
                self.editor_systems[editor_systems_index].2 = new_bound;

                // Also delete systems if this is the last system
                if self.editor_systems[editor_systems_index].1 .0.is_empty() {
                    self.editor_systems.remove(editor_systems_index);
                }
            }
            Selected::Nothing => {}
        }
        self.selected = Selected::Nothing;
    }

    #[export]
    unsafe fn toggle_moving_selected(&mut self, owner: &Node2D, toggle: bool) {
        if self.moving_selected.0 == toggle {
            return;
        }

        if self.moving_selected.0 && self.moving_selected.1 {
            // Try to drop the selected object here.
            let new_global_pos = godot_to_glam(owner.get_global_mouse_position());
            match self.selected {
                Selected::Systems { editor_systems_index } => {
                    self.editor_systems[editor_systems_index].0 = new_global_pos;
                }
                Selected::System {
                    editor_systems_index,
                    system_index,
                } => {
                    let (editor_systems_pos, editor_systems, _) = &mut self.editor_systems[editor_systems_index];
                    editor_systems.0[system_index].position = new_global_pos - *editor_systems_pos;

                    // Update systems bound.
                    let new_bound = self.editor_systems[editor_systems_index].1.get_bound();
                    self.editor_systems[editor_systems_index].2 = new_bound;

                    self.selected = Selected::Nothing;
                }
                Selected::Nothing => {}
            }
        }

        self.moving_selected.0 = toggle;
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

        let new_system = generate_systems(
            seed,
            bound,
            radius_min,
            radius_max,
            min_distance,
            system_density,
            system_size,
        );

        // Place system.
        let systems_bound = new_system.get_bound();
        self.editor_systems.push((Vec2::ZERO, new_system, systems_bound));
    }

    /// Merge all editor systems and return a file with a map from old to new index.
    #[export]
    unsafe fn merge(&mut self, _owner: &Node2D) -> TypedArray<u8> {
        self.selected = Selected::Nothing;
        merge(self)
    }

    /// Return a bin of all editor systems to resume editing latter.
    #[export]
    unsafe fn export_editor_systems(&mut self, _owner: &Node2D) -> TypedArray<u8> {
        self.selected = Selected::Nothing;
        export_editor_systems(self)
    }

    #[export]
    unsafe fn load_editor_systems(&mut self, _owner: &Node2D, data: TypedArray<u8>) -> bool {
        self.selected = Selected::Nothing;
        load_editor_systems(self, data)
    }

    /// Return a bin the first systems to use ingame.
    #[export]
    unsafe fn export_first_systems(&mut self, _owner: &Node2D) -> TypedArray<u8> {
        self.selected = Selected::Nothing;
        export_first_systems(self)
    }

    #[export]
    unsafe fn load_first_systems(&mut self, _owner: &Node2D, data: TypedArray<u8>) -> bool {
        self.selected = Selected::Nothing;
        load_first_systems(self, data)
    }
}

fn load_first_systems(system_editor: &mut SystemEditor, data: TypedArray<u8>) -> bool {
    let d = data.read();
    if let Ok(result) = bincode::deserialize::<Systems>(&d) {
        system_editor.editor_systems.clear();
        let new_bound = result.get_bound();
        system_editor.editor_systems.push((Vec2::ZERO, result, new_bound));
        true
    } else {
        false
    }
}

fn load_editor_systems(system_editor: &mut SystemEditor, data: TypedArray<u8>) -> bool {
    let d = data.read();
    if let Ok(result) = bincode::deserialize(&d) {
        system_editor.editor_systems = result;
        true
    } else {
        false
    }
}

fn export_first_systems(system_editor: &SystemEditor) -> TypedArray<u8> {
    if let Some((_, systems, _)) = system_editor.editor_systems.first() {
        TypedArray::from_vec(bincode::serialize(&systems).unwrap())
    } else {
        TypedArray::new()
    }
}

fn export_editor_systems(system_editor: &SystemEditor) -> TypedArray<u8> {
    TypedArray::from_vec(bincode::serialize(&system_editor.editor_systems).unwrap())
}

fn merge(system_editor: &mut SystemEditor) -> TypedArray<u8> {
    if system_editor.editor_systems.len() < 2 {
        return TypedArray::new();
    }

    let (_, mut first_systems, _) = system_editor.editor_systems.swap_remove(0);

    // Save the old index (old, new).
    let mut old_index: IndexMap<u32, u32> = (0u32..first_systems.0.len() as u32)
        .into_iter()
        .map(|id| (id, id))
        .collect();

    // Create acceleration structure with the old index.
    let mut acc = AccelerationStructure::new();
    acc.colliders.extend(
        first_systems
            .0
            .iter()
            .zip(0u32..)
            .map(|(system, id)| Collider::new(id, system.bound, system.position)),
    );
    acc.update();

    while let Some((other_pos, other, _)) = system_editor.editor_systems.pop() {
        let mut new_colliders = Vec::new();

        for mut system in other.0.into_iter() {
            system.position += other_pos;
            let collider = Collider::new(u32::MAX, system.bound, system.position);
            if !acc.test_collider(collider) {
                first_systems.0.push(system);
                new_colliders.push(collider);
            }
        }

        acc.colliders.extend(new_colliders.drain(..));
        acc.update();
    }

    // Update index.
    acc.colliders.sort_by(|a, b| {
        a.position
            .y
            .partial_cmp(&b.position.y)
            .expect("this should be a real number.")
    });
    for (new_index, collider) in acc.colliders.iter().enumerate() {
        if let Some((_, v)) = old_index.get_index_mut(collider.id as usize) {
            *v = new_index as u32;
        }
    }

    // Update merged systems
    first_systems.0.sort_by(|a, b| {
        a.position
            .y
            .partial_cmp(&b.position.y)
            .expect("this should be a real number.")
    });

    let new_bound = first_systems.get_bound();
    system_editor.editor_systems = vec![(Vec2::ZERO, first_systems, new_bound)];

    TypedArray::from_vec(bincode::serialize(&old_index).unwrap())
}

fn render_systems(system_editor: &SystemEditor, owner: &Node2D) {
    let time = system_editor.time.tick as f32 + system_editor.delta;
    let cam = if let Some(c) = system_editor.camera {
        c
    } else {
        return;
    };
    let (rect, draw_threshold) = unsafe {
        let mut r = owner
            .get_tree()
            .unwrap()
            .assume_safe()
            .root()
            .unwrap()
            .assume_safe()
            .get_visible_rect();
        r.position += cam.assume_safe().position();
        let zoom = cam.assume_safe().zoom();
        r.size *= zoom;
        r.position -= r.size * 0.5;
        (r, zoom.x * 0.5)
    };
    let mut pos_buffer = Vec::new();

    for (i, (systems_pos, systems, systems_bound)) in system_editor.editor_systems.iter().enumerate() {
        // Draw systems.
        let (a, g) = if i == 0 {
            // This is the og system. Highlight it.
            owner.draw_arc(
                Vector2::ZERO,
                systems_bound.to_owned().into(),
                0.0,
                std::f64::consts::TAU,
                32,
                Color {
                    r: 0.1,
                    g: 0.1,
                    b: 1.0,
                    a: 0.5,
                },
                0.5,
                false,
            );
            (0.15, 1.0)
        } else {
            (0.05, 0.8)
        };
        owner.draw_circle(
            glam_to_godot(*systems_pos),
            systems_bound.to_owned().into(),
            Color { r: 0.8, g, b: 0.8, a },
        );
        for system in systems.0.iter() {
            // Do not draw system that are not on screen.
            let system_pos = *systems_pos + system.position;
            if system_pos.x + system.bound < rect.position.x
                || system_pos.x - system.bound > rect.position.x + rect.size.x
                || system_pos.y + system.bound < rect.position.y
                || system_pos.y - system.bound > rect.position.y + rect.size.y
            {
                continue;
            }

            // Draw system.
            owner.draw_circle(
                glam_to_godot(system_pos),
                system.bound.into(),
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.1,
                },
            );

            pos_buffer.clear();
            system.get_bodies_position(time, &mut pos_buffer);

            for (body, body_pos) in system.bodies.iter().zip(pos_buffer.iter()) {
                if body.radius < draw_threshold {
                    continue;
                }
                // Draw bodies.
                owner.draw_circle(
                    glam_to_godot(*body_pos + *systems_pos),
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

    // Draw selected.
    let (pos, radius) = match system_editor.selected {
        Selected::Systems { editor_systems_index } => {
            let (systems_pos, _, systems_bound) = &system_editor.editor_systems[editor_systems_index];
            (*systems_pos, *systems_bound)
        }
        Selected::System {
            editor_systems_index,
            system_index,
        } => {
            let (systems_pos, editor_systems, _) = &system_editor.editor_systems[editor_systems_index];
            let system = &editor_systems.0[system_index];
            (*systems_pos + system.position, system.bound)
        }
        Selected::Nothing => {
            return;
        }
    };
    owner.draw_arc(
        glam_to_godot(pos),
        radius as f64,
        0.0,
        std::f64::consts::TAU,
        32,
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.8,
        },
        0.5,
        false,
    );

    // Draw moving selected.
    if system_editor.moving_selected.0 {
        let (r, g) = if system_editor.moving_selected.1 {
            (0.0, 1.0)
        } else {
            (1.0, 0.0)
        };
        owner.draw_arc(
            owner.get_global_mouse_position(),
            radius as f64,
            0.0,
            std::f64::consts::TAU,
            32,
            Color { r, g, b: 0.0, a: 0.8 },
            0.5,
            false,
        );
    }
}
