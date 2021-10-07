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
        self.ecs.update(delta);
        // start battlescape
        // let num_render = self.render_res.multimesh_allocate;
        // self.battlescapes.values_mut().for_each(|bc| {
        //     let update_request = UpdateRequest {
        //         send_render: Some(num_render),
        //         spawn_ship: Option::None,
        //     };
        //     if bc.update(update_request).is_err() {
        //         // TODO: Remove battlescape.
        //         godot_error!("Can not send to Battlescape. It probably crashed.");
        //     }
        // });

        // todo: wait for result
    }

    #[export]
    unsafe fn _draw(&mut self, _owner: &Node2D) {
        if let Some((render_data, num_instances)) = self.render_pipeline.render_data.take() {
            let visual_server = gdnative::api::VisualServer::godot_singleton();
            visual_server.multimesh_set_as_bulk_array(self.render_pipeline.multimesh_rid, render_data);
            visual_server.multimesh_set_visible_instances(self.render_pipeline.multimesh_rid, num_instances);
            visual_server.canvas_item_add_multimesh(
                self.render_pipeline.canvas_rid,
                self.render_pipeline.multimesh_rid,
                self.render_pipeline.texture_rid,
                self.render_pipeline.texture_rid,
            );
        }
    }
}
