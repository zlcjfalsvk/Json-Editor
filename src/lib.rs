/// Library and WASM entry point
///
/// This module contains the common library code and WASM exports for the web version.
pub mod json_editor;
pub mod platform;
pub mod state;
pub mod ui;
pub mod utils;

pub use state::State;
pub use ui::App;

// WASM-specific code
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;

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

    let app = platform::WasmApp::new();

    // Run event loop - spawns in browser's event loop
    use winit::platform::web::EventLoopExtWebSys;
    event_loop.spawn_app(app);

    Ok(())
}
