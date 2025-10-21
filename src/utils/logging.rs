/// Logging utilities for cross-platform compatibility
///
/// Provides unified logging functions that work on both WASM and desktop platforms.
/// Log a message to the appropriate output (browser console for WASM, stdout for desktop)
///
/// # Arguments
///
/// * `module` - The module name (e.g., "App", "JSON Editor")
/// * `message` - The message to log
pub fn log(module: &str, message: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::console;
        console::log_1(&format!("[{}] {}", module, message).into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[{}] {}", module, message);
    }
}

/// Log an info message
#[allow(dead_code)]
pub fn info(module: &str, message: &str) {
    log::info!("[{}] {}", module, message);
}

/// Log a warning message
#[allow(dead_code)]
pub fn warn(module: &str, message: &str) {
    log::warn!("[{}] {}", module, message);
}

/// Log an error message
#[allow(dead_code)]
pub fn error(module: &str, message: &str) {
    log::error!("[{}] {}", module, message);
}

/// Log a debug message
#[allow(dead_code)]
pub fn debug(module: &str, message: &str) {
    log::debug!("[{}] {}", module, message);
}
