use egui::{Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};

use super::graph::GraphNode;

/// Minimap for graph visualization
/// Displays a small overview of the entire graph in the bottom-right corner
pub struct Minimap {
    /// Size of the minimap
    size: Vec2,
    /// Whether the minimap is visible
    visible: bool,
    /// Background opacity (0.0 = transparent, 1.0 = opaque)
    background_opacity: f32,
}

impl Default for Minimap {
    fn default() -> Self {
        Self {
            size: Vec2::new(200.0, 150.0),
            visible: true,
            background_opacity: 0.8,
        }
    }
}

impl Minimap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle minimap visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set minimap visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if minimap is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Render the minimap
    /// Returns Some(new_offset) if the user clicked on the minimap to navigate
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        canvas_rect: Rect,
        nodes: &[GraphNode],
        current_zoom: f32,
        current_offset: Vec2,
    ) -> Option<Vec2> {
        if !self.visible || nodes.is_empty() {
            return None;
        }

        // Calculate minimap position (bottom-right corner with padding)
        let padding = 10.0;
        let minimap_pos = Pos2::new(
            canvas_rect.max.x - self.size.x - padding,
            canvas_rect.max.y - self.size.y - padding,
        );
        let minimap_rect = Rect::from_min_size(minimap_pos, self.size);

        // Calculate bounds of all nodes to determine the scale
        let (min_bounds, max_bounds) = self.calculate_graph_bounds(nodes);
        let graph_size = max_bounds - min_bounds;

        // Prevent division by zero
        if graph_size.x <= 0.0 || graph_size.y <= 0.0 {
            return None;
        }

        // Calculate scale to fit entire graph in minimap
        let scale_x = (self.size.x - 20.0) / graph_size.x; // 20px padding
        let scale_y = (self.size.y - 20.0) / graph_size.y;
        let scale = scale_x.min(scale_y).min(0.5); // Cap at 0.5 for readability

        // Draw minimap background
        let bg_color =
            Color32::from_rgba_unmultiplied(30, 30, 30, (255.0 * self.background_opacity) as u8);
        painter.rect_filled(minimap_rect, 3.0, bg_color);
        painter.rect_stroke(
            minimap_rect,
            3.0,
            Stroke::new(1.5, Color32::from_gray(100)),
            StrokeKind::Outside,
        );

        // Draw simplified nodes in minimap
        // Content padding area (10px on all sides)
        let content_rect = Rect::from_min_max(
            minimap_rect.min + Vec2::new(10.0, 10.0),
            minimap_rect.max - Vec2::new(10.0, 10.0),
        );

        for node in nodes {
            let node_pos_in_minimap =
                self.world_to_minimap(node.position, minimap_rect, min_bounds, scale);
            let node_size_in_minimap = node.size * scale;

            // Draw node as a small rectangle
            let node_rect = Rect::from_min_size(node_pos_in_minimap, node_size_in_minimap);

            // Only draw if node is within minimap content bounds
            if content_rect.intersects(node_rect) {
                // Clamp node rect to content bounds
                let clamped_node = self.clamp_rect_to_bounds(node_rect, content_rect);
                painter.rect_filled(
                    clamped_node,
                    1.0,
                    Color32::from_rgba_unmultiplied(100, 150, 200, 180),
                );
            }
        }

        // Draw viewport rectangle showing current view
        let viewport_rect = self.calculate_viewport_in_minimap(
            canvas_rect,
            minimap_rect,
            min_bounds,
            scale,
            current_zoom,
            current_offset,
        );

        // Clamp viewport rect to minimap content bounds (not the full minimap rect)
        let clamped_viewport = self.clamp_rect_to_bounds(viewport_rect, content_rect);

        // Draw viewport with semi-transparent fill for better visibility
        painter.rect_filled(
            clamped_viewport,
            0.0,
            Color32::from_rgba_unmultiplied(255, 200, 0, 30),
        );
        painter.rect_stroke(
            clamped_viewport,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(255, 200, 0)),
            StrokeKind::Outside,
        );

        // Handle minimap interaction
        let response = ui.interact(minimap_rect, ui.id().with("minimap"), egui::Sense::click());

        if response.clicked()
            && let Some(click_pos) = response.interact_pointer_pos()
        {
            // Convert click position to world coordinates
            let new_offset = self.minimap_to_world_offset(
                click_pos,
                minimap_rect,
                canvas_rect,
                min_bounds,
                scale,
                current_zoom,
            );
            return Some(new_offset);
        }

        None
    }

    /// Calculate the bounding box of all nodes
    fn calculate_graph_bounds(&self, nodes: &[GraphNode]) -> (Vec2, Vec2) {
        if nodes.is_empty() {
            return (Vec2::ZERO, Vec2::new(100.0, 100.0));
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in nodes {
            min_x = min_x.min(node.position.x);
            min_y = min_y.min(node.position.y);
            max_x = max_x.max(node.position.x + node.size.x);
            max_y = max_y.max(node.position.y + node.size.y);
        }

        // Add some padding
        let padding = 50.0;
        (
            Vec2::new(min_x - padding, min_y - padding),
            Vec2::new(max_x + padding, max_y + padding),
        )
    }

    /// Convert world coordinates to minimap coordinates
    fn world_to_minimap(
        &self,
        world_pos: Pos2,
        minimap_rect: Rect,
        min_bounds: Vec2,
        scale: f32,
    ) -> Pos2 {
        let relative_pos = world_pos.to_vec2() - min_bounds;
        let scaled_pos = relative_pos * scale;
        minimap_rect.min + scaled_pos + Vec2::new(10.0, 10.0) // 10px padding
    }

    /// Calculate the viewport rectangle in minimap coordinates
    /// This shows which part of the graph is currently visible on screen
    fn calculate_viewport_in_minimap(
        &self,
        canvas_rect: Rect,
        minimap_rect: Rect,
        min_bounds: Vec2,
        scale: f32,
        current_zoom: f32,
        current_offset: Vec2,
    ) -> Rect {
        // Calculate the world space coordinates of the visible viewport
        // When zoomed in (zoom > 1), viewport size is smaller in world space
        // When zoomed out (zoom < 1), viewport size is larger in world space
        let viewport_size_in_world = canvas_rect.size() / current_zoom;

        // Calculate the top-left corner of the viewport in world space
        // The transform is: screen = world * zoom + offset
        // So: world = (screen - offset) / zoom
        // For the canvas top-left (screen = 0,0 relative to canvas):
        let viewport_top_left_in_world = -current_offset / current_zoom;

        // Convert world coordinates to minimap coordinates
        let viewport_min = self.world_to_minimap(
            Pos2::ZERO + viewport_top_left_in_world,
            minimap_rect,
            min_bounds,
            scale,
        );
        let viewport_size_in_minimap = viewport_size_in_world * scale;

        Rect::from_min_size(viewport_min, viewport_size_in_minimap)
    }

    /// Convert minimap click position to world offset
    fn minimap_to_world_offset(
        &self,
        click_pos: Pos2,
        minimap_rect: Rect,
        canvas_rect: Rect,
        min_bounds: Vec2,
        scale: f32,
        current_zoom: f32,
    ) -> Vec2 {
        // Convert click position to relative position in minimap
        let relative_pos = click_pos - minimap_rect.min - Vec2::new(10.0, 10.0);

        // Convert to world coordinates
        let world_pos = (relative_pos / scale) + min_bounds;

        // Calculate offset to center the clicked position in the viewport
        let viewport_center_in_world = canvas_rect.size() / (2.0 * current_zoom);
        let target_world_pos = world_pos - viewport_center_in_world;

        // Convert to offset (inverted)
        -target_world_pos * current_zoom
    }

    /// Clamp a rectangle to stay within bounds
    fn clamp_rect_to_bounds(&self, rect: Rect, bounds: Rect) -> Rect {
        let min_x = rect.min.x.max(bounds.min.x);
        let min_y = rect.min.y.max(bounds.min.y);
        let max_x = rect.max.x.min(bounds.max.x);
        let max_y = rect.max.y.min(bounds.max.y);

        // Ensure we have a valid rectangle
        if min_x >= max_x || min_y >= max_y {
            // If clamping results in invalid rect, return a small rect at center
            Rect::from_center_size(bounds.center(), Vec2::new(1.0, 1.0))
        } else {
            Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
        }
    }
}
