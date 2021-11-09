use crate::client::Client;
use common::generation::GenerationParameters;
use common::metascape::*;
use common::packets::UdpClient;
use gdnative::api::*;
use gdnative::prelude::*;
use nalgebra::vector;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::time::Duration;

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
        // // Add my fleet.
        // metascape.add_client_with_fleet(ClientID { id: 53 }, client.get_local_addr(), vector![0.0, 0.0]);

        Game {
            name: String::new(),
            metascape: Some(metascape),
            client: Some(client),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {
        // let asd = autoload::<Node>("asd").unwrap();
        // let cm = asd.cast_instance::<ConfigManager>().unwrap().claim().assume_safe();
        // let n = cm.map(|a, b| {
        //     a.config.audio_config.music_volume;
        // });
    }

    /// For some reason this gets called twice.
    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {
        // self.save_world(owner);

        // // Free the rids we created.
        // if let Some(ecs) = &self.ecs {
        //     if let Some(render_res) = ecs.world.get_resource::<RenderRes>() {
        //         let visual_server = gdnative::api::VisualServer::godot_singleton();
        //         visual_server.free_rid(render_res.multimesh_rid);
        //         visual_server.free_rid(render_res.mesh_rid);
        //     }
        // }
    }

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
                wish_position: vector![wish_pos.x, wish_pos.y],
            };
            // client.send_udp(packet).unwrap();
        }

        if let Some(metascape) = &mut self.metascape {
            metascape.update();
        }

        owner.update();
    }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        if let Some(metascape) = &self.metascape {
            // Draw the balls.
            for (pos, radius) in metascape.get_balls().into_iter() {
                owner.draw_circle(
                    Vector2 { x: pos.x, y: pos.y },
                    radius.into(),
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
        // // TODO: Add parameter in load_world function.
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

    /// Generate a new world.
    #[export]
    unsafe fn generate_world(&mut self, owner: &Node2D, world_name: String, density_img: Ref<Image, Shared>) {
        // let world_path: String = format!("{}{}/", WORLDS_PATH, world_name);

        // Extract the density buffer from the image.
        let density_img = density_img.assume_safe();
        let (h, w) = (density_img.get_height(), density_img.get_width());
        let mut system_density_buffer = Vec::with_capacity((w * h) as usize);
        density_img.lock();
        for y in 0..h {
            for x in 0..w {
                let col = density_img.get_pixel(x, y);
                system_density_buffer.push(col.r);
            }
        }
        density_img.unlock();

        // if let Some(server) = &mut self.server {
        //     // TODO: Generate a new Metascape should not be there.
        //     if let Some(metascape) = &mut server.metascape {
        //         let mut gen = GenerationParameters {
        //             seed: 1477,
        //             rng: GenerationParameters::get_rgn_from_seed(1477),
        //             mods: (),
        //             system_density_buffer_height: h as usize,
        //             system_density_buffer_width: w as usize,
        //             system_density_buffer,
        //             system_density_multiplier: 1.0,
        //         };
        //         gen.generate_system(metascape);

        //         // TODO: Add ourself as a player should not be there.
        //         // metascape.add_player(
        //         //     Player {
        //         //         id: 0,
        //         //     },
        //         //     rapier2d::na::vector![0.0f32, 0.0f32],
        //         // );
        //     }
        // }

        self.name = world_name;

        owner.update();
    }
}
