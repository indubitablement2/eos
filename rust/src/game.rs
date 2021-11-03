use crate::client::Client;
use crate::godot_logger::GodotLogger;
use gdnative::api::*;
use gdnative::prelude::*;
use std::time::Duration;
use strategyscape::generation::GenerationParameters;
use strategyscape::server::Server;
use strategyscape::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for logic.
/// This is either a client (multiplayer) or a server and a client (singleplayer).
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    name: String,
    // Receive input from clients. Send command to clients.
    server: Option<Server>,
    // Send input to server. Receive command from server.
    client: Client,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        if let Ok(client) = Client::connect_local() {
            Game {
                name: String::new(),
                server: Some(Server::new()),
                client,
            }
        } else {
            godot_error!("Could not connect to local host.");
            todo!()
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

    #[export]
    unsafe fn _process(&mut self, _owner: &Node2D, mut delta: f64) {
        // Somehow delta can be negative...
        delta = delta.clamp(0.0, 1.0);

        if let Some(server) = &mut self.server {
            server.tick(Duration::from_secs_f64(delta));
        }
    }

    // #[export]
    // unsafe fn _physic_process(&mut self, _owner: &Node2D, delta: f32) {
    // }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        if let Some(server) = &self.server {
            if let Some(strategyscape) = &server.strategyscape {
                // Draw the systems.
                for (translation, radius) in strategyscape.get_systems() {
                    owner.draw_circle(
                        Vector2 {
                            x: translation.x,
                            y: translation.y,
                        },
                        radius.into(),
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.4,
                        },
                    );
                }

                // Draw the players.
                for (_player, translation, radius) in strategyscape.get_players() {
                    owner.draw_circle(
                        Vector2 {
                            x: translation.x,
                            y: translation.y,
                        },
                        radius.into(),
                        Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.6,
                        },
                    );
                }
            }
        }

        // if let Some(ecs) = &self.ecs {
        //     if let Some(render_res) = ecs.world.get_resource::<RenderRes>() {
        //         let visual_server = gdnative::api::VisualServer::godot_singleton();
        //         visual_server.canvas_item_add_multimesh(
        //             owner.get_canvas(),
        //             render_res.multimesh_rid,
        //             render_res.texture_rid,
        //             render_res.normal_texture_rid
        //         );
        //     }
        // }
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

        if let Some(server) = &mut self.server {
            if let Some(strategyscape) = &mut server.strategyscape {
                let mut gen = GenerationParameters {
                    seed: 1477,
                    rng: GenerationParameters::get_rgn_from_seed(1477),
                    mods: (),
                    system_density_buffer_height: 64,
                    system_density_buffer_width: 64,
                    system_density_buffer: (0..64 * 64).into_iter().map(|_| 0.5f32).collect(),
                    system_density_multiplier: 1.0,
                };

                gen.generate_system(strategyscape);
            }
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

        if let Some(server) = &mut self.server {
            // TODO: Generate a new Strategyscape should not be there.
            if let Some(strategyscape) = &mut server.strategyscape {
                let mut gen = GenerationParameters {
                    seed: 1477,
                    rng: GenerationParameters::get_rgn_from_seed(1477),
                    mods: (),
                    system_density_buffer_height: h as usize,
                    system_density_buffer_width: w as usize,
                    system_density_buffer,
                    system_density_multiplier: 1.0,
                };
                gen.generate_system(strategyscape);

                // TODO: Add ourself as a player should not be there.
                strategyscape.add_player(
                    Player {
                        id: 0,
                        name: "Test".to_string(),
                    },
                    rapier2d::na::vector![0.0f32, 0.0f32],
                );
            }
        }

        self.name = world_name;

        owner.update();
    }
}
