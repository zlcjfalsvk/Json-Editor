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
use wasm_bindgen::JsCast;
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

            // Append canvas to document first
            let canvas_element = web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wgpu-canvas")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(canvas)
                })
                .expect("Failed to append canvas to document");

            // Get actual canvas size from DOM (after append)
            let canvas_html = canvas_element
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("Canvas element is not an HtmlCanvasElement");

            let client_width = canvas_html.client_width() as u32;
            let client_height = canvas_html.client_height() as u32;

            // Ensure minimum size
            let width = client_width.max(1);
            let height = client_height.max(1);

            log::info!("Canvas client size: {}x{}", client_width, client_height);
            log::info!("Using canvas size: {}x{}", width, height);

            // Set canvas pixel dimensions explicitly
            canvas_html.set_width(width);
            canvas_html.set_height(height);

            // Request winit window to match
            let _ = window.request_inner_size(PhysicalSize::new(width, height));

            app_state.window = Some(window);

            // Get window reference for async initialization
            let window_ptr = app_state.window.as_ref().unwrap() as *const Window;
            let state_clone = self.state.clone();

            drop(app_state); // Release borrow before spawning

            // Capture the canvas size for async initialization
            let canvas_width = width;
            let canvas_height = height;

            // Initialize wgpu asynchronously
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("Initializing wgpu...");

                // SAFETY: The window is stored in WasmAppState which is in an Rc,
                // so it won't be moved or dropped while we're using it
                let window_ref = unsafe { &*(window_ptr) };
                let window_static =
                    unsafe { std::mem::transmute::<&Window, &'static Window>(window_ref) };

                // Log actual window size before State initialization
                let actual_size = window_static.inner_size();
                log::info!(
                    "Actual window size before State init: {}x{}",
                    actual_size.width,
                    actual_size.height
                );

                let mut state = State::new(window_static).await;

                // Explicitly set the surface size to match canvas
                log::info!(
                    "Resizing surface to canvas size: {}x{}",
                    canvas_width,
                    canvas_height
                );
                state.resize(PhysicalSize::new(canvas_width, canvas_height));

                log::info!(
                    "WGPU initialized successfully! Surface size: {}x{}",
                    state.config.width,
                    state.config.height
                );

                // Update the shared state
                let mut app_state = state_clone.borrow_mut();
                app_state.state = Some(state);
                app_state.initializing = false;
                drop(app_state); // Release borrow before triggering redraws

                // Trigger initial redraw now that State is ready
                window_static.request_redraw();
                log::info!("Initial redraw requested");

                // Schedule additional redraws to ensure rendering starts
                // This handles timing issues with async initialization
                let window_for_retry = window_static as *const Window;
                wasm_bindgen_futures::spawn_local(async move {
                    // Wait a bit for the event loop to process
                    gloo_timers::future::TimeoutFuture::new(50).await;
                    let window = unsafe { &*window_for_retry };
                    window.request_redraw();
                    log::debug!("Retry redraw requested (50ms delay)");

                    gloo_timers::future::TimeoutFuture::new(100).await;
                    window.request_redraw();
                    log::debug!("Retry redraw requested (150ms total delay)");
                });
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
            log::debug!(
                "window_event received but state not initialized yet: {:?}",
                event
            );
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
