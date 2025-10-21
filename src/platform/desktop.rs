/// Desktop platform implementation
use crate::platform::common;
use crate::state::State;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

/// Desktop application structure implementing ApplicationHandler
pub struct DesktopApp {
    window: Option<Window>,
    state: Option<State<'static>>,
}

impl DesktopApp {
    pub fn new() -> Self {
        Self {
            window: None,
            state: None,
        }
    }
}

impl Default for DesktopApp {
    fn default() -> Self {
        Self::new()
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

        // Handle desktop-specific events (Escape key)
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                },
            ..
        } = event
        {
            log::info!("Escape pressed, closing");
            event_loop.exit();
            return;
        }

        // Handle common events
        common::handle_window_event(state, event_loop, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
