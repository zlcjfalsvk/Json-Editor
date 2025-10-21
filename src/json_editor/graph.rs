use crate::utils;
use egui::{Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use serde_json::Value;

/// A node in the JSON graph visualization
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Node identifier
    pub id: usize,
    /// Display label
    pub label: String,
    /// Node type (object, array, string, number, etc.)
    pub node_type: NodeType,
    /// Position in the visualization
    pub position: Pos2,
    /// Size of the node
    pub size: Vec2,
    /// JSON path to this node (e.g., ["items", "0", "value"])
    pub json_path: Vec<String>,
    /// Node content (for table-based rendering of Objects and Arrays)
    pub content: NodeContent,
}

/// Content of a node (for table-based display)
#[derive(Debug, Clone)]
pub enum NodeContent {
    /// Object with key-value pairs
    Object(Vec<KeyValuePair>),
    /// Array with indexed items
    Array(Vec<ArrayItem>),
    /// Primitive value (displayed inline)
    Primitive(String),
}

/// A key-value pair in an Object node
#[derive(Debug, Clone)]
pub struct KeyValuePair {
    /// Property key
    pub key: String,
    /// Value representation
    pub value_display: String,
    /// Type of the value
    pub value_type: NodeType,
    /// Whether this value is a reference to a child node (object/array)
    pub is_reference: bool,
}

/// An array item
#[derive(Debug, Clone)]
pub struct ArrayItem {
    /// Array index
    pub index: usize,
    /// Value representation
    pub value_display: String,
    /// Type of the value
    pub value_type: NodeType,
    /// Whether this value is a reference to a child node (object/array)
    pub is_reference: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}

impl NodeType {
    fn color(&self) -> Color32 {
        match self {
            NodeType::Object => Color32::from_rgb(100, 150, 200),
            NodeType::Array => Color32::from_rgb(150, 100, 200),
            NodeType::String => Color32::from_rgb(100, 200, 100),
            NodeType::Number => Color32::from_rgb(200, 150, 100),
            NodeType::Boolean => Color32::from_rgb(200, 100, 150),
            NodeType::Null => Color32::from_rgb(150, 150, 150),
        }
    }
}

/// An edge connecting two nodes
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: usize,
    pub to: usize,
    pub label: Option<String>,
}

/// JSON Graph visualization
pub struct JsonGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    next_id: usize,
    /// Zoom level
    zoom: f32,
    /// Pan offset
    offset: Vec2,
    /// Whether the graph is being dragged
    dragging: bool,
    /// Selected node ID
    selected_node: Option<usize>,
}

impl Default for JsonGraph {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 0,
            zoom: 1.0,
            offset: Vec2::ZERO,
            dragging: false,
            selected_node: None,
        }
    }
}

