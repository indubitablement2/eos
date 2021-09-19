use crate::battlescape_schedue::*;
use crate::battlescape_components::*;
use std::convert::TryInto;
use gdnative::api::*;
use gdnative::prelude::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_builder)]
pub struct GodotEcs {
    ecs_ready: bool,
    ecs_paused: bool,
    ecs_world: EcsWorld,
    render_res: RenderRes,
}

#[methods]
impl GodotEcs {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        GodotEcs {
            ecs_ready: false,
            ecs_paused: false,
            ecs_world: EcsWorld::new(),
            render_res: RenderRes::default(),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node) {}

    #[export]
    unsafe fn _process(&mut self, _owner: &Node, _delta: f32) {
        if !self.ecs_paused && self.ecs_ready {
            self.ecs_world.run();
        }
    }

    #[export]
    unsafe fn spawn_ship(&mut self, _owner: &Node, position: Vector2, rotation: f32) {
        self.ecs_world.add_ship(position, rotation);
    }

    /// Draw this node into provided canvas.
    #[export]
    unsafe fn draw(&mut self, _owner: &Node, _enable: bool) {
        if !self.render_res.ready {
            godot_warn!("Can not draw before calling init_renderer on this node.");
            return;
        }

        let visual_server = gdnative::api::VisualServer::godot_singleton();

        let body_set = self.ecs_world.world.get_resource::<crate::battlescape_resources::BodySetRes>().unwrap();
        // let mut query_physic = self.ecs_world.world.query::<(&Renderable, &PhysicBodyHandle)>();
        // let mut query_pos = self.ecs_world.world.query::<(&Renderable, &Position)>();
        // query_physic.iter(&self.ecs_world.world).for_each(|(_renderable, physic_body_handle)| {
            // body_set.0.get(physic_body_handle.0).unwrap().position().to_matrix();
        // });

        // let e = self.ecs_world.world.get(*query.iter().last().unwrap().0);

        // visual_server.multimesh_set_as_bulk_array(self.render_res.multimesh_rid, )
    }

    /// Prepare to draw sprite.
    #[export]
    unsafe fn init_renderer(
        &mut self,
        _owner: &Node,
        canvas_rid: Rid,
        texture_rid: Rid,
        normal_texture_rid: Rid,
        multimesh_rid: Rid,
        multimesh_allocate: i32,
    ) {
        self.render_res = RenderRes {
            ready: true,
            canvas_rid,
            multimesh_rid,
            multimesh_allocate,
            texture_rid,
            normal_texture_rid,
        };

        // TODO: Add ship textures.
        // TODO: Add normal map.
    }
}

/// All that is needed to render sprites.
struct RenderRes {
    pub ready: bool,
    pub canvas_rid: Rid,
    pub multimesh_rid: Rid,
    pub multimesh_allocate: i32,
    pub texture_rid: Rid,
    pub normal_texture_rid: Rid,
}

impl Default for RenderRes {
    fn default() -> Self {
        Self {
            ready: false,
            canvas_rid: Rid::new(),
            multimesh_rid: Rid::new(),
            multimesh_allocate: 100,
            texture_rid: Rid::new(),
            normal_texture_rid: Rid::new(),
            
        }
    }
}
