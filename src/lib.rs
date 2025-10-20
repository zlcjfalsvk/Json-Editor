/// Library and WASM entry point
///
/// This module contains the common library code and WASM exports for the web version.
pub mod app;
pub mod input;
pub mod json_editor;
pub mod renderer;
pub mod state;

pub use app::App;
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
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

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
pub fn run() -> Result<(), JsValue> {
    log::info!("Initializing web application...");

    let event_loop = EventLoop::new()
        .map_err(|e| JsValue::from_str(&format!("Failed to create event loop: {}", e)))?;

    let app = WasmApp::new();

    // Run event loop - spawns in browser's event loop
    use winit::platform::web::EventLoopExtWebSys;
    event_loop.spawn_app(app);

    Ok(())
}

/// WASM Application inner state
#[cfg(target_arch = "wasm32")]
struct WasmAppState {
    window: Option<Window>,
    state: Option<State<'static>>,
    initializing: bool,
}

/// WASM Application structure with shared state
#[cfg(target_arch = "wasm32")]
struct WasmApp {
    state: Rc<RefCell<WasmAppState>>,
}

#[cfg(target_arch = "wasm32")]
impl WasmApp {
    fn new() -> Self {
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

        // Let egui handle the event first
        let handled = state.handle_event(&event);

        // If egui didn't handle it, process it ourselves
        if !handled {
            match event {
                WindowEvent::CloseRequested => {
                    log::info!("Close requested");
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
        let app_state = self.state.borrow();
        if let Some(window) = &app_state.window {
            window.request_redraw();
        }
    }
}
