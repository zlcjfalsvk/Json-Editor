/// Desktop application entry point
///
/// This is the main entry point for the desktop version of the canvas editor.

use wgpu_canvas_editor::renderer::CanvasRenderer;
use wgpu_canvas_editor::State;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

fn main() {
    // Initialize logger
    env_logger::init();

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();

    // Run event loop
    event_loop.run_app(&mut app).unwrap();
}

/// Application structure implementing ApplicationHandler
struct App {
    window: Option<Window>,
    state: Option<State<'static>>,
    renderer: Option<CanvasRenderer>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("WGPU Canvas Editor")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

            let window = event_loop.create_window(window_attributes).unwrap();

            // SAFETY: The window must live as long as the state, which we ensure
            // by storing both in the same struct
            let state = pollster::block_on(unsafe {
                State::new(std::mem::transmute::<&Window, &'static Window>(&window))
            });
            let renderer = CanvasRenderer::new(&state.device, &state.config);

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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                log::info!("Escape pressed, closing");
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
