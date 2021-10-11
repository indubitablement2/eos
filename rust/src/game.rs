use crate::constants::*;
use crate::ecs::*;
use gdnative::api::*;
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    name: String,
    ecs: Option<Ecs>,
    mod_order: Vec<String>,
    game_def: Option<GameDef>,
    sprite_atlas: Option<Ref<TextureArray, Unique>>,
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
            mod_order: Vec::new(),
            game_def: None,
            sprite_atlas: None,
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

    #[export]
    unsafe fn _exit_tree(&mut self, owner: &Node2D) {
        self.save_world(owner);

        if let Some(ecs) = &self.ecs {
            // Free the rids we created.
            let visual_server = gdnative::api::VisualServer::godot_singleton();
            let render_res = ecs
                .world
                .get_resource_unchecked_mut::<crate::ecs_resources::RenderRes>()
                .unwrap();
            visual_server.free_rid(render_res.multimesh_rid);
            visual_server.free_rid(render_res.mesh_rid);
        }
    }

    #[export]
    unsafe fn _process(&mut self, _owner: &Node2D, delta: f32) {
        if let Some(ecs) = &mut self.ecs {
            ecs.update(delta);
        }
    }

    #[export]
    unsafe fn _draw(&mut self, _owner: &Node2D) {}

    /// Load a world.
    #[export]
    unsafe fn load_world(&mut self, owner: &Node2D, world_name: String) {
        self.name = world_name;
        let world_path: String = format!("{}{}/", WORLDS_PATH, self.name);

        // Load mod order.
        let mod_order = load_mod_order(&world_path);

        // TODO: Load GameDef or create a new one.
        let game_def = GameDef::new();

        // Load atlas texture or create a new one.
        let sprite_atlas = load_sprite_atlas(&world_path);

        // Create Ecs.
        self.ecs = Some(Ecs::new(owner.get_canvas_item(), sprite_atlas.get_rid()));

        self.mod_order = mod_order;
        self.game_def = Some(game_def);
        self.sprite_atlas = Some(sprite_atlas);
    }

    /// Save this world.
    #[export]
    unsafe fn save_world(&mut self, _owner: &Node2D) {
        if !self.name.is_empty() {
        } else {
            godot_warn!("Can not save unnamed world.");
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameDef {}
impl GameDef {
    fn new() -> Self {
        GameDef {}
    }
}

/// Load sprite atlas order.
fn load_mod_order(world_path: &String) -> Vec<String> {
    let mut mod_order = Vec::new();

    let file = File::new();

    let mod_order_path = format!("{}mod_order", &world_path);

    if file.open(&mod_order_path, File::READ).is_ok() {
        let mut line = file.get_line().to_string();
        while !line.is_empty() {
            mod_order.push(line);
            line = file.get_line().to_string();
        }
    } else {
        godot_error!("Could not open {}.", mod_order_path);
    }

    file.close();

    mod_order
}

/// Load sprite atlas texture or create a new one.
fn load_sprite_atlas(world_path: &String) -> Ref<TextureArray, Unique> {
    let mut atlas_order = Vec::new();

    let file = File::new();

    let atlas_order_path = format!("{}atlas", world_path);

    // Load sprite atlas order.
    if file.open(&atlas_order_path, File::READ).is_ok() {
        let mut line = file.get_line();
        while !line.is_empty() {
            atlas_order.push(line);
            line = file.get_line();
        }
    } else {
        godot_error!("Could not open {}.", atlas_order_path);
    }

    file.close();

    if atlas_order.is_empty() {
        return create_new_sprite_atlas();
    }

    let mut sprite_atlas = TextureArray::new();
    sprite_atlas.create(
        SPRITE_ATLAS_SIZE,
        SPRITE_ATLAS_SIZE,
        atlas_order.len().try_into().unwrap(),
        Image::FORMAT_DXT5, // TODO: Check if compression is good.
        0,
    );

    // Load sprite atlas.
    let img = Image::new().into_shared();
    for (i, path) in atlas_order.into_iter().enumerate() {
        unsafe {
            if img.assume_safe().load(path).is_ok() {
                if let Err(err) = img
                    .assume_safe()
                    .compress(Image::COMPRESS_S3TC, Image::COMPRESS_SOURCE_GENERIC, 0.7) // TODO: Check if compression is good.
                {
                    godot_warn!("Error while compressing image: {:?}.", err);
                    sprite_atlas = create_new_sprite_atlas();
                    break;
                }
                sprite_atlas.set_layer_data(img.assume_safe(), i.try_into().unwrap());
            } else {
                godot_warn!("Can not load an image from sprite atlas.");
                sprite_atlas = create_new_sprite_atlas();
                break;
            }
        }
    }

    sprite_atlas
}

fn create_new_sprite_atlas() -> Ref<TextureArray, Unique> {
    todo!()
}
