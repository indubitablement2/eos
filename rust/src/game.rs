use crate::ecs::*;
use crate::render_pipeline::*;
use gdnative::api::*;
use gdnative::prelude::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    ecs: Ecs,
    render_pipeline: RenderPipeline,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(owner: &Node2D) -> Self {
        Game {
            ecs: Ecs::new(),
            render_pipeline: RenderPipeline::new(owner),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {}

    /// Free the rid we created.
    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {
        let visual_server = gdnative::api::VisualServer::godot_singleton();
        visual_server.free_rid(self.render_pipeline.multimesh_rid);
        visual_server.free_rid(self.render_pipeline.mesh_rid);
    }

    #[export]
    unsafe fn _process(&mut self, _owner: &Node2D, delta: f32) {
        let update_result = self.ecs.update(delta);
        self.render_pipeline.maybe_render_res = Some(update_result.render_res);
        self.render_pipeline.render();

        // Render
    }

    #[export]
    unsafe fn _draw(&mut self, _owner: &Node2D) {}

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

        let t3 = t2.clone();
        godot_print!("t1: {:?}", &t1);
        godot_print!("t2: {:?}", &t2);
        godot_print!("t3 clone t2: {:?}", &t3);
    }
}
