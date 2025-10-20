/// Application state management
///
/// This module contains the core state management for the canvas editor.
/// It maintains the application state and handles updates.
use wgpu;
use winit::dpi::PhysicalSize;
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

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
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

    /// Update state (placeholder for future logic)
    pub fn update(&mut self) {
        // Future: Update animation, physics, etc.
    }

    /// Render the current frame
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

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
