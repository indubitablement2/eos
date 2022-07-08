use ::utils::acc::*;
use common::idx::SystemId;
use common::system::*;

use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;
use rand::Rng;

use crate::generation::generate_system;
use crate::util::*;

enum Selected {
    System {
        system_id: SystemId,
        moving: bool,
        current_pos_valid: bool,
    },
    Nothing,
}

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Editor {
    camera: Option<Ref<Camera2D>>,
    time_multiplier: f32,
    timef: f32,
    data: Systems,
    systems_acc: AccelerationStructure<Circle, SystemId>,
    selected: Selected,
    mouse_pos: Vec2,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            camera: None,
            time_multiplier: 1.0,
            timef: 0.0,
            data: Systems::default(),
            systems_acc: AccelerationStructure::new(),
            selected: Selected::Nothing,
            mouse_pos: Vec2::ZERO,
        }
    }
}

#[methods]
impl Editor {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        Editor::default()
    }

    #[godot]
    unsafe fn _process(&mut self, #[base] owner: &Node2D, delta: f32) {
        // Increment time.
        self.timef += delta * self.time_multiplier;

        // Get mouse pos.
        self.mouse_pos = godot_to_glam(owner.get_global_mouse_position());

        match &mut self.selected {
            Selected::System {
                system_id,
                moving,
                current_pos_valid,
            } => {
                if *moving {
                    // Check if new position is valid.
                    let system = self.data.systems.get(system_id).unwrap();
                    let collider = Circle::new(self.mouse_pos, system.radius);
                    *current_pos_valid = true;

                    // Check that new system position does not intersect any other system.
                    self.systems_acc.intersect(&collider, |_, other_system_id| {
                        if *other_system_id != *system_id {
                            *current_pos_valid = false;
                            true
                        } else {
                            false
                        }
                    });
                }
            }
            Selected::Nothing => {}
        }

        owner.update();
    }

    #[godot]
    unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
        render(self, owner);
    }

    #[godot]
    unsafe fn set_tick(&mut self, timef: f32) {
        self.timef = timef;
    }

    #[godot]
    unsafe fn get_tick(&mut self) -> f32 {
        self.timef
    }

    #[godot]
    unsafe fn get_num_system(&mut self) -> i64 {
        self.data.systems.len() as i64
    }

    #[godot]
    unsafe fn get_bound(&mut self) -> f32 {
        self.data.bound
    }

    #[godot]
    unsafe fn get_total_num_planet(&mut self) -> i64 {
        self.data.total_num_planet as i64
    }

    #[godot]
    unsafe fn set_time_multiplier(&mut self, time_multiplier: f32) {
        self.time_multiplier = time_multiplier;
    }

    #[godot]
    unsafe fn set_camera(&mut self, camera: Ref<Camera2D>) {
        self.camera = Some(camera);
    }

    /// (De)select something for editing.
    #[godot]
    unsafe fn select(&mut self) {
        match &self.selected {
            Selected::System { system_id, moving: _, current_pos_valid: _ } => {
                // Try to deselect our current system.
                let system = self.data.systems.get(system_id).unwrap();
                let collider = Circle::new(system.position, system.radius);
                if !collider.intersection_test_point(self.mouse_pos) {
                    self.selected = Selected::Nothing;
                    godot_print!("Deselected system.");
                }
            }
            Selected::Nothing => {
                // Try to select a new system.
                self.systems_acc.intersect_point(self.mouse_pos, |_, other_system_id| {
                    self.selected = Selected::System {
                        system_id: *other_system_id,
                        moving: false,
                        current_pos_valid: false,
                    };
                    godot_print!("Selected a system.");
                    true
                });
            }
        }
    }

    #[godot]
    unsafe fn delete_selected(&mut self) {
        if let Selected::System {
            system_id,
            moving: _,
            current_pos_valid: _,
        } = self.selected
        {
            self.data.systems.remove(&system_id);
            self.selected = Selected::Nothing;
            godot_print!("Deleted selected system.");
            update_internals(self);
        }
    }

    #[godot]
    unsafe fn toggle_moving_selected(&mut self, toggle: bool) {
        if let Selected::System {
            system_id,
            moving,
            current_pos_valid,
        } = &mut self.selected
        {
            if *moving == toggle {
                return;
            }

            *moving = toggle;
            godot_print!("Moving: {}", toggle);

            if !*moving {
                // Move system to new position.
                if *current_pos_valid {
                    let system = self.data.systems.get_mut(system_id).unwrap();
                    system.position = self.mouse_pos;
                    update_internals(self);
                    godot_print!("Moved system.")
                } else {
                    godot_print!("Could not move system.")
                }
            }
        }
    }

    #[godot]
    unsafe fn generate(
        &mut self,
        #[base] owner: &Node2D,
        min_size: f32,
        max_size: f32,
        num_try: u32,
        brush_radius: f32,
        min_distance: f32,
    ) {
        let mut rng = rand::thread_rng();
        let center_position = godot_to_glam(owner.get_global_mouse_position());

        let mut new_systems: Vec<System> = Vec::new();
        'outter: for _ in 0..num_try {
            let position = center_position + rng.gen::<Vec2>() * brush_radius * 2.0
                - Vec2::new(brush_radius, brush_radius);
            if position.distance(center_position) > brush_radius {
                // We are outside the brush.
                continue;
            }
            let new_system = generate_system(position, rng.gen_range(min_size..max_size));
            let collider = Circle::new(new_system.position, new_system.radius + min_distance);

            // Check if we collide we the other systems we have just generated.
            for other in new_systems
                .iter()
                .map(|system| Circle::new(system.position, system.radius))
            {
                if collider.intersection_test(&other) {
                    continue 'outter;
                }
            }

            new_systems.push(new_system);
        }

        // Check if the systems we generated collide with any already placed system.
        for new_system in new_systems.into_iter() {
            let collider = Circle::new(new_system.position, new_system.radius + min_distance);
            let mut valid = true;
            self.systems_acc.intersect(&collider, |_, _| {
                valid = false;
                true
            });
            if valid {
                let system_id = self.data.next_system_id;
                self.data.next_system_id.0 += 1;
                self.data.systems.insert(system_id, new_system);
            }
        }

        update_internals(self);
    }

    /// Load the systems from file.
    #[godot]
    unsafe fn load_systems(&mut self, data: PoolArray<u8>) -> bool {
        self.selected = Selected::Nothing;
        match load_data(data) {
            Ok(data) => {
                self.data = data;
                update_internals(self);
                true
            }
            Err(err) => {
                godot_warn!("{:?}", err);
                false
            }
        }
    }

    /// Return a bin the data.
    #[godot]
    unsafe fn export_systems(&mut self) -> PoolArray<u8> {
        export_data(&self.data)
    }
}

