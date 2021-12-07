use std::net::IpAddr;
use std::net::SocketAddr;

use crate::client::Client;
use common::packets::*;
use common::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::vec2;
use glam::Vec2;

struct ClientMetascape {
    fleets: Vec<Vec2>,
}
impl ClientMetascape {
    fn new() -> Self {
        Self { fleets: Vec::new() }
    }
}

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for logic.
/// This is either a client (multiplayer) or a server and a client (singleplayer).
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    name: String,
    // Send input to server. Receive command from server.
    client: Option<Client>,
    // TODO: Implement client matascape.
    client_metascape: ClientMetascape,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        Game {
            name: String::new(),
            client: None,
            client_metascape: ClientMetascape::new(),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {}

    /// For some reason this gets called twice.
    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {}

    #[export]
    unsafe fn _process(&mut self, owner: &Node2D, mut delta: f64) {
        // Somehow delta can be negative...
        delta = delta.clamp(0.0, 1.0);

        // Handle server packets.
        if let Some(client) = &self.client {
            loop {
                match client.udp_receiver.try_recv() {
                    Ok(udp_packet) => match udp_packet {
                        UdpServer::Battlescape { client_inputs, tick } => todo!(),
                        UdpServer::Metascape { fleets_position, tick } => {
                            self.client_metascape.fleets = fleets_position;
                            owner.update();
                        }
                    },
                    Err(err) => {
                        if err == crossbeam_channel::TryRecvError::Disconnected {
                            error!("Client disconnected.");
                        }
                        break;
                    }
                }
            }
        }
    }

    #[export]
    unsafe fn _physics_process(&mut self, owner: &Node2D, _delta: f64) {
        // Send client packets.
        if let Some(client) = &self.client {
            let wish_pos = owner.get_global_mouse_position();
            let packet = UdpClient::Metascape {
                wish_position: vec2(wish_pos.x, wish_pos.y),
            };
            client.udp_sender.send(packet).unwrap();
        }
    }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        for position in &self.client_metascape.fleets {
            // Draw all fleet.
            owner.draw_circle(
                Vector2 {
                    x: position.x,
                    y: position.y,
                },
                10.0,
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 0.5,
                },
            );
        }
    }

    #[export]
    unsafe fn connect_client(&mut self, _owner: &Node2D, godot_addr: StringArray) -> bool {
        if self.client.is_some() {
            return true;
        } else {
            let godot_addr_read = godot_addr.read();
            for s in godot_addr_read.iter() {
                if let Ok(addr) = s.to_string().parse::<IpAddr>() {
                    let server_addresses = ServerAddresses {
                        tcp_address: SocketAddr::new(addr, SERVER_PORT),
                        udp_address: SocketAddr::new(addr, SERVER_PORT),
                    };

                    if let Ok(new_client) = Client::new(server_addresses) {
                        self.client.replace(new_client);
                        return true;
                    }
                }
            }
        }
        false
    }
}

//     /// Generate a new Metascape.
//     #[export]
//     unsafe fn generate_metascape(
//         &mut self,
//         owner: &Node2D,
//         metascape_name: String,
//         bound: f32,
//         system_density_img: Ref<Image, Shared>,
//     ) {
//         // Create metascape params
//         let parameters = MetascapeParameters {
//             bound,
//             movement_friction: 0.95,
//         };

//         // Connect localy.
//         let mut metascape = MetascapeWrapper::new(true, parameters).unwrap();
//         let client = Client::new(metascape.get_addresses()).unwrap();

//         let mut gen = GenerationParameters::new(0, img_to_generation_mask(system_density_img), GenerationMask::default());

//         // Generate random Metascape.
//         metascape.generate(&mut gen);

//         self.name = metascape_name;
//         self.metascape = Some(metascape);
//         self.client = Some(client);

//         owner.update();
//     }

// fn img_to_generation_mask(img: Ref<Image, Shared>) -> GenerationMask {
//     // Extract the density buffer from the image.
//     let img = unsafe { img.assume_safe() };
//     let (h, w) = (img.get_height(), img.get_width());
//     let mut system_density_buffer = Vec::with_capacity((w * h) as usize);
//     img.lock();
//     for y in 0..h {
//         for x in 0..w {
//             let col = img.get_pixel(x, y);
//             // Only read mask image (black and white).
//             system_density_buffer.push(col.r);
//         }
//     }
//     img.unlock();

//     // Create GenerationParameters.
//     GenerationMask {
//         width: w as usize,
//         height: h as usize,
//         buffer: system_density_buffer,
//         multiplier: 1.0,
//     }
// }
