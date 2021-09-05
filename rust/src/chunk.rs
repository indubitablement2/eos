use std::convert::TryInto;

use gdnative::api::*;
use gdnative::prelude::*;

use crate::ecs_schedue::*;

pub struct ChunkInfo {
    pub width: i64,
    pub height: i64,
    // texture: Ref<ImageTexture, thread_access::Shared>,
}
impl ChunkInfo {
    pub fn new() -> ChunkInfo {
        ChunkInfo {
            width: 0,
            height: 0,
            // texture: ImageTexture::new().into_shared(),
        }
    }
}

#[derive(NativeClass)]
#[inherit(Sprite)]
#[register_with(Self::register_builder)]
pub struct Chunk {
    is_ready: bool,
    pub run_sender: crossbeam_channel::Sender<Run>,
    pub post_update_receiver: crossbeam_channel::Receiver<PostUpdate>,
    update_pending: i32,
    pub chunk_info: ChunkInfo,
}

// Only __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl Chunk {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {
    }

    /// The "constructor" of the class.
    fn new(_owner: &Sprite) -> Self {
        Chunk {
            run_sender: crossbeam_channel::unbounded::<Run>().0,
            is_ready: false,
            update_pending: 0,
            post_update_receiver: crossbeam_channel::unbounded::<PostUpdate>().1,
            chunk_info: ChunkInfo::new(),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Sprite) {
        // owner.set_texture(self.chunk_info.texture);
        godot_print!("Ready!");
    }

    /// Simulate 1 tick in parallel.
    #[export]
    unsafe fn update(&mut self, _owner: &Sprite) {
        if !self.is_ready {
            godot_error!("Can not update. Chunk is not ready.");
            return;
        }

        let run = Run{
            query_terrain_on_update: true,
            force_query_terrain: false,
            query_pawn: true,
        };

        self.update_pending += 1;
        if self.run_sender.try_send(run).is_err() {
            godot_error!("Can not update. Shedule probably panicked.")
        }
    }

    /// Init chunk by generating a new one.
    #[export]
    unsafe fn init_generate_chunk(
        &mut self, 
        _owner: &Sprite, 
        width: i64,
        height: i64,
    ) {
        if self.is_ready {
            godot_error!("Can not generate chunk. Chunk already ready.");
            return;
        }

        if width < 1 || width > 1024 || height < 1 || height > 1024 {
            godot_error!("Invalid width or height to generate chunk.");
            return;
        }

        self.chunk_info.width = width;
        self.chunk_info.height = height;

        let (rs, pur) = init_generate(
            width.try_into().unwrap(), 
            height.try_into().unwrap()
        );
        self.post_update_receiver = pur;
        self.run_sender = rs;

        godot_print!("Generated chunk.");
        self.is_ready = true;
    }

    #[export]
    unsafe fn init_load_chunk(
        &mut self,
        _owner: &Sprite,
    ) {
        init_load();
        godot_error!("Not implemented.");
    }

    /// Render interpolated between previous and latest update.
    ///
    /// Assumed that one tick takes 100ms.
    #[export]
    unsafe fn _process(
        &mut self, 
        owner: &Sprite, 
        _delta: f32,
    ) {
        // * Try to gather PostUpdate without blocking.
        if self.update_pending < 1 {
            // No PostUpdate to fetch.
            return;
        }

        let result = self.post_update_receiver.try_recv();
        match result {
            Ok(post_update) => {
                // We got a fresh new PostUpdate.
                self.update_pending -= 1;

                godot_print!("Got update #{}", post_update.tick);

                // Draw terrain.
                if let Some(new_terrain) = post_update.terrain {
                    if let Some(tex) = owner.texture() {
                        if let Some(img_tex) = tex.cast::<ImageTexture>() {
                            draw_terrain(&self.chunk_info, &new_terrain, img_tex);
                        }
                    } else {
                        godot_warn!("Chunk's texture is not an ImageTexture.");
                    }
                }
            }
            Err(err) => {
                if err == crossbeam_channel::TryRecvError::Disconnected {
                    // TODO: Try to gracefully handle that.
                    godot_error!("Can not receive PostUpdate. Shedule probably panicked.");
                }
            }
        }
    }
}

fn draw_terrain(chunk_info: &ChunkInfo, new_terrain: &[u8], texture: Ref<ImageTexture>) {
    // Create image from data.
    let img = Image::new();
    img.create_from_data(
        chunk_info.width, 
        chunk_info.height, 
        false, 
        Image::FORMAT_R8, 
        new_terrain.to_variant().to_byte_array() // TODO: This is probably not the right way.
    );

    unsafe{texture.assume_safe().create_from_image(img, 0);} 
}
