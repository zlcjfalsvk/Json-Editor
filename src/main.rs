use wgpu_canvas_editor::State;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

/// Desktop application entry point
///
/// This is the main entry point for the desktop version of the canvas editor.
fn main() {
    // Initialize logger
    env_logger::init();

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    let mut app = DesktopApp::new();

    // Run event loop
    event_loop.run_app(&mut app).unwrap();
}

/// Application structure implementing ApplicationHandler
struct DesktopApp {
    window: Option<Window>,
    state: Option<State<'static>>,
}

impl DesktopApp {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
        }
    }
}

impl ApplicationHandler for DesktopApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("WGPU Canvas Editor - JSON Visualizer")
                .with_inner_size(winit::dpi::LogicalSize::new(1200, 800));

            let window = event_loop.create_window(window_attributes).unwrap();

            // Store window first
            self.window = Some(window);

            // SAFETY: The window must live as long as the state, which we ensure
            // by storing both in the same struct
            let window_ref = self.window.as_ref().unwrap();
            let state = pollster::block_on(unsafe {
                State::new(std::mem::transmute::<&Window, &'static Window>(window_ref))
            });

            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Skip if not initialized yet
        if self.state.is_none() {
            return;
        }

        let state = self.state.as_mut().unwrap();

        // Let egui handle the event first
        let handled = state.handle_event(&event);

        // If egui didn't handle it, process it ourselves
        if !handled {
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
                _ => {}
            }
        }

        // Always handle redraw
        if let WindowEvent::RedrawRequested = event {
            state.update();

            match state.render() {
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
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
