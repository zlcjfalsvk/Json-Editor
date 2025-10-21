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

/// Editing state for a cell in the graph
#[derive(Debug, Clone)]
pub struct EditingCell {
    /// Node ID being edited
    pub node_id: usize,
    /// Key or index being edited
    pub key: String,
    /// Current editing text
    pub text: String,
    /// Original value type (for validation)
    pub value_type: NodeType,
}

/// State for adding a new property/item
#[derive(Debug, Clone)]
pub struct AddingState {
    /// Node ID where we're adding
    pub node_id: usize,
    /// Whether this is an Object (true) or Array (false)
    pub is_object: bool,
    /// Key for new property (Object only)
    pub key: String,
    /// Value for new property/item
    pub value: String,
    /// Selected value type
    pub value_type: NodeType,
}

/// State for renaming a property key
#[derive(Debug, Clone)]
pub struct RenamingKey {
    /// Node ID being edited
    pub node_id: usize,
    /// Old key name
    pub old_key: String,
    /// New key name being edited
    pub new_key: String,
}

/// Context menu state
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// Node ID for the context menu
    pub node_id: usize,
    /// Key or index for the row (None for container-level menu)
    pub row_key: Option<String>,
    /// Whether this is an Object (true) or Array (false)
    pub is_object: bool,
    /// Whether the row has a primitive value (can be edited)
    pub is_primitive: bool,
    /// Value type (for edit action)
    pub value_type: Option<NodeType>,
    /// Position where the menu was opened (fixed position)
    pub position: Pos2,
}

/// Clicked action on a node
#[derive(Debug, Clone)]
pub enum ClickAction {
    /// Edit a cell value
    EditCell(String, NodeType),
    /// Delete a row
    DeleteRow(String),
    /// Add a new row
    AddRow,
    /// Rename a key (Object properties only)
    RenameKey(String),
}

/// Type of modification operation
#[derive(Debug, Clone)]
pub enum ModifyOperation {
    /// Update an existing value
    Update { new_value: String },
    /// Delete a property or array item
    Delete,
    /// Add a new property (for Objects) or item (for Arrays)
    Add { key: String, value: String },
    /// Rename a property key (Object properties only)
    Rename { old_key: String, new_key: String },
}

/// Result of a completed modification operation
#[derive(Debug, Clone)]
pub struct EditResult {
    /// JSON path to the modified location
    pub json_path: Vec<String>,
    /// The operation performed
    pub operation: ModifyOperation,
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
    /// Currently editing cell (if any)
    editing_cell: Option<EditingCell>,
    /// Currently adding a new property/item (if any)
    adding_state: Option<AddingState>,
    /// Currently renaming a key (if any)
    renaming_key: Option<RenamingKey>,
    /// Context menu state (if showing)
    context_menu: Option<ContextMenuState>,
    /// Pending edit result to be processed by App
    pending_edit: Option<EditResult>,
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
            editing_cell: None,
            adding_state: None,
            renaming_key: None,
            context_menu: None,
            pending_edit: None,
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
        self.editing_cell = None; // Cancel any ongoing edits
        self.adding_state = None; // Cancel any ongoing adds
        self.renaming_key = None; // Cancel any ongoing renames
        self.context_menu = None; // Clear any context menu
        self.pending_edit = None; // Clear any pending edits

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
        let x = 100.0 + x_offset; // Increased left margin
        let y = 50.0 + depth as f32 * 200.0; // Increased vertical spacing significantly

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
        // Only create child nodes for Object and Array values (not primitives)
        let mut child_offset = x_offset;
        let mut total_width = 0.0;

