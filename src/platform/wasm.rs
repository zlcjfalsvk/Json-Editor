/// WASM platform implementation
#[cfg(target_arch = "wasm32")]
use crate::platform::common;
#[cfg(target_arch = "wasm32")]
use crate::state::State;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::ActiveEventLoop,
    platform::web::WindowExtWebSys,
    window::{Window, WindowId},
};

/// WASM Application inner state
#[cfg(target_arch = "wasm32")]
pub struct WasmAppState {
    pub window: Option<Window>,
    pub state: Option<State<'static>>,
    pub initializing: bool,
}

/// WASM Application structure with shared state
#[cfg(target_arch = "wasm32")]
pub struct WasmApp {
    state: Rc<RefCell<WasmAppState>>,
}

#[cfg(target_arch = "wasm32")]
impl WasmApp {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(WasmAppState {
                window: None,
                state: None,
                initializing: false,
            })),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl ApplicationHandler for WasmApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut app_state = self.state.borrow_mut();

        if app_state.window.is_none() && !app_state.initializing {
            app_state.initializing = true;
            log::info!("Creating window...");

            let window_attributes = Window::default_attributes().with_title("WGPU Canvas Editor");

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

            app_state.window = Some(window);

            // Get window reference for async initialization
            let window_ptr = app_state.window.as_ref().unwrap() as *const Window;
            let state_clone = self.state.clone();

            drop(app_state); // Release borrow before spawning

            // Initialize wgpu asynchronously
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("Initializing wgpu...");

                // SAFETY: The window is stored in WasmAppState which is in an Rc,
                // so it won't be moved or dropped while we're using it
                let window_ref = unsafe { &*(window_ptr) };
                let window_static =
                    unsafe { std::mem::transmute::<&Window, &'static Window>(window_ref) };

                let state = State::new(window_static).await;

                log::info!("WGPU initialized successfully!");

                // Update the shared state
                let mut app_state = state_clone.borrow_mut();
                app_state.state = Some(state);
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut app_state = self.state.borrow_mut();

        // Skip if not initialized yet
        if app_state.state.is_none() {
            return;
        }

        let state = app_state.state.as_mut().unwrap();

        // Handle common events
        common::handle_window_event(state, event_loop, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let app_state = self.state.borrow();
        if let Some(window) = &app_state.window {
            window.request_redraw();
        }
    }
}
