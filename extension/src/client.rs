use super::*;
use crate::{
    client_battlescape::{ClientBattlescape, ClientType},
    client_config::ClientConfig,
};
use data::*;
use godot::{
    engine::{node::InternalMode, Engine},
    prelude::*,
};
use player_inputs::PlayerInputs;

#[derive(GodotClass)]
#[class(base=Node)]
struct Client {
    inputs: PlayerInputs,
    focus: Option<i64>,
    bcs: AHashMap<i64, ClientBattlescape>,
    mc: Gd<Node2D>,
    client_config: ClientConfig,
    #[base]
    base: Base<Node>,
}
impl Client {
    fn focused_battlescape(&mut self) -> Option<&mut ClientBattlescape>{
        self.focus.and_then(|focus| self.bcs.get_mut(&focus))
    } 
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
        if self.bcs.contains_key(&id) {
            self.focus = Some(id);
        } else {
            log::warn!("Can not focus battlescape {}. Not found. Ignoring...", id);
        }
    }

    #[func]
    fn c_bs_add_fleet(&mut self, fleet_id: i64, team: u32, owner: ) {

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
        let mc = Node2D::new_alloc();
        base.add_child(
            mc.share().upcast(),
            false,
            InternalMode::INTERNAL_MODE_DISABLED,
        );

        Self {
            inputs: Default::default(),
            bcs: Default::default(),
            mc,
            client_config,
            focus: None,
            base,
        }
    }

    fn ready(&mut self) {
        Data::clear();
    }

    fn process(&mut self, delta: f64) {
        // TODO: Do not want to run in edit. Maybe this won't be needed later?
        if Engine::singleton().is_editor_hint() {
            return;
        }

        for (id, bc) in self.bcs.iter_mut() {
            // Only give inputs to focused bc.
            let inputs = self.focus.and_then(|focus| {
                if *id == focus {
                    Some(&mut self.inputs)
                } else {
                    None
                }
            });

            if let Some(cmds) = bc.update(delta as f32, inputs) {
                // TODO: Send cmds to server.
            }
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
