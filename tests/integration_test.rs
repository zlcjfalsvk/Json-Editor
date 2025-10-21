/// Integration tests for the canvas editor
///
/// These tests verify the core functionality of the application.
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
