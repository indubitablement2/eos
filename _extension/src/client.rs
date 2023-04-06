use super::*;
use crate::{
    client_battlescape::{ClientBattlescape, ClientType},
    client_config::ClientConfig,
    metascape::{fleet::Fleet, ship::Ship, BattlescapeId},
    util::*,
};
use battlescape::command::*;
use data::*;
use godot::{engine::Engine, prelude::*};

#[derive(GodotClass)]
#[class(base=Node)]
struct Client {
    client_id: ClientId,
    bss: AHashMap<BattlescapeId, Gd<ClientBattlescape>>,
    ms: Gd<Node2D>,
    client_config: ClientConfig,
    #[base]
    base: Base<Node>,
}
impl Client {}
#[godot_api]
impl Client {
    #[func]
    fn metascape(&mut self) -> Gd<Node2D> {
        self.ms.share()
    }

    // ---------- Data ----------

    #[func]
    fn clear_data(&mut self) {
        for (_, mut bs) in self.bss.drain() {
            bs.bind_mut().queue_free();
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        Data::clear();
    }

    #[func]
    fn dbg_print_data(&mut self) {
        log::info!("{:#?}", Data::data());
    }

    #[func]
    fn set_log_level(&mut self, level: u8) {
        self.client_config.log_level = level;
        log::set_max_level(log_level_from_int(level));
    }

    #[func]
    fn new_test_battlescape(&mut self) -> Gd<ClientBattlescape> {
        let mut cmds = Commands::default();

        let ships = ship_data_iter()
            .map(|ship_data_id| Ship {
                ship_data_id,
                condition: Default::default(),
            })
            .collect::<Vec<_>>();

        for i in 0..4 {
            cmds.push(AddFleet {
                fleet_id: FleetId(i),
                fleet: Fleet {
                    owner: Some(ClientId(i)),
                    ships: ships.clone(),
                },
                team: i as u32,
            });
        }

        let replay = Replay::new(Default::default(), Default::default(), vec![cmds]);

        let client_bs = ClientBattlescape::new(
            replay,
            &self.client_config,
            ClientId(0),
            ClientType::LocalCheat,
        );

        add_child_node(&mut self.base, &client_bs);

        if let Some(mut previous) = self.bss.insert(Default::default(), client_bs.share()) {
            previous.bind_mut().queue_free();
        }

        client_bs
    }
}
#[godot_api]
impl GodotExt for Client {
    fn init(mut base: Base<Node>) -> Self {
        godot_logger::GodotLogger::init();

        // TODO: Load configs from file.
        let client_config: ClientConfig = Default::default();

        // Apply configs.
        log::set_max_level(log_level_from_int(client_config.log_level));

        // TODO: Temporary.
        let ms = Node2D::new_alloc();
        add_child_node(&mut base, &ms);

        Self {
            client_id: ClientId(0),
            bss: Default::default(),
            ms,
            client_config,
            base,
        }
    }

    fn ready(&mut self) {
        Data::clear();
    }

    fn process(&mut self, _delta: f64) {
        // TODO: Do not want to run in edit. Maybe this won't be needed later?
        if Engine::singleton().is_editor_hint() {
            return;
        }
    }
}

fn log_level_from_int(level: u8) -> log::LevelFilter {
    match level {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}