/// Common event handling logic shared between desktop and WASM platforms
use crate::state::State;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;

/// Handle window events common to both platforms
///
/// Returns true if the application should continue running
pub fn handle_window_event(
    state: &mut State,
    event_loop: &ActiveEventLoop,
    event: WindowEvent,
) -> bool {
    // Let egui handle the event first
    let handled = state.handle_event(&event);

    // If egui didn't handle it, process it ourselves
    if !handled {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
                return false;
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
                return false;
            }
            Err(e) => {
                log::error!("Render error: {:?}", e);
            }
        }
    }

    true
}
