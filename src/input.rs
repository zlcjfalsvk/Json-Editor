/// Input event handling
///
/// This module handles user input events such as keyboard and mouse interactions.

use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Input handler state
pub struct InputHandler {
    /// Whether the left mouse button is pressed
    pub mouse_left_pressed: bool,
    /// Whether the right mouse button is pressed
    pub mouse_right_pressed: bool,
    /// Current mouse position (x, y)
    pub mouse_position: (f64, f64),
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            mouse_left_pressed: false,
            mouse_right_pressed: false,
            mouse_position: (0.0, 0.0),
        }
    }

    /// Handle keyboard input
    ///
    /// # Arguments
    ///
    /// * `event` - The keyboard event
    ///
    /// # Returns
    ///
    /// true if the event was handled, false otherwise
    pub fn handle_keyboard(&mut self, event: &KeyEvent) -> bool {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Escape) => {
                if event.state == ElementState::Pressed {
                    log::info!("Escape key pressed");
                    return true;
                }
            }
            PhysicalKey::Code(KeyCode::Space) => {
                if event.state == ElementState::Pressed {
                    log::info!("Space key pressed");
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Handle mouse button input
    ///
    /// # Arguments
    ///
    /// * `button` - The mouse button
    /// * `state` - The button state (pressed/released)
    ///
    /// # Returns
    ///
    /// true if the event was handled, false otherwise
    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) -> bool {
        match button {
            MouseButton::Left => {
                self.mouse_left_pressed = state == ElementState::Pressed;
                log::debug!(
                    "Left mouse button: {}",
                    if self.mouse_left_pressed {
                        "pressed"
                    } else {
                        "released"
                    }
                );
                true
            }
            MouseButton::Right => {
                self.mouse_right_pressed = state == ElementState::Pressed;
                log::debug!(
                    "Right mouse button: {}",
                    if self.mouse_right_pressed {
                        "pressed"
                    } else {
                        "released"
                    }
                );
                true
            }
            _ => false,
        }
    }

    /// Handle mouse movement
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    pub fn handle_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_position = (x, y);
        log::trace!("Mouse moved to: ({}, {})", x, y);
    }

    /// Handle mouse wheel/scroll
    ///
    /// # Arguments
    ///
    /// * `delta` - The scroll delta
    ///
    /// # Returns
    ///
    /// true if the event was handled, false otherwise
    pub fn handle_mouse_wheel(&mut self, delta: f32) -> bool {
        log::debug!("Mouse wheel delta: {}", delta);
        // Future: Implement zoom functionality
        true
    }
}
