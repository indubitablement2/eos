use crate::client::Client;
use common::generation::GenerationParameters;
use common::metascape::*;
use common::packets::UdpClient;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::vec2;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for logic.
/// This is either a client (multiplayer) or a server and a client (singleplayer).
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    name: String,
    // Receive input from clients. Send command to clients.
    metascape: Option<Metascape>,
    // Send input to server. Receive command from server.
    client: Option<Client>,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        // Connect localy.
        let metascape = Metascape::new(true).unwrap();
        let client = Client::new(metascape.connection_manager.get_addresses()).unwrap();

        Game {
            name: String::new(),
            metascape: Some(metascape),
            client: Some(client),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {}

    /// For some reason this gets called twice.
    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {}

    // #[export]
    // unsafe fn _process(&mut self, _owner: &Node2D, mut delta: f64) {
    //     // Somehow delta can be negative...
    //     delta = delta.clamp(0.0, 1.0);
    // }

    #[export]
    unsafe fn _physics_process(&mut self, owner: &Node2D, _delta: f64) {
        if let Some(client) = &mut self.client {
            let wish_pos = owner.get_global_mouse_position();
            let packet = UdpClient::Metascape {
                wish_position: vec2(wish_pos.x, wish_pos.y),
            };
            client.udp_sender.send(packet).unwrap();
        }

        if let Some(metascape) = &mut self.metascape {
            metascape.update();
        }

        owner.update();
    }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        if let Some(metascape) = &self.metascape {
            // Draw all Colliders.
            for collider in metascape.get_colliders().into_iter() {
                owner.draw_circle(
                    Vector2 {
                        x: collider.position.x,
                        y: collider.position.y,
                    },
                    collider.radius.into(),
                    Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 0.5,
                    },
                );
            }
        }
    }

    /// Load a world.
    #[export]
    unsafe fn load_world(&mut self, owner: &Node2D, world_name: String) {
        // let world_path: String = format!("{}{}/", WORLDS_PATH, world_name);

        // // Load Def or create a new one.
        // let def = Def::load(&world_path, false, true);

        // // Create Ecs.
        // self.ecs = Some(Ecs::new(owner, &def));
        // self.def = Some(def);

        if let Some(metascape) = &mut self.metascape {
            let mut gen = GenerationParameters {
                seed: 1477,
                rng: GenerationParameters::get_rgn_from_seed(1477),
                mods: (),
                system_density_buffer_height: 64,
                system_density_buffer_width: 64,
                system_density_buffer: (0..64 * 64).into_iter().map(|_| 0.5f32).collect(),
                system_density_multiplier: 1.0,
            };

            gen.generate_system(metascape);
        }

        self.name = world_name;

        owner.update();
    }

    /// Generate a new Metascape.
    #[export]
    unsafe fn generate_metascape(&mut self, owner: &Node2D, metascape_name: String, density_img: Ref<Image, Shared>) {
        // Extract the density buffer from the image.
        let density_img = density_img.assume_safe();
        let (h, w) = (density_img.get_height(), density_img.get_width());
        let mut system_density_buffer = Vec::with_capacity((w * h) as usize);
        density_img.lock();
        for y in 0..h {
            for x in 0..w {
                let col = density_img.get_pixel(x, y);
                // Only read mask image (black and white).
                system_density_buffer.push(col.r);
            }
        }
        density_img.unlock();

        // Create GenerationParameters.
        let mut gen = GenerationParameters {
            seed: 1477,
            rng: GenerationParameters::get_rgn_from_seed(1477),
            mods: (),
            system_density_buffer_height: h as usize,
            system_density_buffer_width: w as usize,
            system_density_buffer,
            system_density_multiplier: 1.0,
        };

        // Generate some System.
        if let Some(metascape) = &mut self.metascape {
            gen.generate_system(metascape);
        }

        self.name = metascape_name;

        owner.update();
    }
}
