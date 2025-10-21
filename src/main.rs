use wgpu_canvas_editor::platform::DesktopApp;
use winit::event_loop::EventLoop;

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
