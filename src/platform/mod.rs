/// Platform-specific implementations
///
/// This module contains platform-specific code for desktop and WASM targets.
pub mod common;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export platform-specific types
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::DesktopApp;

#[cfg(target_arch = "wasm32")]
pub use wasm::{WasmApp, WasmAppState};
