use super::*;
use crate::{
    client_battlescape::{ClientBattlescape, ClientType},
    client_config::ClientConfig,
};
use data::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
struct Client {
    focus: Option<i64>,
    bcs: AHashMap<i64, ClientBattlescape>,
    client_config: ClientConfig,
    #[base]
    base: Base<Node>,
}
#[godot_api]
impl Client {
    #[func]
    fn try_load_data(&mut self, path: GodotString) {
        Data::try_load_data(path);
    }

    #[func]
    fn clear_data(&mut self) {
        Data::clear();
    }

    #[func]
    fn set_log_level(&mut self, level: u8) {
        self.client_config.log_level = level;
        log::set_max_level(log_level_from_int(level));
    }

    #[func]
    fn new_local_battlescape(&mut self) -> i64 {
        // TODO: Actual id.
        let id = rand::random::<i64>();

        self.bcs.insert(
            id,
            ClientBattlescape::new(
                self.base.share(),
                Default::default(),
                &self.client_config,
                ClientId(0),
                ClientType::Local,
            ),
        );

        id
    }

    #[func]
    fn focus_metascape(&mut self) {
        self.focus = None;
    }

    #[func]
    fn focus_battlescape(&mut self, id: i64) {
        self.focus = Some(id);
    }
}
#[godot_api]
impl GodotExt for Client {
    fn init(base: Base<Node>) -> Self {
        godot_logger::GodotLogger::init();

        // TODO: Load configs from file.
        let client_config: ClientConfig = Default::default();

        // Apply configs.
        log::set_max_level(log_level_from_int(client_config.log_level));

        Self {
            bcs: Default::default(),
            client_config,
            focus: None,
            base,
        }
    }

    fn ready(&mut self) {
        Data::clear();
    }

    fn process(&mut self, delta: f64) {
        if let Some(bc) = self.focus.and_then(|focus|self.bcs.get_mut(&focus)) {
            let input = Input::singleton();
            
        } else {
            self.focus = None;
            // TODO: Give inputs to mc;
        }
        
        for bc in self.bcs.values_mut() {
            if let Some(cmds) = bc.update(delta as f32) {
                // TODO: Send cmds to server.
            }
        }
    }

    // fn input() {

    // }
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
