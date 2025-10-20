/// Integration tests for the canvas editor
///
/// These tests verify the core functionality of the application.
#[cfg(test)]
mod tests {
    use wgpu_canvas_editor::input::InputHandler;
    use winit::event::{ElementState, MouseButton};

    #[test]
    fn test_input_handler_creation() {
        let handler = InputHandler::new();
        assert!(!handler.mouse_left_pressed);
        assert!(!handler.mouse_right_pressed);
        assert_eq!(handler.mouse_position, (0.0, 0.0));
    }

    #[test]
    fn test_mouse_button_handling() {
        let mut handler = InputHandler::new();

        // Test left button press
        let handled = handler.handle_mouse_button(MouseButton::Left, ElementState::Pressed);
        assert!(handled);
        assert!(handler.mouse_left_pressed);

        // Test left button release
        let handled = handler.handle_mouse_button(MouseButton::Left, ElementState::Released);
        assert!(handled);
        assert!(!handler.mouse_left_pressed);

        // Test right button
        let handled = handler.handle_mouse_button(MouseButton::Right, ElementState::Pressed);
        assert!(handled);
        assert!(handler.mouse_right_pressed);
    }

    #[test]
    fn test_mouse_move() {
        let mut handler = InputHandler::new();

        handler.handle_mouse_move(100.0, 200.0);
        assert_eq!(handler.mouse_position, (100.0, 200.0));

        handler.handle_mouse_move(150.5, 250.5);
        assert_eq!(handler.mouse_position, (150.5, 250.5));
    }

    #[test]
    fn test_mouse_wheel() {
        let mut handler = InputHandler::new();

        let handled = handler.handle_mouse_wheel(1.0);
        assert!(handled);

        let handled = handler.handle_mouse_wheel(-1.0);
        assert!(handled);
    }
}

#[cfg(test)]
mod renderer_tests {
    // Note: Renderer tests require a GPU and are better suited for manual testing
    // or headless CI environments with GPU support.

    #[test]
    fn test_vertex_size() {
        use wgpu_canvas_editor::renderer::canvas::Vertex;

        // Verify vertex size for GPU buffer alignment
        let size = std::mem::size_of::<Vertex>();
        assert_eq!(size, 24); // 3 floats (position) + 3 floats (color) = 24 bytes
    }

    #[test]
    fn test_vertex_creation() {
        use wgpu_canvas_editor::renderer::canvas::Vertex;

        let vertex = Vertex {
            position: [0.0, 1.0, 0.0],
            color: [1.0, 0.0, 0.0],
        };

        assert_eq!(vertex.position, [0.0, 1.0, 0.0]);
        assert_eq!(vertex.color, [1.0, 0.0, 0.0]);
    }
}
