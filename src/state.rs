/// Application state management
///
/// This module contains the core state management for the canvas editor.
/// It maintains the application state and handles updates.
use crate::ui::App;
use wgpu;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

/// Main application state
pub struct State<'a> {
    /// wgpu surface for rendering
    pub surface: wgpu::Surface<'a>,
    /// Graphics device
    pub device: wgpu::Device,
    /// Command queue
    pub queue: wgpu::Queue,
    /// Surface configuration
    pub config: wgpu::SurfaceConfiguration,
    /// Window size
    pub size: PhysicalSize<u32>,
    /// Reference to the window
    pub window: &'a Window,
    /// egui context
    pub egui_ctx: egui::Context,
    /// egui-winit state
    pub egui_state: egui_winit::State,
    /// egui-wgpu renderer
    pub egui_renderer: egui_wgpu::Renderer,
    /// Application UI
    pub app: App,
}

impl<'a> State<'a> {
    /// Create a new state instance
    ///
    /// # Arguments
    ///
    /// * `window` - Reference to the window
    ///
    /// # Returns
    ///
    /// A new State instance
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // Create wgpu instance
        #[cfg(target_arch = "wasm32")]
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window).unwrap();

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Request device and queue
        // WASM: Use downlevel_defaults for WebGPU compatibility
        #[cfg(target_arch = "wasm32")]
        let required_limits = wgpu::Limits::downlevel_defaults();

        #[cfg(not(target_arch = "wasm32"))]
        let required_limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits,
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: Default::default(),
                trace: Default::default(),
            })
            .await
            .unwrap();

        // Get surface capabilities
        let surface_caps = surface.get_capabilities(&adapter);

        // Select surface format
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // Configure surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Initialize egui
        let egui_ctx = egui::Context::default();

        // Get viewport info from window
        let viewport_id = egui::ViewportId::ROOT;
        let pixels_per_point = window.scale_factor() as f32;

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            viewport_id,
            window,
            Some(pixels_per_point),
            None,       // theme
            Some(2048), // max_texture_side
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );

        // Initialize application
        let app = App::new();

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            egui_ctx,
            egui_state,
            egui_renderer,
            app,
        }
    }

    /// Get a reference to the window
    pub fn window(&self) -> &Window {
        self.window
    }

    /// Resize the surface
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new size for the surface
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Handle window event
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_state.on_window_event(self.window, event).consumed
    }

    /// Update state
    pub fn update(&mut self) {
        // Placeholder for future animation/physics logic
        // UI updates happen in render() via egui context
    }

    /// Render the current frame with egui
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Prepare egui
        let raw_input = self.egui_state.take_egui_input(self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.app.update(ctx);
        });

        self.egui_state
            .handle_platform_output(self.window, full_output.platform_output);

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        // Upload egui texture
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        // Update egui buffers
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        // Render
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render egui
            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &tris,
                &screen_descriptor,
            );
        }

        // Cleanup egui textures
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
