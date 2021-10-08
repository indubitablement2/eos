use crate::ecs::*;
use gdnative::api::*;
use gdnative::prelude::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    ecs: Option<Ecs>,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        Game {
            ecs: None,
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {}

    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {
        if let Some(ecs) = &self.ecs {
            // Free the rids we created.
            let visual_server = gdnative::api::VisualServer::godot_singleton();
            let render_res = ecs.world.get_resource_unchecked_mut::<crate::ecs_resources::RenderRes>().unwrap();
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

    #[export]
    unsafe fn init_ecs(&mut self, owner: &Node2D, texture_rid: Rid) {
        if self.ecs.is_none() {
            self.ecs = Some(Ecs::new(owner.get_canvas_item(), texture_rid))
        }
    }

    #[export]
    unsafe fn test(&mut self, _ownder: &Node2D) {
        println!("hello");
        let mut t1: TypedArray<f32> = TypedArray::default();
        let mut t2 = t1.clone();
        godot_print!("t1 empty: {:?}", &t1);
        godot_print!("t2 clone: {:?}", &t2);

        t1.push(123.4);
        godot_print!("t1 push: {:?}", &t1);
        godot_print!("t2: {:?}", &t2);

        t2.push(8.8);
        godot_print!("t1: {:?}", &t1);
        godot_print!("t2 push: {:?}", &t2);

        let mut t3 = t2.clone();
        godot_print!("t1: {:?}", &t1);
        godot_print!("t2: {:?}", &t2);
        godot_print!("t3 clone t2: {:?}", &t3);

        t3.set(0, 50.0);
        godot_print!("t1: {:?}", &t1);
        godot_print!("t2: {:?}", &t2);
        godot_print!("t3 set 50.0: {:?}", &t3);

        t2.resize(0);
        godot_print!("t1: {:?}", &t1);
        godot_print!("t2 resize 0: {:?}", &t2);
        godot_print!("t3: {:?}", &t3);
    }
}
