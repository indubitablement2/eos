use super::*;
use crate::{client_battlescape::ClientBattlescape, client_config::ClientConfig};
use data::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Client {
    bcs: Vec<ClientBattlescape>,
    client_config: ClientConfig,
    #[base]
    base: Base<Node>,
}
#[godot_api]
impl Client {
    #[func]
    fn load_data(&mut self, path: GodotString) {
        Data::load_data(path.to_string().as_str());
    }

    #[func]
    fn reset_data(&mut self) {
        Data::reset();
    }
}
#[godot_api]
impl GodotExt for Client {
    fn init(base: Base<Node>) -> Self {
        godot_logger::GodotLogger::init();

        Data::reset();

        // TODO: Load configs from file.
        let client_config = Default::default();

        Self {
            bcs: Default::default(),
            client_config,
            base,
        }
    }

    fn ready(&mut self) {
        if self.base.has_method("has_method".into()) {
            log::debug!("true");
        }

        log::info!("Ready");

        self.bcs.push(ClientBattlescape::new(
            self.base.share(),
            Default::default(),
            &self.client_config,
            ClientId(0),
        ));
    }

    fn process(&mut self, delta: f64) {
        return; // TODO: process gets called always?
        for bc in self.bcs.iter_mut() {
            bc.update(delta as f32)
        }
    }
}