        match value {
            Value::Object(map) => {
                for (key, child_value) in map {
                    // Only create child nodes for Object and Array types
                    if child_value.is_object() || child_value.is_array() {
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
                    // Primitive values are already displayed in the table
                }
            }
            Value::Array(arr) => {
                for (idx, child_value) in arr.iter().enumerate() {
                    // Only create child nodes for Object and Array types
                    if child_value.is_object() || child_value.is_array() {
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
                    // Primitive values are already displayed in the table
                }
            }
            _ => {}
        }

        // Return the width used by this subtree
        // If no children, return a base width; otherwise return children's total width
        if total_width > 0.0 {
            total_width
        } else {
            300.0 // Base width for leaf nodes (increased for better spacing)
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

    /// Take and return the pending edit result (if any)
    /// This clears the pending edit after returning it
    pub fn take_pending_edit(&mut self) -> Option<EditResult> {
        self.pending_edit.take()
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

                    // Reserve space for delete button (20px from right)
                    let delete_button_size = 16.0 * zoom;
                    let delete_button_x = rect.max.x - delete_button_size - 5.0;

                    // Draw value (right column) with type-specific color
                    let value_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + key_column_width + 5.0, y),
                        Vec2::new(rect.width() - key_column_width - delete_button_size - 20.0, row_height),
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

                    // Draw delete button (X icon)
                    let delete_center = Pos2::new(delete_button_x + delete_button_size / 2.0, y + row_height / 2.0);

                    // Draw button background (light gray circle)
                    painter.circle_filled(delete_center, delete_button_size / 2.0, Color32::from_rgb(80, 80, 80));

                    // Draw X
                    let x_size = delete_button_size * 0.4;
                    painter.line_segment(
                        [
                            delete_center + Vec2::new(-x_size, -x_size),
                            delete_center + Vec2::new(x_size, x_size),
                        ],
                        Stroke::new(2.0 * zoom, Color32::WHITE),
                    );
                    painter.line_segment(
                        [
                            delete_center + Vec2::new(x_size, -x_size),
                            delete_center + Vec2::new(-x_size, x_size),
                        ],
                        Stroke::new(2.0 * zoom, Color32::WHITE),
                    );
                }

                // Show "..." if there are more rows
                let bottom_y = if pairs.len() > max_visible_rows {
                    let y = rect.min.y + header_height + (max_visible_rows as f32 * row_height);
                    painter.text(
                        Pos2::new(rect.center().x, y),
                        egui::Align2::CENTER_CENTER,
                        format!("... {} more", pairs.len() - max_visible_rows),
                        egui::FontId::proportional(font_size),
                        Color32::from_gray(200),
                    );
                    y + row_height
                } else {
                    rect.min.y + header_height + (pairs.len() as f32 * row_height)
                };

                // Draw "Add Property" button at the bottom
                let add_button_height = 20.0 * zoom;
                let add_button_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 5.0, bottom_y + 5.0),
                    Vec2::new(rect.width() - 10.0, add_button_height),
                );

                // Button background
                painter.rect_filled(
                    add_button_rect,
                    3.0,
                    Color32::from_rgb(60, 120, 80),
                );

