mod camera;
mod util;

extern crate nalgebra as na;

use camera::Camera;
use util::QUAD;
use wgpu::util::DeviceExt;
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

// Ships
// objects (turret, projectile)
// gpu particles (debris, ammo casing, spark)
// albedo + normal + roughness/metalness + ambiant occ

// smoke

// lights

// bloom

// bg
// game
// ui

struct MainState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    surface_size: winit::dpi::PhysicalSize<u32>,

    geometry_pipeline: wgpu::RenderPipeline,
    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    instance_vertex_buffer: wgpu::Buffer,

    camera: Camera,

    /// A small buffer of 4 floats representing 2 triangles strip making an unit sized centered quad.
    quad_buffer: wgpu::Buffer,

    albedo_g_buffer: wgpu::Buffer,
    normal_g_buffer: wgpu::Buffer,
}
impl MainState {
    async fn new(window: &Window) -> Self {
        let surface_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: surface_size.width,
            height: surface_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                    },
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                }],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &QUAD,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Vertex Buffer"),
            contents: &[0; 256 * 8],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &QUAD,
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create geometry buffers.

        let size = (surface_size.height * surface_size.width * 16) as u64;

        let albedo_g_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Albedo Geometry Buffer"),
            size,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let normal_g_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Normal Geometry Buffer"),
            size,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            config,
            surface_size,
            render_pipeline,
            vertex_buffer,
            instance_vertex_buffer,
            quad_buffer,
            albedo_g_buffer,
            normal_g_buffer,
            geometry_pipeline: todo!(),
            camera: Default::default(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.surface_size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        // Create geometry buffers.

        let size = (self.surface_size.height * self.surface_size.width * 16) as u64;

        self.albedo_g_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Albedo Geometry Buffer"),
            size,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.normal_g_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Normal Geometry Buffer"),
            size,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        // TODO: Draw ships.

        render_pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
        for ship in 0..10 {
            // tODO: Set the ship data.
            render_pass.set_vertex_buffer(1, self.quad_buffer.slice(..));

            render_pass.draw(0..4, 0..1);
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn suspend(&mut self) {}

    fn resume(&mut self) {}

    /// Emitted when the event loop is being shut down.
    fn terminated(&mut self) {}
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_min_inner_size(Some(winit::dpi::PhysicalSize::new(64, 64)));

    let mut state = pollster::block_on(MainState::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::NewEvents(_) => {}
        winit::event::Event::WindowEvent { window_id, event } if window_id == window.id() => {
            match event {
                winit::event::WindowEvent::Resized(new_size) => state.resize(new_size),
                winit::event::WindowEvent::ScaleFactorChanged {
                    scale_factor: _,
                    new_inner_size,
                } => state.resize(*new_inner_size),
                winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::Destroyed => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::ReceivedCharacter(_) => {}
                winit::event::WindowEvent::Focused(focus) => {}
                winit::event::WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {}
                winit::event::WindowEvent::ModifiersChanged(_) => {}
                winit::event::WindowEvent::CursorMoved {
                    device_id,
                    position,
                    modifiers,
                } => {}
                winit::event::WindowEvent::MouseWheel {
                    device_id,
                    delta,
                    phase,
                    modifiers,
                } => {}
                winit::event::WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    modifiers,
                } => {}
                _ => {}
            }
        }
        winit::event::Event::Suspended => state.suspend(),
        winit::event::Event::Resumed => state.resume(),
        winit::event::Event::MainEventsCleared => {
            state.update();
            window.request_redraw();
        }
        winit::event::Event::RedrawRequested(window_id) if window_id == window.id() => {
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => {
                    log::debug!("Render swap chain lost. Recreating...");
                    state.resize(state.surface_size);
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    log::error!(
                        "There is no more memory left to alocate a new frame for rendering."
                    );
                    *control_flow = ControlFlow::Exit;
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(err) => log::warn!("Error while rendering ({:?}). Ignoring...", &err),
            }
        }
        winit::event::Event::RedrawEventsCleared => {}
        winit::event::Event::LoopDestroyed => {
            state.terminated();
        }
        _ => {}
    })
}
