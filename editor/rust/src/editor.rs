use common::idx::SystemId;
use common::intersection::*;
use common::systems::*;
use common::time::Time;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;
use rand::Rng;

use crate::generation::generate_system;
use crate::util::glam_to_godot;
use crate::util::godot_to_glam;

enum Selected {
    System { system_id: SystemId },
    Nothing,
}

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Editor {
    camera: Option<Ref<Camera2D>>,
    time_multiplier: f32,
    delta: f32,
    time: Time,
    data: Systems,
    systems_acc: AccelerationStructure<SystemId, NoFilter>,
    selected: Selected,
    /// is_moving, new_pos_valid
    moving_selected: (bool, bool),
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            camera: None,
            time_multiplier: 1.0,
            delta: 0.0,
            time: Time::default(),
            data: Systems::default(),
            systems_acc: AccelerationStructure::new(),
            selected: Selected::Nothing,
            moving_selected: (false, true),
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

        if self.moving_selected.0 {
            // Check if new position is valid.
            match self.selected {
                Selected::System { system_id } => {
                    let mouse_pos = godot_to_glam(owner.get_global_mouse_position());
                    let collider = Collider::new(self.data.systems[system_id].bound, mouse_pos);
                    self.moving_selected.1 = true;

                    // Check that new system position does not intersect any other system.
                    if self
                        .systems_acc
                        .intersect_collider(collider)
                        .into_iter()
                        .any(|id| id != system_id)
                    {
                        self.moving_selected.1 = false;
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
        render(self, owner);
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
    unsafe fn get_num_system(&mut self, _owner: &Node2D) -> i64 {
        self.data.systems.len() as i64
    }

    #[export]
    unsafe fn get_bound(&mut self, _owner: &Node2D) -> f32 {
        self.data.bound
    }

    #[export]
    unsafe fn get_total_num_planet(&mut self, _owner: &Node2D) -> i64 {
        self.data.total_num_planet as i64
    }

    #[export]
    unsafe fn set_time_multiplier(&mut self, _owner: &Node2D, time_multiplier: f32) {
        self.time_multiplier = time_multiplier;
    }

    #[export]
    unsafe fn set_camera(&mut self, _owner: &Node2D, camera: Ref<Camera2D>) {
        self.camera = Some(camera);
    }

    /// Select something for editing.
    #[export]
    unsafe fn select(&mut self, owner: &Node2D) {
        let mouse_pos = godot_to_glam(owner.get_global_mouse_position());

        if let Some(system_id) = self.systems_acc.intersect_point_first(mouse_pos) {
            self.selected = Selected::System { system_id };
            godot_print!("selected a system.");
        } else {
            // We select nothing.
            godot_print!("selected nothing!");
            self.selected = Selected::Nothing;
        }
    }

    #[export]
    unsafe fn delete_selected(&mut self, _owner: &Node2D) {
        match self.selected {
            Selected::System { system_id } => {
                self.data.systems.swap_remove(system_id.0 as usize);

                update_internals(self);
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
            match self.selected {
                Selected::System { system_id } => {
                    let mouse_pos = godot_to_glam(owner.get_global_mouse_position());
                    let collider = Collider::new(self.data.systems[system_id].bound, mouse_pos);

                    // Check that new system position does not intersect any other system.
                    if !self
                        .systems_acc
                        .intersect_collider(collider)
                        .into_iter()
                        .any(|id| id != system_id)
                    {
                        let system = &mut self.data.systems[system_id];

                        // Update selected system position.
                        system.position = mouse_pos;

                        update_internals(self);
                    }
                }
                Selected::Nothing => {}
            }

            self.selected = Selected::Nothing;
        }

        self.moving_selected.0 = toggle;
        godot_print!("Moving: {}", self.moving_selected.0);
    }

    #[export]
    unsafe fn generate(
        &mut self,
        owner: &Node2D,
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
            let position =
                center_position + rng.gen::<Vec2>() * brush_radius * 2.0 - Vec2::new(brush_radius, brush_radius);
            if position.distance(center_position) > brush_radius {
                // We are outside the brush.
                continue;
            }
            let new_system = generate_system(position, rng.gen_range(min_size..max_size) * 1.20);
            let collider = Collider::new(new_system.bound + min_distance, new_system.position);

            for other in new_systems
                .iter()
                .map(|system| Collider::new(system.bound, system.position))
            {
                if collider.intersection_test(other) {
                    continue 'outter;
                }
            }

            new_systems.push(new_system);
        }

        for new_system in new_systems.into_iter() {
            let collider = Collider::new(new_system.bound + min_distance, new_system.position);
            if self.systems_acc.intersect_collider_first(collider).is_none() {
                self.data.systems.push(new_system);
            }
        }

        update_internals(self);
    }

    /// Load only the systems from file.
    #[export]
    unsafe fn load_systems(&mut self, _owner: &Node2D, data: PoolArray<u8>) -> bool {
        self.selected = Selected::Nothing;
        match load_data(data) {
            Ok(data) => {
                self.data.systems = data.systems;
                self.data.bound = data.bound;
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
    #[export]
    unsafe fn export_systems(&mut self, _owner: &Node2D) -> PoolArray<u8> {
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
        .extend(editor.data.systems.iter().zip(0u16..).map(|(system, system_id)| {
            (
                Collider::new(system.bound, system.position),
                SystemId(system_id),
                NoFilter::default(),
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
    let time = editor.time.tick as f32 + editor.delta;
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
    for i in 0..32 {
        let start = part * f64::from(i) * 2.0;
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

    for system in editor.data.systems.iter() {
        // Do not draw system that are not on screen.
        if system.position.x + system.bound < rect.position.x
            || system.position.x - system.bound > rect.position.x + rect.size.x
            || system.position.y + system.bound < rect.position.y
            || system.position.y - system.bound > rect.position.y + rect.size.y
        {
            continue;
        }

        // Draw system bound.
        owner.draw_circle(
            glam_to_godot(system.position),
            system.bound.into(),
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
                glam_to_godot(planet.relative_orbit.to_position(time, system.position)),
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
    let (pos, radius) = match editor.selected {
        Selected::System { system_id } => {
            let system = &editor.data.systems[system_id];
            (system.position, system.bound)
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
    if editor.moving_selected.0 {
        let (r, g) = if editor.moving_selected.1 {
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
