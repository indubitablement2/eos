use super::*;
use crate::{client_battlescape::ClientBattlescape, client_config::ClientConfig};
use data::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
struct Client {
    bcs: Vec<ClientBattlescape>,
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
}
#[godot_api]
impl GodotExt for Client {
    fn init(base: Base<Node>) -> Self {
        godot_logger::GodotLogger::init();

        // TODO: Load configs from file.
        let client_config = Default::default();

        Self {
            bcs: Default::default(),
            client_config,
            base,
        }
    }

    fn ready(&mut self) {
        Data::clear();

        self.bcs.push(ClientBattlescape::new(
            self.base.share(),
            Default::default(),
            &self.client_config,
            ClientId(0),
        ));
    }

    fn process(&mut self, delta: f64) {
        // for bc in self.bcs.iter_mut() {
        //     bc.update(delta as f32)
        // }
    }
}
