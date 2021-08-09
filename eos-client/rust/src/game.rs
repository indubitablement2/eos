use eos_common::connection_manager::Connection;
use eos_common::connection_manager::PollingThread;
use eos_common::const_var::APP_VERSION;
use eos_common::const_var::SERVER_ADDR;
use eos_common::idx::ClientId;
use eos_common::packet_common::*;
use gdnative::api::*;
use gdnative::prelude::*;

/// The Game "class"
#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_builder)]
pub struct Game {
    pt: PollingThread,
    connection: Connection,
    tick: u64,
}

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {
    }

    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        godot_print!("Connecting...");
        let pt = PollingThread::new(false);
        let socket = std::net::TcpStream::connect(SERVER_ADDR).unwrap();
        let tokio_socket = pt.connection_starter.convert_std_to_tokio(socket).unwrap();
        let connection = pt.connection_starter.create_connection(tokio_socket);

        godot_print!("Starting steam...");
        let steam = Steam::godot_singleton();
        steam.steamInit(false);
        let id = u64::from_be_bytes(steam.getSteamID().to_be_bytes());
        
        godot_print!("Making auth ticket...");
        let result = steam.getAuthSessionTicket();
        godot_print!("{:?}\n", &result);

        // Size
        let size = result.get("size").unwrap().to_u64();
        godot_print!("size: {}\n", size);

        // Bin
        let mut ticket_bin: Vec<u8> = result.get("buffer").unwrap().to_byte_array().read().to_vec();
        godot_print!("bin: {:?}", &ticket_bin);
        godot_print!("bin.len: {}", ticket_bin.len());
        ticket_bin.truncate(size as usize);
        godot_print!("bin_trunc: {:?}", &ticket_bin);
        godot_print!("bin_trunc.len: {}\n", ticket_bin.len());

        // Hex
        // let mut ticket_hex = [0u8; 4096];
        // ticket_hex.fill(0);
        // hex::encode_to_slice(ticket_bin, &mut ticket_hex).unwrap();
        // godot_print!("hex: {:?}", &ticket_hex);

        // Str
        // let ticket_str = unsafe { String::from_utf8(ticket_bin) };
        // godot_print!("str: {:?}", &ticket_str);

        let ticket_str_2 = format!("{:x?}", ticket_bin);
        godot_print!("str_2: {:?}\n", &ticket_str_2);

        let mut ticket_str_3 = String::new();
        for i in 0..size as usize {
            ticket_str_3 += format!("{:02x?}", ticket_bin[i]).as_str();
        }
        godot_print!("str_3: {:?}\n", &ticket_str_3);

        let packet = OtherPacket::ClientLogin {
            app_version: APP_VERSION,
            steam_id: ClientId(id),
            ticket: ticket_str_3,
        };
        godot_print!("{:?}\n", &packet);

        std::thread::sleep(std::time::Duration::from_secs(2));
        let success = connection.send_packet(packet.serialize());
        godot_print!("Packet sent: {}", success);

        Game {
            pt,
            connection,
            tick: 0,
        }
    }

    // In order to make a method known to Godot, the #[export] attribute has to be used.
    // In Godot script-classes do not actually inherit the parent class.
    // Instead they are "attached" to the parent object, called the "owner".
    // The owner is passed to every single exposed method.
    #[export]
    unsafe fn _ready(&mut self, _owner: &Node) {
    }

    // This function will be called in every frame
    #[export]
    unsafe fn _process(&mut self, _owner: &Node, delta: f64) {
        self.tick += 1;
        if self.tick % 10 == 0 {
            self.pt.poll();
        }

        match self.connection.other_packet_receiver.try_recv() {
            Ok(packet) => {
                godot_print!("Server: {:?}", packet);
            }
            Err(err) => {
                if err == flume::TryRecvError::Disconnected && self.tick % 60 == 0 {
                    godot_print!("Lost connection to server...");
                }
            }
        }
    }
}
