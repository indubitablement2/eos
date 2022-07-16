mod client_configs;
mod input;
mod metascape_manager;
mod time_manager;

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct State {
    client_config: client_configs::ClientConfig,
    /// When the program was started.
    start_instant: std::time::Instant,

    window: winit::window::Window,
    window_size: PhysicalSize<u32>,
    scale_factor: f32,

    egui_context: egui::Context,
    egui: egui_winit::State,
    egui_rpass: egui_wgpu::renderer::RenderPass,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,

    /// Async Runtime.
    rt: tokio::runtime::Runtime,
}
impl State {
    // Creating some of the wgpu types requires async code
    fn new() -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();

        // Create the window.
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(64, 64))
            .with_title("Eos")
            .with_transparent(true)
            .with_decorations(false)
            .with_inner_size(PhysicalSize::new(128, 128))
            .build(&event_loop)
            .unwrap();
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;

        let client_config = client_configs::ClientConfig::load();

        // Create the async runtime.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // The instance is a handle to our GPU.
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = rt.block_on(async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap()
        });

        let (device, queue) = rt.block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None, // Trace path
                )
                .await
                .unwrap()
        });

        let surface_format = surface.get_supported_formats(&adapter)[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: match client_config.graphic_config.vsync {
                true => wgpu::PresentMode::AutoVsync,
                false => wgpu::PresentMode::AutoNoVsync,
            },
        };
        surface.configure(&device, &surface_config);

        let egui_context = egui::Context::default();
        let mut egui = egui_winit::State::new(&event_loop);
        egui.set_max_texture_side(adapter.limits().max_texture_dimension_2d as usize);
        egui.set_pixels_per_point(window.scale_factor() as f32);
        let egui_rpass = egui_wgpu::renderer::RenderPass::new(&device, surface_format, 1);

        let state = Self {
            client_config,
            surface,
            device,
            queue,
            surface_config,
            window_size,
            scale_factor,
            rt,
            window,
            egui_context,
            egui,
            egui_rpass,
            start_instant: std::time::Instant::now(),
        };

        (state, event_loop)
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
        // See: https://github.com/rust-windowing/winit/issues/208
        // This solves an issue where the app would panic when minimizing on Windows.
        if new_size.width == 0 || new_size.height == 0 {
            log::debug!("Ignored resize with zero size.");
            return;
        }

        self.window_size = new_size;
        self.scale_factor = self.window.scale_factor() as f32;

        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);

        self.egui.set_pixels_per_point(self.scale_factor);

        log::info!(
            "Resized window to {:?} with scale factor of {}.",
            new_size,
            self.scale_factor
        );
    }

    fn input(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::Resized(new_size) => self.resize(new_size),
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Destroyed => {}
            WindowEvent::ReceivedCharacter(_) => {}
            WindowEvent::Focused(_) => {}
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {}
            WindowEvent::ModifiersChanged(_) => {}
            WindowEvent::CursorMoved {
                device_id,
                position,
                modifiers: _,
            } => {}
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                modifiers: _,
            } => {}
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers: _,
            } => {}
            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                new_inner_size,
            } => self.resize(*new_inner_size),
            _ => {}
        }
    }

    fn update(&mut self) {
        // Begin to draw the UI frame.
        self.egui_context
            .begin_frame(self.egui.take_egui_input(&self.window));

        egui::CentralPanel::default().show(&self.egui_context, |ui| {
            ui.label("Hello egui!");
        });
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output_frame = self.surface.get_current_texture()?;
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let ui_output = self.egui_context.end_frame();

        // Handle ui textures add.
        let tdelta = ui_output.textures_delta;
        for (id, image_delta) in tdelta.set.iter() {
            self.egui_rpass
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        // Handle ui buffer changes.
        let paint_jobs = self.egui_context.tessellate(ui_output.shapes);
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.window_size.width, self.window_size.height],
            pixels_per_point: self.scale_factor,
        };
        self.egui_rpass
            .update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Draw the ui.
        self.egui_rpass.execute(
            &mut encoder,
            &output_view,
            &paint_jobs,
            &screen_descriptor,
            None,
        );

        // Submit the commands & draw.
        self.queue.submit(std::iter::once(encoder.finish()));
        output_frame.present();

        // Handle ui textures remove.
        for id in tdelta.free.iter() {
            self.egui_rpass.free_texture(id);
        }

        Ok(())
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_module_path(true)
        .parse_default_env()
        .init();

    let (mut state, event_loop) = State::new();

    event_loop.run(move |event, _, mut control_flow| match event {
        Event::NewEvents(_) => {}
        Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
            if !state.egui.on_event(&state.egui_context, &event) {
                state.input(event, &mut control_flow);
            }
        }
        Event::MainEventsCleared => {
            state.update();
            state.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            if let Err(err) = state.render() {
                match err {
                    wgpu::SurfaceError::Timeout => todo!(),
                    wgpu::SurfaceError::Outdated => todo!(),
                    wgpu::SurfaceError::Lost => todo!(),
                    wgpu::SurfaceError::OutOfMemory => {
                        log::error!("Out of memory.");
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
        }
        Event::RedrawEventsCleared => {}
        Event::LoopDestroyed => {}
        _ => {}
    });
}
