/// Desktop application entry point
///
/// This is the main entry point for the desktop version of the canvas editor.

use wgpu_canvas_editor::renderer::CanvasRenderer;
use wgpu_canvas_editor::State;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

fn main() {
    // Initialize logger
    env_logger::init();

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("WGPU Canvas Editor")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    // Create state and renderer
    let mut state = pollster::block_on(State::new(&window));
    let mut renderer = CanvasRenderer::new(&state.device, &state.config);

    // Run event loop
    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => {
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
                // Request redraw
                state.window().request_redraw();
            }
            _ => {}
        })
        .unwrap();
}

/// Render a frame
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

    // Use the canvas renderer
    renderer.render(&mut encoder, &view);

    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