fn update_internals(editor: &mut Editor) {
    editor.selected = Selected::Nothing;

    // Update systems.
    editor.data.update_all();

    // Update systems acceleration structure.
    editor.systems_acc.clear();
    editor
        .systems_acc
        .extend(editor.data.systems.iter().map(|(system_id, system)| {
            (
                Circle::new(system.position, system.radius),
                *system_id,
            )
        }));
    editor.systems_acc.update();
}

fn load_data(data: PoolArray<u8>) -> Result<Systems, Box<bincode::ErrorKind>> {
    let d = data.read();
    let r = bincode::deserialize(&d);
    if let Err(err) = &r {
        godot_warn!("{:?}", err);
    }

    r
}

fn export_data(data: &Systems) -> PoolArray<u8> {
    PoolArray::from_vec(bincode::serialize(data).unwrap())
}

fn render(editor: &Editor, owner: &Node2D) {
    let cam = if let Some(c) = editor.camera {
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

    // Draw systems bound.
    let part = std::f64::consts::TAU / 64.0;
    for i in (0..64).step_by(2) {
        let start = part * f64::from(i);
        owner.draw_arc(
            Vector2::ZERO,
            editor.data.bound.into(),
            start,
            start + part,
            3,
            Color {
                r: 0.95,
                g: 0.95,
                b: 1.0,
                a: 1.0,
            },
            0.5,
            false,
        );
    }

    for system in editor.data.systems.values() {
        let timef = editor.timef;

        // Do not draw system that are not on screen.
        if system.position.x + system.radius < rect.position.x
            || system.position.x - system.radius > rect.position.x + rect.size.x
            || system.position.y + system.radius < rect.position.y
            || system.position.y - system.radius > rect.position.y + rect.size.y
        {
            continue;
        }

        // Draw system bound.
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

        if system.star.star_type != StarType::Nebula {
            // Draw system's star.
            let (r, g, b) = match system.star.star_type {
                StarType::Star => (1.0, 0.2, 0.0),
                StarType::BlackHole => (0.0, 0.0, 0.0),
                StarType::Nebula => (0.0, 0.0, 0.0),
            };

            owner.draw_circle(
                glam_to_godot(system.position),
                system.star.radius.into(),
                Color { r, g, b, a: 1.0 },
            )
        }

        // Draw system's planet.
        for planet in system.planets.iter() {
            if planet.radius < draw_threshold {
                continue;
            }
            // Draw bodies.
            owner.draw_circle(
                glam_to_godot(planet.relative_orbit.to_position(timef, system.position)),
                planet.radius.into(),
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                },
            );
        }
    }

    // Draw selected.
    if let Selected::System {
        system_id,
        moving,
        current_pos_valid,
    } = &editor.selected
    {
        let system = editor.data.systems.get(system_id).unwrap();

        // Draw selected highlight.
        owner.draw_arc(
            glam_to_godot(system.position),
            system.radius as f64,
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
        if *moving {
            let (r, g) = if *current_pos_valid {
                (0.0, 1.0)
            } else {
                (1.0, 0.0)
            };
            owner.draw_arc(
                glam_to_godot(editor.mouse_pos),
                system.radius as f64,
                0.0,
                std::f64::consts::TAU,
                32,
                Color {
                    r,
                    g,
                    b: 0.0,
                    a: 0.8,
                },
                0.5,
                false,
            );
        }
    }
}
