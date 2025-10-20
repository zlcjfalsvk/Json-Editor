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
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    platform::web::WindowExtWebSys,
    window::WindowBuilder,
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
    let window = WindowBuilder::new()
        .with_title("WGPU Canvas Editor")
        .build(&event_loop)
        .unwrap();

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

    // Create state and renderer
    let mut state = State::new(&window).await;
    let renderer = CanvasRenderer::new(&state.device, &state.config);

    log::info!("Application initialized, starting event loop");

    // Run event loop
    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Close requested");
                        control_flow.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        log::info!("Resized to: {:?}", physical_size);
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        state.update();

                        match render(&mut state, &renderer) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                log::warn!("Surface lost, resizing");
                                state.resize(state.size);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("Out of memory");
                                control_flow.exit();
                            }
                            Err(e) => {
                                log::error!("Render error: {:?}", e);
                            }
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    state.window().request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
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