impl JsonGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build graph from JSON value
    pub fn build_from_json(&mut self, value: &Value) {
        self.nodes.clear();
        self.edges.clear();
        self.next_id = 0;
        self.selected_node = None;

        if value.is_null() {
            return;
        }

        self.build_node(value, None, None, 0, 0.0, Vec::new());
        self.log_to_console(&format!("Built graph with {} nodes", self.nodes.len()));
    }

    /// Recursively build nodes from JSON value
    /// Returns the width used by this subtree
    fn build_node(
        &mut self,
        value: &Value,
        parent_id: Option<usize>,
        edge_label: Option<String>,
        depth: usize,
        x_offset: f32,
        json_path: Vec<String>,
    ) -> f32 {
        let node_id = self.next_id;
        self.next_id += 1;

        // Build node content and determine type
        let (label, node_type, content) = match value {
            Value::Object(map) => {
                let label = format!("Object ({})", map.len());
                let mut pairs = Vec::new();

                for (key, val) in map {
                    let (value_display, value_type, is_reference) = match val {
                        Value::Object(m) => (format!("{{ {} }}", m.len()), NodeType::Object, true),
                        Value::Array(a) => (format!("[ {} ]", a.len()), NodeType::Array, true),
                        Value::String(s) => {
                            let display = if s.len() > 30 {
                                format!("\"{}...\"", &s[..30])
                            } else {
                                format!("\"{}\"", s)
                            };
                            (display, NodeType::String, false)
                        }
                        Value::Number(n) => (n.to_string(), NodeType::Number, false),
                        Value::Bool(b) => (b.to_string(), NodeType::Boolean, false),
                        Value::Null => ("null".to_string(), NodeType::Null, false),
                    };

                    pairs.push(KeyValuePair {
                        key: key.clone(),
                        value_display,
                        value_type,
                        is_reference,
                    });
                }

                (label, NodeType::Object, NodeContent::Object(pairs))
            }
            Value::Array(arr) => {
                let label = format!("Array [{}]", arr.len());
                let mut items = Vec::new();

                for (index, val) in arr.iter().enumerate() {
                    let (value_display, value_type, is_reference) = match val {
                        Value::Object(m) => (format!("{{ {} }}", m.len()), NodeType::Object, true),
                        Value::Array(a) => (format!("[ {} ]", a.len()), NodeType::Array, true),
                        Value::String(s) => {
                            let display = if s.len() > 30 {
                                format!("\"{}...\"", &s[..30])
                            } else {
                                format!("\"{}\"", s)
                            };
                            (display, NodeType::String, false)
                        }
                        Value::Number(n) => (n.to_string(), NodeType::Number, false),
                        Value::Bool(b) => (b.to_string(), NodeType::Boolean, false),
                        Value::Null => ("null".to_string(), NodeType::Null, false),
                    };

                    items.push(ArrayItem {
                        index,
                        value_display,
                        value_type,
                        is_reference,
                    });
                }

                (label, NodeType::Array, NodeContent::Array(items))
            }
            Value::String(s) => {
                let display = if s.len() > 20 {
                    format!("\"{}...\"", &s[..20])
                } else {
                    format!("\"{}\"", s)
                };
                (display.clone(), NodeType::String, NodeContent::Primitive(display))
            }
            Value::Number(n) => {
                let display = n.to_string();
                (display.clone(), NodeType::Number, NodeContent::Primitive(display))
            }
            Value::Bool(b) => {
                let display = b.to_string();
                (display.clone(), NodeType::Boolean, NodeContent::Primitive(display))
            }
            Value::Null => {
                ("null".to_string(), NodeType::Null, NodeContent::Primitive("null".to_string()))
            }
        };

        // Calculate position based on depth and offset
        let x = 50.0 + x_offset;
        let y = 50.0 + depth as f32 * 100.0; // Increased vertical spacing for table nodes

        // Calculate node size based on content
        let size = self.calculate_node_size(&content);

        let node = GraphNode {
            id: node_id,
            label,
            node_type,
            position: Pos2::new(x, y),
            size,
            json_path: json_path.clone(),
            content,
        };

        self.nodes.push(node);

        // Create edge from parent
        if let Some(parent) = parent_id {
            self.edges.push(GraphEdge {
                from: parent,
                to: node_id,
                label: edge_label,
            });
        }

        // Process children and calculate total width
        let mut child_offset = x_offset;
        let mut total_width = 0.0;

        match value {
            Value::Object(map) => {
                for (key, child_value) in map {
                    let mut child_path = json_path.clone();
                    child_path.push(key.clone());
                    let child_width = self.build_node(
                        child_value,
                        Some(node_id),
                        Some(key.clone()),
                        depth + 1,
                        child_offset,
                        child_path,
                    );
                    child_offset += child_width;
                    total_width += child_width;
                }
            }
            Value::Array(arr) => {
                for (idx, child_value) in arr.iter().enumerate() {
                    let mut child_path = json_path.clone();
                    child_path.push(idx.to_string());
                    let child_width = self.build_node(
                        child_value,
                        Some(node_id),
                        Some(format!("[{}]", idx)),
                        depth + 1,
                        child_offset,
                        child_path,
                    );
                    child_offset += child_width;
                    total_width += child_width;
                }
            }
            _ => {}
        }

        // Return the width used by this subtree
        // If no children, return a base width; otherwise return children's total width
        if total_width > 0.0 {
            total_width
        } else {
            150.0 // Base width for leaf nodes
        }
    }

    /// Calculate node size based on content
    fn calculate_node_size(&self, content: &NodeContent) -> Vec2 {
        match content {
            NodeContent::Object(pairs) => {
                // Width: enough for key + value columns
                let width = 250.0;
                // Height: header + rows (20px per row) + padding
                let row_height = 22.0;
                let header_height = 25.0;
                let padding = 10.0;
                let max_visible_rows = 10; // Limit height for very large objects
                let visible_rows = pairs.len().min(max_visible_rows);
                let height = header_height + (visible_rows as f32 * row_height) + padding;
                Vec2::new(width, height.max(60.0))
            }
            NodeContent::Array(items) => {
                // Similar to Object but with index column
                let width = 250.0;
                let row_height = 22.0;
                let header_height = 25.0;
                let padding = 10.0;
                let max_visible_rows = 10;
                let visible_rows = items.len().min(max_visible_rows);
                let height = header_height + (visible_rows as f32 * row_height) + padding;
                Vec2::new(width, height.max(60.0))
            }
            NodeContent::Primitive(_) => {
                // Small fixed size for primitive values
                Vec2::new(120.0, 40.0)
            }
        }
    }

    /// Get the selected node's JSON path
    pub fn get_selected_path(&self) -> Option<Vec<String>> {
        self.selected_node
            .and_then(|id| self.nodes.iter().find(|n| n.id == id))
            .map(|node| node.json_path.clone())
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_node = None;
    }

    /// Select a node by its JSON path
    /// Returns true if a matching node was found and selected
    pub fn select_by_path(&mut self, path: &[String]) -> bool {
        // Find node with matching path
        for node in &self.nodes {
            if node.json_path == path {
                self.selected_node = Some(node.id);
                self.log_to_console(&format!(
                    "Selected node by path: {} (path: {:?})",
                    node.label, node.json_path
                ));
                return true;
            }
        }

        // No exact match found - try to find the closest match
        let mut best_match: Option<&GraphNode> = None;
        let mut best_match_len = 0;

        for node in &self.nodes {
            // Count how many path segments match
            let match_len = node
                .json_path
                .iter()
                .zip(path.iter())
                .take_while(|(a, b)| a == b)
                .count();

            if match_len > 0 && match_len > best_match_len {
                best_match = Some(node);
                best_match_len = match_len;
            }
        }

        if let Some(node) = best_match {
            self.selected_node = Some(node.id);
            self.log_to_console(&format!(
                "Selected closest match: {} (path: {:?}, matched {} segments)",
                node.label, node.json_path, best_match_len
            ));
            true
        } else {
            false
        }
    }

    /// Render node content (table for Object/Array, text for primitives)
    fn render_node_content(
        &self,
        painter: &egui::Painter,
        node: &GraphNode,
        rect: Rect,
        zoom: f32,
    ) {
        let font_size = (11.0 * zoom).max(8.0);
        let header_font_size = (12.0 * zoom).max(9.0);

        match &node.content {
            NodeContent::Object(pairs) => {
                // Draw header with label
                let header_height = 25.0 * zoom;
                let header_rect =
                    Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));

                painter.text(
                    header_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &node.label,
                    egui::FontId::proportional(header_font_size),
                    Color32::WHITE,
                );

                // Draw header separator
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x, rect.min.y + header_height),
                        Pos2::new(rect.max.x, rect.min.y + header_height),
                    ],
                    Stroke::new(1.0 * zoom, Color32::from_gray(200)),
                );

                // Draw table rows
                let row_height = 22.0 * zoom;
                let key_column_width = rect.width() * 0.4;
                let max_visible_rows = 10;

                for (i, pair) in pairs.iter().enumerate().take(max_visible_rows) {
                    let y = rect.min.y + header_height + (i as f32 * row_height);

                    // Draw horizontal separator
                    if i > 0 {
                        painter.line_segment(
                            [Pos2::new(rect.min.x + 5.0, y), Pos2::new(rect.max.x - 5.0, y)],
                            Stroke::new(0.5 * zoom, Color32::from_gray(180)),
                        );
                    }

                    // Draw vertical separator between columns
                    painter.line_segment(
                        [
                            Pos2::new(rect.min.x + key_column_width, y),
                            Pos2::new(rect.min.x + key_column_width, y + row_height),
                        ],
                        Stroke::new(0.5 * zoom, Color32::from_gray(180)),
                    );

                    // Draw key (left column)
                    let key_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + 5.0, y),
                        Vec2::new(key_column_width - 10.0, row_height),
                    );
                    painter.text(
                        Pos2::new(key_rect.min.x, key_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &pair.key,
                        egui::FontId::monospace(font_size),
                        Color32::from_gray(240),
                    );

                    // Draw value (right column) with type-specific color
                    let value_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + key_column_width + 5.0, y),
                        Vec2::new(rect.width() - key_column_width - 10.0, row_height),
                    );
                    let value_color = if pair.is_reference {
                        Color32::from_rgb(150, 200, 255) // Light blue for references
                    } else {
                        pair.value_type.color()
                    };
                    painter.text(
                        Pos2::new(value_rect.min.x, value_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &pair.value_display,
                        egui::FontId::monospace(font_size),
                        value_color,
                    );
                }

                // Show "..." if there are more rows
                if pairs.len() > max_visible_rows {
                    let y = rect.min.y + header_height + (max_visible_rows as f32 * row_height);
                    painter.text(
                        Pos2::new(rect.center().x, y),
                        egui::Align2::CENTER_CENTER,
                        &format!("... {} more", pairs.len() - max_visible_rows),
                        egui::FontId::proportional(font_size),
                        Color32::from_gray(200),
                    );
                }
            }
            NodeContent::Array(items) => {
                // Draw header with label
                let header_height = 25.0 * zoom;
                let header_rect =
                    Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));

                painter.text(
                    header_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &node.label,
                    egui::FontId::proportional(header_font_size),
                    Color32::WHITE,
                );

                // Draw header separator
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x, rect.min.y + header_height),
                        Pos2::new(rect.max.x, rect.min.y + header_height),
                    ],
                    Stroke::new(1.0 * zoom, Color32::from_gray(200)),
                );

                // Draw table rows
                let row_height = 22.0 * zoom;
                let index_column_width = 40.0 * zoom;
                let max_visible_rows = 10;

                for (i, item) in items.iter().enumerate().take(max_visible_rows) {
                    let y = rect.min.y + header_height + (i as f32 * row_height);

                    // Draw horizontal separator
                    if i > 0 {
                        painter.line_segment(
                            [Pos2::new(rect.min.x + 5.0, y), Pos2::new(rect.max.x - 5.0, y)],
                            Stroke::new(0.5 * zoom, Color32::from_gray(180)),
                        );
                    }

                    // Draw vertical separator between columns
                    painter.line_segment(
                        [
                            Pos2::new(rect.min.x + index_column_width, y),
                            Pos2::new(rect.min.x + index_column_width, y + row_height),
                        ],
                        Stroke::new(0.5 * zoom, Color32::from_gray(180)),
                    );

                    // Draw index (left column)
                    let index_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + 5.0, y),
                        Vec2::new(index_column_width - 10.0, row_height),
                    );
                    painter.text(
                        Pos2::new(index_rect.center().x, index_rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        &format!("[{}]", item.index),
                        egui::FontId::monospace(font_size),
                        Color32::from_gray(200),
                    );

                    // Draw value (right column) with type-specific color
                    let value_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + index_column_width + 5.0, y),
                        Vec2::new(rect.width() - index_column_width - 10.0, row_height),
                    );
                    let value_color = if item.is_reference {
                        Color32::from_rgb(150, 200, 255) // Light blue for references
                    } else {
                        item.value_type.color()
                    };
                    painter.text(
                        Pos2::new(value_rect.min.x, value_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &item.value_display,
                        egui::FontId::monospace(font_size),
                        value_color,
                    );
                }

                // Show "..." if there are more rows
                if items.len() > max_visible_rows {
                    let y = rect.min.y + header_height + (max_visible_rows as f32 * row_height);
                    painter.text(
                        Pos2::new(rect.center().x, y),
                        egui::Align2::CENTER_CENTER,
                        &format!("... {} more", items.len() - max_visible_rows),
                        egui::FontId::proportional(font_size),
                        Color32::from_gray(200),
                    );
                }
            }
            NodeContent::Primitive(value) => {
                // Simple text rendering for primitive values
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    value,
                    egui::FontId::proportional((12.0 * zoom).max(9.0)),
                    Color32::WHITE,
                );
            }
        }
    }

    /// Render the graph using egui
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut selection_changed = false;

        ui.heading("JSON Graph Visualization");

        // Controls
        ui.horizontal(|ui| {
            ui.label(format!("Nodes: {}", self.nodes.len()));
            ui.separator();

            if ui.button("Reset View").clicked() {
                self.zoom = 1.0;
                self.offset = Vec2::ZERO;
                self.log_to_console("Reset view");
            }

            if ui.button("Clear Selection").clicked() {
                self.clear_selection();
                selection_changed = true;
                self.log_to_console("Selection cleared");
            }

            ui.separator();
            ui.label(format!("Zoom: {:.2}x", self.zoom));
        });

        ui.separator();

        // Canvas
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height().max(400.0)),
            egui::Sense::click_and_drag(),
        );

        // Handle panning
        if response.dragged() {
            self.offset += response.drag_delta();
            self.dragging = true;
            self.log_to_console("Panning graph");
        } else {
            self.dragging = false;
        }

        // Handle zoom with scroll
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom *= 1.0 + scroll_delta * 0.001;
                self.zoom = self.zoom.clamp(0.1, 5.0);
                self.log_to_console(&format!("Zoom: {:.2}x", self.zoom));
            }
        }

        let canvas_rect = response.rect;

        // Draw edges
        for edge in &self.edges {
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.iter().find(|n| n.id == edge.from),
                self.nodes.iter().find(|n| n.id == edge.to),
            ) {
                let from_pos = self.transform_pos(
                    from_node.position + Vec2::new(from_node.size.x / 2.0, from_node.size.y),
                    canvas_rect,
                );
                let to_pos = self.transform_pos(
                    to_node.position + Vec2::new(to_node.size.x / 2.0, 0.0),
                    canvas_rect,
                );

                painter.line_segment(
                    [from_pos, to_pos],
                    Stroke::new(2.0 * self.zoom, Color32::GRAY),
                );

                // Draw edge label
                if let Some(label) = &edge.label {
                    let mid_pos =
                        Pos2::new((from_pos.x + to_pos.x) / 2.0, (from_pos.y + to_pos.y) / 2.0);
                    painter.text(
                        mid_pos,
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(10.0 * self.zoom),
                        Color32::DARK_GRAY,
                    );
                }
            }
        }

        // Draw nodes and handle clicks
        for node in &self.nodes {
            let pos = self.transform_pos(node.position, canvas_rect);
            let size = node.size * self.zoom;

            let rect = Rect::from_min_size(pos, size);

            // Check if node is clicked
            if response.clicked()
                && let Some(click_pos) = response.interact_pointer_pos()
                && rect.contains(click_pos)
            {
                self.selected_node = Some(node.id);
                selection_changed = true;
                self.log_to_console(&format!(
                    "Selected node: {} (path: {:?})",
                    node.label, node.json_path
                ));
            }

            // Check if this node is selected
            let is_selected = self.selected_node == Some(node.id);

            // Node background (highlight if selected)
            let bg_color = if is_selected {
                // Brighter version for selected node
                let base = node.node_type.color();
                Color32::from_rgb(
                    base.r().saturating_add(50),
                    base.g().saturating_add(50),
                    base.b().saturating_add(50),
                )
            } else {
                node.node_type.color()
            };

            painter.rect_filled(rect, 5.0, bg_color);
            painter.rect_stroke(
                rect,
                5.0,
                Stroke::new(
                    if is_selected { 3.0 } else { 2.0 },
                    if is_selected {
                        Color32::YELLOW
                    } else {
                        Color32::BLACK
                    },
                ),
                StrokeKind::Outside,
            );

            // Render node content based on type
            self.render_node_content(&painter, node, rect, self.zoom);
        }

        // Instructions
        if self.nodes.is_empty() {
            painter.text(
                canvas_rect.center(),
                egui::Align2::CENTER_CENTER,
                "No valid JSON to visualize",
                egui::FontId::proportional(20.0),
                Color32::GRAY,
            );
        }

        selection_changed
    }

    /// Transform position with zoom and offset
    fn transform_pos(&self, pos: Pos2, canvas_rect: Rect) -> Pos2 {
        let transformed = pos.to_vec2() * self.zoom + self.offset;
        canvas_rect.min + transformed
    }

    /// Log message to browser console (WASM) or stdout (desktop)
    fn log_to_console(&self, message: &str) {
        utils::log("JSON Graph", message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_graph() {
        let graph = JsonGraph::new();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_build_simple_object() {
        let mut graph = JsonGraph::new();
        let json = json!({"key": "value"});
        graph.build_from_json(&json);

        assert!(!graph.nodes.is_empty());
        assert_eq!(graph.edges.len(), 1); // One edge from object to value
    }

    #[test]
    fn test_build_array() {
        let mut graph = JsonGraph::new();
        let json = json!([1, 2, 3]);
        graph.build_from_json(&json);

        assert_eq!(graph.nodes.len(), 4); // Array node + 3 number nodes
        assert_eq!(graph.edges.len(), 3); // 3 edges from array to numbers
    }

    #[test]
    fn test_build_nested() {
        let mut graph = JsonGraph::new();
        let json = json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });
        graph.build_from_json(&json);

        assert!(graph.nodes.len() >= 4); // Root object + user object + name + age
    }

    #[test]
    fn test_node_type_colors() {
        assert_ne!(NodeType::Object.color(), NodeType::Array.color());
        assert_ne!(NodeType::String.color(), NodeType::Number.color());
    }

    #[test]
    fn test_build_default_json() {
        let mut graph = JsonGraph::new();
        let json = json!({
            "name": "example",
            "version": "1.0.0",
            "items": [
                {"id": 1, "value": "first"},
                {"id": 2, "value": "second"}
            ]
        });
        graph.build_from_json(&json);

        println!("\nAll nodes created:");
        for node in &graph.nodes {
            println!(
                "  Node {}: {} at pos ({}, {})",
                node.id, node.label, node.position.x, node.position.y
            );
        }

        println!("\nAll edges:");
        for edge in &graph.edges {
            let from_label = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.from)
                .map(|n| n.label.as_str())
                .unwrap_or("?");
            let to_label = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.to)
                .map(|n| n.label.as_str())
                .unwrap_or("?");
            println!(
                "  Edge: {} -> {} (label: {:?})",
                from_label, to_label, edge.label
            );
        }

        // Expected: 10 nodes
        // 0: Object (3) - root
        // 1: "example" - name
        // 2: "1.0.0" - version
        // 3: Array [2] - items
        // 4: Object (2) - items[0]
        // 5: 1 - items[0].id
        // 6: "first" - items[0].value
        // 7: Object (2) - items[1]
        // 8: 2 - items[1].id
        // 9: "second" - items[1].value

        assert_eq!(
            graph.nodes.len(),
            10,
            "Expected 10 nodes for default_json structure"
        );

        // Check that "first" value node exists
        let has_first = graph.nodes.iter().any(|n| n.label.contains("first"));
        assert!(has_first, "Should have a node with 'first' value");

        // Check that "second" value node exists
        let has_second = graph.nodes.iter().any(|n| n.label.contains("second"));
        assert!(has_second, "Should have a node with 'second' value");
    }
}
