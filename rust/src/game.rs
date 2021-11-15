use crate::client::Client;
use common::generation::GenerationMask;
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
        Game {
            name: String::new(),
            metascape: None,
            client: None,
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

    // #[export]
    // unsafe fn _physics_process(&mut self, owner: &Node2D, _delta: f64) {
    //     if let Some(client) = &mut self.client {
    //         let wish_pos = owner.get_global_mouse_position();
    //         let packet = UdpClient::Metascape {
    //             wish_position: vec2(wish_pos.x, wish_pos.y),
    //         };
    //         client.udp_sender.send(packet).unwrap();
    //     }

    //     if let Some(metascape) = &mut self.metascape {
    //         metascape.update();
    //     }

    //     owner.update();
    // }

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

            // Draw System row separation.
            for height in metascape.get_system_rows_separation() {
                owner.draw_line(
                    Vector2 {
                        x: -metascape.bound,
                        y: height,
                    },
                    Vector2 {
                        x: metascape.bound,
                        y: height,
                    },
                    Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.3,
                        a: 0.5,
                    },
                    4.0,
                    false,
                );
            }
        }
    }

    #[export]
    unsafe fn manual_update(&mut self, owner: &Node2D) {
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

    /// Generate a new Metascape.
    #[export]
    unsafe fn generate_metascape(
        &mut self,
        owner: &Node2D,
        metascape_name: String,
        bound: f32,
        system_density_img: Ref<Image, Shared>,
    ) {
        // Connect localy.
        let mut metascape = Metascape::new(true, bound).unwrap();
        let client = Client::new(metascape.connection_manager.get_addresses()).unwrap();

        let mut gen = GenerationParameters::new(0, img_to_generation_mask(system_density_img), GenerationMask::default());

        // Generate random Metascape.
        metascape.generate(&mut gen);

        self.name = metascape_name;
        self.metascape = Some(metascape);
        self.client = Some(client);

        owner.update();
    }
}

fn img_to_generation_mask(img: Ref<Image, Shared>) -> GenerationMask {
    // Extract the density buffer from the image.
    let img = unsafe { img.assume_safe() };
    let (h, w) = (img.get_height(), img.get_width());
    let mut system_density_buffer = Vec::with_capacity((w * h) as usize);
    img.lock();
    for y in 0..h {
        for x in 0..w {
            let col = img.get_pixel(x, y);
            // Only read mask image (black and white).
            system_density_buffer.push(col.r);
        }
    }
    img.unlock();

    // Create GenerationParameters.
    GenerationMask {
        width: w as usize,
        height: h as usize,
        buffer: system_density_buffer,
        multiplier: 1.0,
    }
}
