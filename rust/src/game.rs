use crate::constants::*;
use crate::ecs::Ecs;
use crate::ecs_render_pipeline::RenderRes;
use crate::game_def::GameDef;
use gdnative::api::*;
use gdnative::prelude::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    name: String,
    ecs: Option<Ecs>,
    game_def: Option<GameDef>,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        Game {
            name: String::new(),
            ecs: None,
            game_def: None,
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
    unsafe fn _exit_tree(&mut self, owner: &Node2D) {
        self.save_world(owner);

        // Free the rids we created.
        if let Some(ecs) = &self.ecs {
            if let Some(render_res) = ecs.world.get_resource::<RenderRes>() {
                let visual_server = gdnative::api::VisualServer::godot_singleton();
                visual_server.free_rid(render_res.multimesh_rid);
                visual_server.free_rid(render_res.mesh_rid);
            }
        }
    }

    #[export]
    unsafe fn _process(&mut self, _owner: &Node2D, delta: f32) {
        if let Some(ecs) = &mut self.ecs {
            ecs.update(delta);
        }
    }

    // #[export]
    // unsafe fn _physic_process(&mut self, _owner: &Node2D, delta: f32) {
    // }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        if let Some(ecs) = &self.ecs {
            if let Some(render_res) = ecs.world.get_resource::<RenderRes>() {
                let visual_server = gdnative::api::VisualServer::godot_singleton();
                visual_server.canvas_item_add_multimesh(
                    owner.get_canvas(),
                    render_res.multimesh_rid,
                    render_res.texture_rid,
                    render_res.normal_texture_rid
                );
            }
        }
    }

    /// Load a world.
    #[export]
    unsafe fn load_world(&mut self, owner: &Node2D, world_name: String) {
        let world_path: String = format!("{}{}/", WORLDS_PATH, world_name);

        // Load GameDef or create a new one.
        // TODO: Add parameter in load_world function.
        let game_def = GameDef::load(&world_path, false, true);

        // Create Ecs.
        self.ecs = Some(Ecs::new(owner, &game_def));

        self.name = world_name;
        self.game_def = Some(game_def);

        owner.update();
    }

    /// Save this world.
    #[export]
    unsafe fn save_world(&mut self, _owner: &Node2D) {
        if !self.name.is_empty() {
            // TODO: Save world.
        } else {
            godot_warn!("Can not save unnamed world.");
        }
    }
}