                // Button text
                painter.text(
                    add_button_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "+ Add Property",
                    egui::FontId::proportional((10.0 * zoom).max(8.0)),
                    Color32::WHITE,
                );
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
                        format!("[{}]", item.index),
                        egui::FontId::monospace(font_size),
                        Color32::from_gray(200),
                    );

                    // Reserve space for delete button
                    let delete_button_size = 16.0 * zoom;
                    let delete_button_x = rect.max.x - delete_button_size - 5.0;

                    // Draw value (right column) with type-specific color
                    let value_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + index_column_width + 5.0, y),
                        Vec2::new(rect.width() - index_column_width - delete_button_size - 20.0, row_height),
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

                    // Draw delete button (X icon)
                    let delete_center = Pos2::new(delete_button_x + delete_button_size / 2.0, y + row_height / 2.0);

                    // Draw button background (light gray circle)
                    painter.circle_filled(delete_center, delete_button_size / 2.0, Color32::from_rgb(80, 80, 80));

                    // Draw X
                    let x_size = delete_button_size * 0.4;
                    painter.line_segment(
                        [
                            delete_center + Vec2::new(-x_size, -x_size),
                            delete_center + Vec2::new(x_size, x_size),
                        ],
                        Stroke::new(2.0 * zoom, Color32::WHITE),
                    );
                    painter.line_segment(
                        [
                            delete_center + Vec2::new(x_size, -x_size),
                            delete_center + Vec2::new(-x_size, x_size),
                        ],
                        Stroke::new(2.0 * zoom, Color32::WHITE),
                    );
                }

                // Show "..." if there are more rows
                let bottom_y = if items.len() > max_visible_rows {
                    let y = rect.min.y + header_height + (max_visible_rows as f32 * row_height);
                    painter.text(
                        Pos2::new(rect.center().x, y),
                        egui::Align2::CENTER_CENTER,
                        format!("... {} more", items.len() - max_visible_rows),
                        egui::FontId::proportional(font_size),
                        Color32::from_gray(200),
                    );
                    y + row_height
                } else {
                    rect.min.y + header_height + (items.len() as f32 * row_height)
                };

                // Draw "Add Item" button at the bottom
                let add_button_height = 20.0 * zoom;
                let add_button_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 5.0, bottom_y + 5.0),
                    Vec2::new(rect.width() - 10.0, add_button_height),
                );

                // Button background
                painter.rect_filled(
                    add_button_rect,
                    3.0,
                    Color32::from_rgb(60, 120, 80),
                );

                // Button text
                painter.text(
                    add_button_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "+ Add Item",
                    egui::FontId::proportional((10.0 * zoom).max(8.0)),
                    Color32::WHITE,
                );
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

            // Check if node is right-clicked (for context menu)
            if response.secondary_clicked()
                && let Some(click_pos) = response.interact_pointer_pos()
                && rect.contains(click_pos)
            {
                // Show context menu
                if let Some(mut menu_info) = self.get_context_menu_info(node, rect, click_pos) {
                    menu_info.position = click_pos; // Save the click position
                    self.context_menu = Some(menu_info);
                    self.log_to_console("Context menu opened");
                }
            }
            // Check if node is clicked
            else if response.clicked()
                && let Some(click_pos) = response.interact_pointer_pos()
                && rect.contains(click_pos)
            {
                // Check what action was clicked
                if let Some(action) = self.get_click_action(node, rect, click_pos) {
                    match action {
                        ClickAction::EditCell(key, value_type) => {
                            // Enter edit mode for this cell
                            if let Some(current_value) = self.get_cell_value(node, &key) {
                                self.editing_cell = Some(EditingCell {
                                    node_id: node.id,
                                    key: key.clone(),
                                    text: current_value,
                                    value_type,
                                });
                                self.log_to_console(&format!(
                                    "Editing cell: {} = {:?}",
                                    key,
                                    self.editing_cell.as_ref().unwrap().text
                                ));
                            }
                        }
                        ClickAction::DeleteRow(key) => {
                            // Handle delete operation
                            let mut json_path = node.json_path.clone();
                            json_path.push(key.clone());

                            self.pending_edit = Some(EditResult {
                                json_path,
                                operation: ModifyOperation::Delete,
                            });

                            self.log_to_console(&format!("Delete row: {}", key));
                            selection_changed = true;
                        }
                        ClickAction::AddRow => {
                            // Show add property/item dialog
                            let is_object = matches!(node.content, NodeContent::Object(_));
                            self.adding_state = Some(AddingState {
                                node_id: node.id,
                                is_object,
                                key: String::new(),
                                value: String::new(),
                                value_type: NodeType::String, // Default to string
                            });
                            self.log_to_console(if is_object {
                                "Add property dialog opened"
                            } else {
                                "Add item dialog opened"
                            });
                        }
                        ClickAction::RenameKey(old_key) => {
                            // Show rename key dialog
                            self.renaming_key = Some(RenamingKey {
                                node_id: node.id,
                                old_key: old_key.clone(),
                                new_key: old_key.clone(), // Start with the old key
                            });
                            self.log_to_console(&format!("Rename key dialog opened: {}", old_key));
                        }
                    }
                } else {
                    // Just select the node
                    self.selected_node = Some(node.id);
                    selection_changed = true;
                    self.log_to_console(&format!(
                        "Selected node: {} (path: {:?})",
                        node.label, node.json_path
                    ));
                }
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

        // Show editing window if a cell is being edited
        let mut close_window = false;
        let mut save_edit = false;
        let mut edit_data: Option<(usize, String, String, NodeType)> = None;

        if let Some(editing) = &mut self.editing_cell {
            egui::Window::new("Edit Value")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Key:");
                        ui.label(&editing.key);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        ui.label(format!("{:?}", editing.value_type));
                    });

                    ui.separator();

                    ui.label("Value:");
                    let text_edit = egui::TextEdit::singleline(&mut editing.text)
                        .desired_width(300.0)
                        .font(egui::TextStyle::Monospace);

                    let response = ui.add(text_edit);

                    // Auto-focus on first show
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        save_edit = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close_window = true;
                    }

                    // Request focus if not focused
                    if !response.has_focus() {
                        response.request_focus();
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            save_edit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_window = true;
                        }
                    });

                    // Show validation hint
                    match editing.value_type {
                        NodeType::Number => {
                            ui.label(egui::RichText::new("💡 Enter a number").small().italics());
                        }
                        NodeType::Boolean => {
                            ui.label(egui::RichText::new("💡 Enter true or false").small().italics());
                        }
                        _ => {}
                    }
                });

            // Extract data for later use (to avoid borrow checker issues)
            if save_edit {
                edit_data = Some((
                    editing.node_id,
                    editing.key.clone(),
                    editing.text.clone(),
                    editing.value_type.clone(),
                ));
            }
        }

        // Process save outside of the borrow
        if let Some((node_id, key, text, value_type)) = edit_data {
            // Validate first
            if let Some(validated_value) = Self::validate_value(&text, &value_type) {
                // Then update
                if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id)
                    && Self::update_cell_value(node, &key, &validated_value)
                {
                    // Build complete JSON path for this edit
                    let mut json_path = node.json_path.clone();
                    json_path.push(key.clone());

                    // Store edit result for App to process
                    self.pending_edit = Some(EditResult {
                        json_path,
                        operation: ModifyOperation::Update {
                            new_value: validated_value.clone(),
                        },
                    });

                    self.log_to_console(&format!("Saved edit: {} = {}", key, text));
                    close_window = true;
                    selection_changed = true; // Trigger synchronization
                }
            } else {
                self.log_to_console("Validation failed");
            }
        }

        if close_window {
            self.editing_cell = None;
        }

        // Show adding dialog if adding a new property/item
        let mut close_add_dialog = false;
        let mut save_add = false;
        let mut add_data: Option<(usize, bool, String, String, NodeType)> = None;

        if let Some(adding) = &mut self.adding_state {
            egui::Window::new(if adding.is_object {
                "Add Property"
            } else {
                "Add Item"
            })
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                if adding.is_object {
                    // Object: need key and value
                    ui.label("Property Name:");
                    let key_response = ui.add(
                        egui::TextEdit::singleline(&mut adding.key)
                            .desired_width(300.0)
                            .font(egui::TextStyle::Monospace),
                    );

                    ui.separator();

                    ui.label("Value Type:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(matches!(adding.value_type, NodeType::String), "String").clicked() {
                            adding.value_type = NodeType::String;
                        }
                        if ui.selectable_label(matches!(adding.value_type, NodeType::Number), "Number").clicked() {
                            adding.value_type = NodeType::Number;
                        }
                        if ui.selectable_label(matches!(adding.value_type, NodeType::Boolean), "Boolean").clicked() {
                            adding.value_type = NodeType::Boolean;
                        }
                        if ui.selectable_label(matches!(adding.value_type, NodeType::Null), "Null").clicked() {
                            adding.value_type = NodeType::Null;
                        }
                    });

                    ui.separator();

                    ui.label("Value:");
                    let value_response = ui.add(
                        egui::TextEdit::singleline(&mut adding.value)
                            .desired_width(300.0)
                            .font(egui::TextStyle::Monospace),
                    );

                    // Auto-focus on first show
                    if !key_response.has_focus() && !value_response.has_focus() {
                        key_response.request_focus();
                    }

                    // Handle Enter/ESC
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        save_add = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close_add_dialog = true;
                    }
                } else {
                    // Array: only need value (index is automatic)
                    ui.label("Value Type:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(matches!(adding.value_type, NodeType::String), "String").clicked() {
                            adding.value_type = NodeType::String;
                        }
                        if ui.selectable_label(matches!(adding.value_type, NodeType::Number), "Number").clicked() {
                            adding.value_type = NodeType::Number;
                        }
                        if ui.selectable_label(matches!(adding.value_type, NodeType::Boolean), "Boolean").clicked() {
                            adding.value_type = NodeType::Boolean;
                        }
                        if ui.selectable_label(matches!(adding.value_type, NodeType::Null), "Null").clicked() {
                            adding.value_type = NodeType::Null;
                        }
                    });

                    ui.separator();

                    ui.label("Value:");
                    let value_response = ui.add(
                        egui::TextEdit::singleline(&mut adding.value)
                            .desired_width(300.0)
                            .font(egui::TextStyle::Monospace),
                    );

                    // Auto-focus
                    if !value_response.has_focus() {
                        value_response.request_focus();
                    }

                    // Handle Enter/ESC
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        save_add = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close_add_dialog = true;
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        save_add = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close_add_dialog = true;
                    }
                });

                // Show validation hint
                match adding.value_type {
                    NodeType::Number => {
                        ui.label(egui::RichText::new("💡 Enter a number").small().italics());
                    }
                    NodeType::Boolean => {
                        ui.label(egui::RichText::new("💡 Enter true or false").small().italics());
                    }
                    NodeType::Null => {
                        ui.label(egui::RichText::new("💡 Enter null").small().italics());
                    }
                    _ => {}
                }
            });

            // Extract data for later use
            if save_add {
                add_data = Some((
                    adding.node_id,
                    adding.is_object,
                    adding.key.clone(),
                    adding.value.clone(),
                    adding.value_type.clone(),
                ));
            }
        }

        // Process add outside of the borrow
        if let Some((node_id, is_object, key, value, value_type)) = add_data {
            // Validate key for Object
            if is_object && key.is_empty() {
                self.log_to_console("Property name cannot be empty");
            } else if let Some(validated_value) = Self::validate_value(&value, &value_type) {
                // Find the node to get its path
                if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                    let json_path = node.json_path.clone();

                    // Create the add operation
                    self.pending_edit = Some(EditResult {
                        json_path,
                        operation: ModifyOperation::Add {
                            key: if is_object { key.clone() } else { String::new() },
                            value: validated_value,
                        },
                    });

                    self.log_to_console(&format!(
                        "Added {} = {}",
                        if is_object { &key } else { "item" },
                        value
                    ));
                    close_add_dialog = true;
                    selection_changed = true;
                }
            } else {
                self.log_to_console("Invalid value");
            }
        }

        if close_add_dialog {
            self.adding_state = None;
        }

        // Show renaming dialog if renaming a key
        let mut close_rename_dialog = false;
        let mut save_rename = false;
        let mut rename_data: Option<(usize, String, String)> = None;

        if let Some(renaming) = &mut self.renaming_key {
            egui::Window::new("Rename Property")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Old Name:");
                        ui.label(&renaming.old_key);
                    });

                    ui.separator();

                    ui.label("New Name:");
                    let key_response = ui.add(
                        egui::TextEdit::singleline(&mut renaming.new_key)
                            .desired_width(300.0)
                            .font(egui::TextStyle::Monospace),
                    );

                    // Auto-focus on first show
                    if !key_response.has_focus() {
                        key_response.request_focus();
                    }

                    // Handle Enter/ESC
                    if key_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        save_rename = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close_rename_dialog = true;
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Rename").clicked() {
                            save_rename = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_rename_dialog = true;
                        }
                    });
                });

            // Extract data for later use
            if save_rename {
                rename_data = Some((
                    renaming.node_id,
                    renaming.old_key.clone(),
                    renaming.new_key.clone(),
                ));
            }
        }

        // Process rename outside of the borrow
        if let Some((node_id, old_key, new_key)) = rename_data {
            // Validate new key is not empty
            if new_key.is_empty() {
                self.log_to_console("Property name cannot be empty");
            } else if new_key == old_key {
                self.log_to_console("New name is the same as old name");
                close_rename_dialog = true;
            } else {
                // Find the node to get its path
                if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                    let json_path = node.json_path.clone();

                    // Create the rename operation
                    self.pending_edit = Some(EditResult {
                        json_path,
                        operation: ModifyOperation::Rename { old_key, new_key: new_key.clone() },
                    });

                    self.log_to_console(&format!("Renamed property to: {}", new_key));
                    close_rename_dialog = true;
                    selection_changed = true;
                }
            }
        }

        if close_rename_dialog {
            self.renaming_key = None;
        }

        // Show context menu if active
        let mut close_context_menu = false;

        if let Some(menu_state) = &self.context_menu {
            // Clone data for use after borrow ends
            let node_id = menu_state.node_id;
            let row_key = menu_state.row_key.clone();
            let is_object = menu_state.is_object;
            let is_primitive = menu_state.is_primitive;
            let value_type = menu_state.value_type.clone();
            let menu_position = menu_state.position;

            egui::Area::new(egui::Id::new("context_menu"))
                .fixed_pos(menu_position) // Use the saved position
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .show(ui, |ui| {
                            ui.set_min_width(150.0);

                    if let Some(key) = &row_key {
                        // Row-level context menu
                        if is_primitive
                            && ui.button("✏ Edit Value").clicked()
                        {
                            // Trigger edit action
                            if let Some(node) = self.nodes.iter().find(|n| n.id == node_id)
                                && let Some(current_value) = self.get_cell_value(node, key)
                            {
                                self.editing_cell = Some(EditingCell {
                                    node_id,
                                    key: key.clone(),
                                    text: current_value,
                                    value_type: value_type.clone().unwrap(),
                                });
                            }
                            close_context_menu = true;
                        }

                        if is_object
                            && ui.button("✎ Rename Key").clicked()
                        {
                            // Trigger rename action
                            self.renaming_key = Some(RenamingKey {
                                node_id,
                                old_key: key.clone(),
                                new_key: key.clone(),
                            });
                            close_context_menu = true;
                        }

                        if ui.button("🗑 Delete").clicked() {
                            // Trigger delete action
                            if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                                let mut json_path = node.json_path.clone();
                                json_path.push(key.clone());

                                self.pending_edit = Some(EditResult {
                                    json_path,
                                    operation: ModifyOperation::Delete,
                                });
                                selection_changed = true;
                            }
                            close_context_menu = true;
                        }
                    } else {
                        // Container-level context menu (add button area)
                        let label = if is_object { "➕ Add Property" } else { "➕ Add Item" };
                        if ui.button(label).clicked() {
                            self.adding_state = Some(AddingState {
                                node_id,
                                is_object,
                                key: String::new(),
                                value: String::new(),
                                value_type: NodeType::String,
                            });
                            close_context_menu = true;
                        }
                    }

                    ui.separator();
                    if ui.button("✖ Cancel").clicked() {
                        close_context_menu = true;
                    }
                        }); // Close Frame::popup
                }); // Close Area
        }

        if close_context_menu {
            self.context_menu = None;
        }

        // Close context menu if user clicks on the canvas (outside the menu)
        if self.context_menu.is_some() && response.clicked() {
            self.context_menu = None;
        }

        selection_changed
    }

    /// Transform position with zoom and offset
    fn transform_pos(&self, pos: Pos2, canvas_rect: Rect) -> Pos2 {
        let transformed = pos.to_vec2() * self.zoom + self.offset;
        canvas_rect.min + transformed
    }

    /// Check if a click position is on an action area (edit, delete, add button)
    /// Returns None if clicking on header or empty space
    fn get_click_action(&self, node: &GraphNode, rect: Rect, click_pos: Pos2) -> Option<ClickAction> {
        let header_height = 25.0 * self.zoom;
        let row_height = 22.0 * self.zoom;
        let delete_button_size = 16.0 * self.zoom;

        // Check if click is below header
        if click_pos.y < rect.min.y + header_height {
            return None; // Clicking on header
        }

        // Calculate which row was clicked
        let relative_y = click_pos.y - (rect.min.y + header_height);
        let row_index = (relative_y / row_height).floor() as usize;

        match &node.content {
            NodeContent::Object(pairs) => {
                let max_visible_rows = pairs.len().min(10);
                let key_column_width = rect.width() * 0.4;
                let delete_button_x = rect.max.x - delete_button_size - 5.0;

                // Check if clicking on "Add Property" button
                let bottom_y = if pairs.len() > 10 {
                    rect.min.y + header_height + (10.0 * row_height) + row_height
                } else {
                    rect.min.y + header_height + (pairs.len() as f32 * row_height)
                };
                let add_button_height = 20.0 * self.zoom;
                if click_pos.y >= bottom_y + 5.0
                    && click_pos.y <= bottom_y + 5.0 + add_button_height
                    && click_pos.x >= rect.min.x + 5.0
                    && click_pos.x <= rect.max.x - 5.0
                {
                    return Some(ClickAction::AddRow);
                }

                // Check if clicking within a valid row
                if row_index < max_visible_rows {
                    let pair = &pairs[row_index];
                    let y = rect.min.y + header_height + (row_index as f32 * row_height);

                    // Check if clicking on delete button
                    let delete_center_x = delete_button_x + delete_button_size / 2.0;
                    let delete_center_y = y + row_height / 2.0;
                    let distance = ((click_pos.x - delete_center_x).powi(2)
                        + (click_pos.y - delete_center_y).powi(2))
                    .sqrt();
                    if distance <= delete_button_size / 2.0 {
                        return Some(ClickAction::DeleteRow(pair.key.clone()));
                    }

                    // Check if clicking on key column for renaming
                    if click_pos.x >= rect.min.x + 5.0
                        && click_pos.x <= rect.min.x + key_column_width - 5.0
                    {
                        return Some(ClickAction::RenameKey(pair.key.clone()));
                    }

                    // Check if clicking on value column for editing (only primitives)
                    if !pair.is_reference && click_pos.x > rect.min.x + key_column_width
                        && click_pos.x < delete_button_x - 5.0
                    {
                        return Some(ClickAction::EditCell(
                            pair.key.clone(),
                            pair.value_type.clone(),
                        ));
                    }
                }
            }
            NodeContent::Array(items) => {
                let max_visible_rows = items.len().min(10);
                let index_column_width = 40.0 * self.zoom;
                let delete_button_x = rect.max.x - delete_button_size - 5.0;

                // Check if clicking on "Add Item" button
                let bottom_y = if items.len() > 10 {
                    rect.min.y + header_height + (10.0 * row_height) + row_height
                } else {
                    rect.min.y + header_height + (items.len() as f32 * row_height)
                };
                let add_button_height = 20.0 * self.zoom;
                if click_pos.y >= bottom_y + 5.0
                    && click_pos.y <= bottom_y + 5.0 + add_button_height
                    && click_pos.x >= rect.min.x + 5.0
                    && click_pos.x <= rect.max.x - 5.0
                {
                    return Some(ClickAction::AddRow);
                }

                // Check if clicking within a valid row
                if row_index < max_visible_rows {
                    let item = &items[row_index];
                    let y = rect.min.y + header_height + (row_index as f32 * row_height);

                    // Check if clicking on delete button
                    let delete_center_x = delete_button_x + delete_button_size / 2.0;
                    let delete_center_y = y + row_height / 2.0;
                    let distance = ((click_pos.x - delete_center_x).powi(2)
                        + (click_pos.y - delete_center_y).powi(2))
                    .sqrt();
                    if distance <= delete_button_size / 2.0 {
                        return Some(ClickAction::DeleteRow(item.index.to_string()));
                    }

                    // Check if clicking on value column for editing (only primitives)
                    if !item.is_reference
                        && click_pos.x > rect.min.x + index_column_width
                        && click_pos.x < delete_button_x - 5.0
                    {
                        return Some(ClickAction::EditCell(
                            item.index.to_string(),
                            item.value_type.clone(),
                        ));
                    }
                }
            }
            NodeContent::Primitive(_) => {
                // Primitive nodes don't have interactive elements
                return None;
            }
        }

        None
    }

    /// Get context menu information for a right-click position
    /// Returns None if not clicking on a row
    fn get_context_menu_info(&self, node: &GraphNode, rect: Rect, click_pos: Pos2) -> Option<ContextMenuState> {
        let header_height = 25.0 * self.zoom;
        let row_height = 22.0 * self.zoom;

        // Check if click is below header
        if click_pos.y < rect.min.y + header_height {
            return None; // Clicking on header
        }

        // Calculate which row was clicked
        let relative_y = click_pos.y - (rect.min.y + header_height);
        let row_index = (relative_y / row_height).floor() as usize;

        match &node.content {
            NodeContent::Object(pairs) => {
                let max_visible_rows = pairs.len().min(10);

                // Check if clicking on "Add Property" button area
                let bottom_y = if pairs.len() > 10 {
                    rect.min.y + header_height + (10.0 * row_height) + row_height
                } else {
                    rect.min.y + header_height + (pairs.len() as f32 * row_height)
                };
                let add_button_height = 20.0 * self.zoom;
                if click_pos.y >= bottom_y + 5.0
                    && click_pos.y <= bottom_y + 5.0 + add_button_height
                {
                    // Context menu for adding (container level)
                    return Some(ContextMenuState {
                        node_id: node.id,
                        row_key: None,
                        is_object: true,
                        is_primitive: false,
                        value_type: None,
                        position: Pos2::ZERO, // Will be set by caller
                    });
                }

                // Check if clicking within a valid row
                if row_index < max_visible_rows {
                    let pair = &pairs[row_index];
                    return Some(ContextMenuState {
                        node_id: node.id,
                        row_key: Some(pair.key.clone()),
                        is_object: true,
                        is_primitive: !pair.is_reference,
                        value_type: if !pair.is_reference {
                            Some(pair.value_type.clone())
                        } else {
                            None
                        },
                        position: Pos2::ZERO, // Will be set by caller
                    });
                }
            }
            NodeContent::Array(items) => {
                let max_visible_rows = items.len().min(10);

                // Check if clicking on "Add Item" button area
                let bottom_y = if items.len() > 10 {
                    rect.min.y + header_height + (10.0 * row_height) + row_height
                } else {
                    rect.min.y + header_height + (items.len() as f32 * row_height)
                };
                let add_button_height = 20.0 * self.zoom;
                if click_pos.y >= bottom_y + 5.0
                    && click_pos.y <= bottom_y + 5.0 + add_button_height
                {
                    // Context menu for adding (container level)
                    return Some(ContextMenuState {
                        node_id: node.id,
                        row_key: None,
                        is_object: false,
                        is_primitive: false,
                        value_type: None,
                        position: Pos2::ZERO, // Will be set by caller
                    });
                }

                // Check if clicking within a valid row
                if row_index < max_visible_rows {
                    let item = &items[row_index];
                    return Some(ContextMenuState {
                        node_id: node.id,
                        row_key: Some(item.index.to_string()),
                        is_object: false,
                        is_primitive: !item.is_reference,
                        value_type: if !item.is_reference {
                            Some(item.value_type.clone())
                        } else {
                            None
                        },
                        position: Pos2::ZERO, // Will be set by caller
                    });
                }
            }
            NodeContent::Primitive(_) => {
                // Primitive nodes don't have rows
                return None;
            }
        }

        None
    }

    /// Validate a value based on its type
    /// Returns Some(validated_string) if valid, None if invalid
    fn validate_value(new_value: &str, value_type: &NodeType) -> Option<String> {
        match value_type {
            NodeType::String => {
                // Strings are always valid
                Some(format!("\"{}\"", new_value))
            }
            NodeType::Number => {
                // Try to parse as number
                if new_value.parse::<f64>().is_ok() {
                    Some(new_value.to_string())
                } else {
                    None
                }
            }
            NodeType::Boolean => {
                // Must be "true" or "false"
                let lowercase = new_value.to_lowercase();
                if lowercase == "true" || lowercase == "false" {
                    Some(lowercase)
                } else {
                    None
                }
            }
            NodeType::Null => {
                // Only accept "null"
                if new_value.to_lowercase() == "null" {
                    Some("null".to_string())
                } else {
                    None
                }
            }
            _ => {
                // Object and Array types shouldn't be edited inline
                None
            }
        }
    }

    /// Update a cell value in a node
    /// Returns true if update succeeded
    fn update_cell_value(node: &mut GraphNode, key: &str, validated_value: &str) -> bool {
        match &mut node.content {
            NodeContent::Object(pairs) => {
                if let Some(pair) = pairs.iter_mut().find(|p| p.key == key) {
                    pair.value_display = validated_value.to_string();
                    return true;
                }
            }
            NodeContent::Array(items) => {
                if let Ok(index) = key.parse::<usize>()
                    && let Some(item) = items.get_mut(index)
                {
                    item.value_display = validated_value.to_string();
                    return true;
                }
            }
            NodeContent::Primitive(_) => {
                // Primitives don't have cells
                return false;
            }
        }

        false
    }

    /// Get the current value of a cell as a string
    fn get_cell_value(&self, node: &GraphNode, key: &str) -> Option<String> {
        match &node.content {
            NodeContent::Object(pairs) => {
                pairs.iter()
                    .find(|p| p.key == key)
                    .map(|p| {
                        // Return value without quotes for strings, raw for others
                        if p.value_type == NodeType::String {
                            // Remove quotes from display
                            let display = &p.value_display;
                            if display.starts_with('"') && display.ends_with('"') {
                                display[1..display.len()-1].to_string()
                            } else {
                                display.clone()
                            }
                        } else {
                            p.value_display.clone()
                        }
                    })
            }
            NodeContent::Array(items) => {
                if let Ok(index) = key.parse::<usize>() {
                    items.get(index).map(|item| {
                        // Return value without quotes for strings, raw for others
                        if item.value_type == NodeType::String {
                            let display = &item.value_display;
                            if display.starts_with('"') && display.ends_with('"') {
                                display[1..display.len()-1].to_string()
                            } else {
                                display.clone()
                            }
                        } else {
                            item.value_display.clone()
                        }
                    })
                } else {
                    None
                }
            }
            NodeContent::Primitive(_) => None,
        }
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

        // Only 1 node: the root object (primitive values shown in table)
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 0); // No edges since no child nodes

        // Check that the object has the key in its content
        if let NodeContent::Object(pairs) = &graph.nodes[0].content {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0].key, "key");
            assert_eq!(pairs[0].value_display, "\"value\"");
        } else {
            panic!("Expected Object content");
        }
    }

    #[test]
    fn test_build_array() {
        let mut graph = JsonGraph::new();
        let json = json!([1, 2, 3]);
        graph.build_from_json(&json);

        // Only 1 node: the array (primitive values shown in table)
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 0); // No edges since no child nodes

        // Check that the array has 3 items in its content
        if let NodeContent::Array(items) = &graph.nodes[0].content {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].value_display, "1");
            assert_eq!(items[1].value_display, "2");
            assert_eq!(items[2].value_display, "3");
        } else {
            panic!("Expected Array content");
        }
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

        // 2 nodes: root object + user object (name and age are shown in user's table)
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1); // One edge from root to user

        // Check that user object has name and age in its content
        // Find the second object node (id > 0)
        let user_node = graph.nodes.iter().find(|n| n.id > 0 && n.label.contains("Object"));
        assert!(user_node.is_some());
        if let NodeContent::Object(pairs) = &user_node.unwrap().content {
            assert_eq!(pairs.len(), 2);
            assert!(pairs.iter().any(|p| p.key == "name"));
            assert!(pairs.iter().any(|p| p.key == "age"));
        }
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

        // Expected: 4 nodes (only Objects and Arrays, not primitives)
        // 0: Object (3) - root (name, version shown in table)
        // 1: Array [2] - items
        // 2: Object (2) - items[0] (id, value shown in table)
        // 3: Object (2) - items[1] (id, value shown in table)

        assert_eq!(
            graph.nodes.len(),
            4,
            "Expected 4 nodes for default_json structure"
        );

        // Check that root object has name and version in its content
        let root_node = &graph.nodes[0];
        if let NodeContent::Object(pairs) = &root_node.content {
            assert_eq!(pairs.len(), 3); // name, version, items
            assert!(pairs.iter().any(|p| p.key == "name"));
            assert!(pairs.iter().any(|p| p.key == "version"));
        }

        // Check that items array has 2 objects as child nodes
        let items_node = graph.nodes.iter().find(|n| n.label.contains("Array"));
        assert!(items_node.is_some());

        // Check that item objects have id and value in their content
        let item_objects: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.label.contains("Object") && n.id > 0)
            .collect();
        assert_eq!(item_objects.len(), 2);

        for item in item_objects {
            if let NodeContent::Object(pairs) = &item.content {
                assert_eq!(pairs.len(), 2); // id and value
                assert!(pairs.iter().any(|p| p.key == "id"));
                assert!(pairs.iter().any(|p| p.key == "value"));
            }
        }
    }
}
