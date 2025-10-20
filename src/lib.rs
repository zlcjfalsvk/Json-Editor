/// Library and WASM entry point
///
/// This module contains the common library code and WASM exports for the web version.

pub mod input;
pub mod renderer;
pub mod state;

pub use state::State;

// WASM-specific code
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::web::WindowExtWebSys,
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use crate::renderer::CanvasRenderer;

/// Initialize the WASM application
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logger
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");

    log::info!("WASM application starting...");
}

/// Run the web application
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run() {
    log::info!("Initializing web application...");

    let event_loop = EventLoop::new().unwrap();
    let mut app = WasmApp::new();

    // Run event loop
    event_loop.run_app(&mut app).unwrap();
}

/// WASM Application structure
#[cfg(target_arch = "wasm32")]
struct WasmApp {
    window: Option<Window>,
    state: Option<State<'static>>,
    renderer: Option<CanvasRenderer>,
}

#[cfg(target_arch = "wasm32")]
impl WasmApp {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
            renderer: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ApplicationHandler for WasmApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("WGPU Canvas Editor");

            let window = event_loop.create_window(window_attributes).unwrap();

            // Append canvas to document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wgpu-canvas")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Failed to append canvas to document");

            // Set canvas size
            let _ = window.request_inner_size(PhysicalSize::new(800, 600));

            // SAFETY: The window must live as long as the state, which we ensure
            // by storing both in the same struct
            let state = pollster::block_on(unsafe {
                State::new(std::mem::transmute::<&Window, &'static Window>(&window))
            });
            let renderer = CanvasRenderer::new(&state.device, &state.config);

            log::info!("Application initialized");

            self.state = Some(state);
            self.renderer = Some(renderer);
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        let renderer = self.renderer.as_ref().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                log::info!("Resized to: {:?}", physical_size);
                state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                state.update();

                match render(state, renderer) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        log::warn!("Surface lost, resizing");
                        state.resize(state.size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory");
                        event_loop.exit();
                    }
                    Err(e) => {
                        log::error!("Render error: {:?}", e);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Render a frame (WASM)
#[cfg(target_arch = "wasm32")]
fn render(state: &mut State, renderer: &CanvasRenderer) -> Result<(), wgpu::SurfaceError> {
    let output = state.surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    renderer.render(&mut encoder, &view);

    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
