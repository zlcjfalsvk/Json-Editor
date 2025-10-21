use crate::utils;
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

/// View mode for JSON editor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    /// Raw text editor mode
    Text,
    /// Tree view with folding
    Tree,
}

/// JSON Editor state and functionality
pub struct JsonEditor {
    /// The raw JSON text being edited
    text: String,
    /// Previous text for undo tracking
    previous_text: String,
    /// Parsed JSON value (None if invalid)
    parsed_value: Option<Value>,
    /// Last validation error message
    error_message: Option<String>,
    /// Whether to show pretty-printed JSON
    pretty_print: bool,
    /// Current indentation level for pretty printing
    indent_size: usize,
    /// Undo history stack
    undo_stack: Vec<String>,
    /// Redo history stack
    redo_stack: Vec<String>,
    /// Maximum history size
    max_history: usize,
    /// Show line numbers
    show_line_numbers: bool,
    /// Target line to scroll to (None if no scroll needed)
    target_line: Option<usize>,
    /// Clicked line number (for editor-to-graph sync)
    clicked_line: Option<usize>,
    /// Current view mode
    view_mode: ViewMode,
}

impl Default for JsonEditor {
    fn default() -> Self {
        let default_json = r#"{
  "name": "example",
  "version": "1.0.0",
  "languages": {
    "korean": "안녕하세요",
    "chinese": "你好",
    "japanese": "こんにちは",
    "english": "Hello"
  },
  "items": [
    {"id": 1, "value": "first"},
    {"id": 2, "value": "second"}
  ]
}"#;

        Self {
            text: default_json.to_string(),
            previous_text: default_json.to_string(),
            parsed_value: serde_json::from_str(default_json).ok(),
            error_message: None,
            pretty_print: true,
            indent_size: 2,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
            show_line_numbers: true,
            target_line: None,
            clicked_line: None,
            view_mode: ViewMode::Text,
        }
    }
}

impl JsonEditor {
    /// Create a new JSON editor with empty content
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a JSON editor with initial content
    pub fn with_text(text: String) -> Self {
        let mut editor = Self {
            text: text.clone(),
            previous_text: text.clone(),
            parsed_value: None,
            error_message: None,
            pretty_print: true,
            indent_size: 2,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
            show_line_numbers: true,
            target_line: None,
            clicked_line: None,
            view_mode: ViewMode::Text,
        };
        editor.validate();
        editor
    }

    /// Get the current text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set new text content
    pub fn set_text(&mut self, text: String) {
        self.push_undo();
        self.text = text;
        self.validate();
        self.log_to_console("JSON content updated");
    }

    /// Push current text to undo stack
    fn push_undo(&mut self) {
        self.undo_stack.push(self.text.clone());
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo last change
    pub fn undo(&mut self) -> bool {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(self.text.clone());
            self.text = previous.clone();
            self.previous_text = previous;
            self.validate();
            self.log_to_console("Undo");
            true
        } else {
            false
        }
    }

    /// Redo last undone change
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.text.clone());
            self.text = next.clone();
            self.previous_text = next;
            self.validate();
            self.log_to_console("Redo");
            true
        } else {
            false
        }
    }

    /// Can undo
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Can redo
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Scroll to specific line
    pub fn scroll_to_line(&mut self, line: usize) {
        self.target_line = Some(line);
        self.log_to_console(&format!("Scroll to line {}", line));
    }

    /// Find line number for a JSON path
    /// Returns the line number (1-indexed) where the path can be found
    pub fn find_line_for_path(&self, path: &[String]) -> Option<usize> {
        if path.is_empty() {
            return Some(1); // Root is at line 1
        }

        let lines: Vec<&str> = self.text.lines().collect();
        let mut current_line = 0;
        let mut path_index = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check if this line contains the current path segment
            if path_index < path.len() {
                let segment = &path[path_index];

                // For object keys
                if trimmed.contains(&format!("\"{}\"", segment)) {
                    current_line = line_num + 1; // 1-indexed
                    path_index += 1;

                    // If we've found all path segments, return this line
                    if path_index == path.len() {
                        return Some(current_line);
                    }
                }
            }
        }

        // If we didn't find the exact path, return the last matched line
        if current_line > 0 {
            Some(current_line)
        } else {
            None
        }
    }

    /// Get and clear the clicked line (for one-time event handling)
    pub fn take_clicked_line(&mut self) -> Option<usize> {
        self.clicked_line.take()
    }

    /// Find JSON path for a given line number
    /// This is a reverse lookup: line number -> JSON path
    pub fn find_path_for_line(&self, target_line: usize) -> Option<Vec<String>> {
        if target_line == 0 {
            return None;
        }

        let lines: Vec<&str> = self.text.lines().collect();
        if target_line > lines.len() {
            return None;
        }

        // Simple heuristic: look for JSON keys on or near the target line
        let mut path = Vec::new();

        // Start from beginning and track the path to target_line
        for line_num in 1..=target_line {
            if line_num > lines.len() {
                break;
            }

            let line = lines[line_num - 1];
            let trimmed = line.trim();

            // Look for object keys: "key":
            if let Some(key_start) = trimmed.find('"')
                && let Some(key_end) = trimmed[key_start + 1..].find('"')
            {
                let key = &trimmed[key_start + 1..key_start + 1 + key_end];
                // Check if this is followed by a colon (indicating it's a key)
                if trimmed[key_start + 1 + key_end + 1..]
                    .trim_start()
                    .starts_with(':')
                {
                    // This is a key - add to path if we're building towards target
                    if line_num <= target_line {
                        path.push(key.to_string());
                    }
                }
            }
        }

        if path.is_empty() { None } else { Some(path) }
    }

    /// Toggle line numbers
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// Validate the JSON syntax
    pub fn validate(&mut self) -> bool {
        match serde_json::from_str::<Value>(&self.text) {
            Ok(value) => {
                self.parsed_value = Some(value);
                self.error_message = None;
                true
            }
            Err(e) => {
                self.parsed_value = None;
                self.error_message = Some(format!("JSON Error: {}", e));
                false
            }
        }
    }

    /// Get the validation error message if any
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Check if the current JSON is valid
    pub fn is_valid(&self) -> bool {
        self.parsed_value.is_some()
    }

    /// Get the parsed JSON value
    pub fn parsed_value(&self) -> Option<&Value> {
        self.parsed_value.as_ref()
    }

    /// Apply pretty printing to the JSON
    pub fn apply_pretty_print(&mut self) {
        if let Some(ref value) = self.parsed_value
            && let Ok(pretty) = serde_json::to_string_pretty(value)
        {
            self.text = pretty.clone();
            self.previous_text = pretty;
            self.log_to_console("Applied pretty print");
        }
    }

    /// Compact the JSON (remove unnecessary whitespace)
    pub fn apply_compact(&mut self) {
        if let Some(ref value) = self.parsed_value
            && let Ok(compact) = serde_json::to_string(value)
        {
            self.text = compact.clone();
            self.previous_text = compact;
            self.log_to_console("Applied compact format");
        }
    }

    /// Toggle pretty print mode
    pub fn toggle_pretty_print(&mut self) {
        self.pretty_print = !self.pretty_print;
        if self.pretty_print {
            self.apply_pretty_print();
        } else {
            self.apply_compact();
        }
    }

    /// Set indent size for pretty printing
    pub fn set_indent_size(&mut self, size: usize) {
        self.indent_size = size;
    }

    /// Get current indent size
    pub fn indent_size(&self) -> usize {
        self.indent_size
    }

    /// Toggle view mode between Text and Tree
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Text => ViewMode::Tree,
            ViewMode::Tree => ViewMode::Text,
        };
        self.log_to_console(&format!("View mode: {:?}", self.view_mode));
    }

    /// Update a value at a specific JSON path
    /// Returns true if the update succeeded
    pub fn update_value_at_path(&mut self, path: &[String], new_value_str: &str) -> bool {
        if let Some(mut value) = self.parsed_value.clone() {
            // Navigate to the target location
            if let Some(target) = Self::navigate_to_path_mut(&mut value, path) {
                // Parse the new value based on its format
                let new_value = if new_value_str.starts_with('"') && new_value_str.ends_with('"') {
                    // It's a string (with quotes)
                    serde_json::Value::String(new_value_str[1..new_value_str.len() - 1].to_string())
                } else if let Ok(num) = new_value_str.parse::<f64>() {
                    // It's a number
                    serde_json::json!(num)
                } else if new_value_str == "true" {
                    serde_json::Value::Bool(true)
                } else if new_value_str == "false" {
                    serde_json::Value::Bool(false)
                } else if new_value_str == "null" {
                    serde_json::Value::Null
                } else {
                    // Default to string without quotes
                    serde_json::Value::String(new_value_str.to_string())
                };

                *target = new_value;

                // Update the text with pretty-printed JSON
                if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                    self.push_undo();
                    self.text = pretty.clone();
                    self.previous_text = pretty;
                    self.parsed_value = Some(value);
                    self.error_message = None;
                    self.log_to_console(&format!("Updated value at path: {:?}", path));
                    return true;
                }
            }
        }
        false
    }

    /// Navigate to a mutable reference at a JSON path
    fn navigate_to_path_mut<'a>(value: &'a mut Value, path: &[String]) -> Option<&'a mut Value> {
        let mut current = value;

        for segment in path {
            current = match current {
                Value::Object(map) => map.get_mut(segment)?,
                Value::Array(arr) => {
                    let index: usize = segment.parse().ok()?;
                    arr.get_mut(index)?
                }
                _ => return None,
            };
        }

        Some(current)
    }

    /// Delete a value at a specific JSON path
    /// Returns true if the delete succeeded
    pub fn delete_value_at_path(&mut self, path: &[String]) -> bool {
        if path.is_empty() {
            return false;
        }

        if let Some(mut value) = self.parsed_value.clone() {
            let parent_path = &path[..path.len() - 1];
            let key = &path[path.len() - 1];

            if let Some(parent) = Self::navigate_to_path_mut(&mut value, parent_path) {
                match parent {
                    Value::Object(map) => {
                        if map.remove(key).is_some() {
                            // Update the text with pretty-printed JSON
                            if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                                self.push_undo();
                                self.text = pretty.clone();
                                self.previous_text = pretty;
                                self.parsed_value = Some(value);
                                self.error_message = None;
                                self.log_to_console(&format!("Deleted property: {}", key));
                                return true;
                            }
                        }
                    }
                    Value::Array(arr) => {
                        if let Ok(index) = key.parse::<usize>()
                            && index < arr.len()
                        {
                            arr.remove(index);
                            // Update the text with pretty-printed JSON
                            if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                                self.push_undo();
                                self.text = pretty.clone();
                                self.previous_text = pretty;
                                self.parsed_value = Some(value);
                                self.error_message = None;
                                self.log_to_console(&format!(
                                    "Deleted array item at index: {}",
                                    index
                                ));
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Add a value at a specific JSON path
    /// For Objects: key is the property name, value_str is the value
    /// For Arrays: key is empty, value_str is appended to the array
    /// Returns true if the add succeeded
    pub fn add_value_at_path(&mut self, path: &[String], key: &str, value_str: &str) -> bool {
        if let Some(mut value) = self.parsed_value.clone()
            && let Some(target) = Self::navigate_to_path_mut(&mut value, path)
        {
            // Parse the new value based on its format
            let new_value = if value_str.starts_with('"') && value_str.ends_with('"') {
                // It's a string (with quotes)
                serde_json::Value::String(value_str[1..value_str.len() - 1].to_string())
            } else if let Ok(num) = value_str.parse::<f64>() {
                // It's a number
                serde_json::json!(num)
            } else if value_str == "true" {
                serde_json::Value::Bool(true)
            } else if value_str == "false" {
                serde_json::Value::Bool(false)
            } else if value_str == "null" {
                serde_json::Value::Null
            } else {
                // Default to string without quotes
                serde_json::Value::String(value_str.to_string())
            };

            match target {
                Value::Object(map) => {
                    if key.is_empty() {
                        self.log_to_console("Property name cannot be empty");
                        return false;
                    }
                    // Add new property to object
                    map.insert(key.to_string(), new_value);

                    // Update the text with pretty-printed JSON
                    if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                        self.push_undo();
                        self.text = pretty.clone();
                        self.previous_text = pretty;
                        self.parsed_value = Some(value);
                        self.error_message = None;
                        self.log_to_console(&format!("Added property: {} = {}", key, value_str));
                        return true;
                    }
                }
                Value::Array(arr) => {
                    // Append new item to array
                    arr.push(new_value);

                    // Update the text with pretty-printed JSON
                    if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                        self.push_undo();
                        self.text = pretty.clone();
                        self.previous_text = pretty;
                        self.parsed_value = Some(value);
                        self.error_message = None;
                        self.log_to_console(&format!("Added array item: {}", value_str));
                        return true;
                    }
                }
                _ => {
                    self.log_to_console("Cannot add to non-Object/Array value");
                    return false;
                }
            }
        }
        false
    }

    /// Rename a property key in an Object
    /// Path points to the Object containing the key to rename
    /// Returns true if the rename succeeded
    pub fn rename_key_at_path(&mut self, path: &[String], old_key: &str, new_key: &str) -> bool {
        if let Some(mut value) = self.parsed_value.clone()
            && let Some(target) = Self::navigate_to_path_mut(&mut value, path)
        {
            match target {
                Value::Object(map) => {
                    // Check if old key exists
                    if !map.contains_key(old_key) {
                        self.log_to_console(&format!("Property '{}' not found", old_key));
                        return false;
                    }

                    // Check if new key already exists
                    if map.contains_key(new_key) && old_key != new_key {
                        self.log_to_console(&format!("Property '{}' already exists", new_key));
                        return false;
                    }

                    // Remove old key and insert with new key
                    if let Some(old_value) = map.remove(old_key) {
                        map.insert(new_key.to_string(), old_value);

                        // Update the text with pretty-printed JSON
                        if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                            self.push_undo();
                            self.text = pretty.clone();
                            self.previous_text = pretty;
                            self.parsed_value = Some(value);
                            self.error_message = None;
                            self.log_to_console(&format!(
                                "Renamed property: {} -> {}",
                                old_key, new_key
                            ));
                            return true;
                        }
                    }
                }
                _ => {
                    self.log_to_console("Cannot rename key in non-Object value");
                    return false;
                }
            }
        }
        false
    }

    /// Render JSON tree view recursively
    #[allow(clippy::only_used_in_recursion)]
    fn render_tree_view(&self, ui: &mut egui::Ui, value: &Value, key: Option<&str>, path: String) {
        match value {
            Value::Object(map) => {
                let header_text = if let Some(k) = key {
                    format!("{}: {{ {} items }}", k, map.len())
                } else {
                    format!("{{ {} items }}", map.len())
                };

                egui::CollapsingHeader::new(header_text)
                    .id_salt(path.clone())
                    .default_open(true)
                    .show(ui, |ui| {
                        for (k, v) in map {
                            let new_path = if path.is_empty() {
                                k.clone()
                            } else {
                                format!("{}.{}", path, k)
                            };
                            self.render_tree_view(ui, v, Some(k), new_path);
                        }
                    });
            }
            Value::Array(arr) => {
                let header_text = if let Some(k) = key {
                    format!("{}: [ {} items ]", k, arr.len())
                } else {
                    format!("[ {} items ]", arr.len())
                };

                egui::CollapsingHeader::new(header_text)
                    .id_salt(path.clone())
                    .default_open(true)
                    .show(ui, |ui| {
                        for (idx, v) in arr.iter().enumerate() {
                            let new_path = format!("{}[{}]", path, idx);
                            self.render_tree_view(ui, v, Some(&format!("[{}]", idx)), new_path);
                        }
                    });
            }
            Value::String(s) => {
                let text = if let Some(k) = key {
                    format!("{}: \"{}\"", k, s)
                } else {
                    format!("\"{}\"", s)
                };
                ui.label(egui::RichText::new(text).color(egui::Color32::from_rgb(100, 200, 100)));
            }
            Value::Number(n) => {
                let text = if let Some(k) = key {
                    format!("{}: {}", k, n)
                } else {
                    format!("{}", n)
                };
                ui.label(egui::RichText::new(text).color(egui::Color32::from_rgb(200, 150, 100)));
            }
            Value::Bool(b) => {
                let text = if let Some(k) = key {
                    format!("{}: {}", k, b)
                } else {
                    format!("{}", b)
                };
                ui.label(egui::RichText::new(text).color(egui::Color32::from_rgb(200, 100, 150)));
            }
            Value::Null => {
                let text = if let Some(k) = key {
                    format!("{}: null", k)
                } else {
                    "null".to_string()
                };
                ui.label(egui::RichText::new(text).color(egui::Color32::from_gray(150)));
            }
        }
    }

    /// Log message to browser console (WASM) or stdout (desktop)
    fn log_to_console(&self, message: &str) {
        utils::log("JSON Editor", message);
    }

    /// Render the editor UI using egui
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let text_edit_id = ui.id().with("json_text_edit");

        // Handle keyboard shortcuts
        let mut undo_requested = false;
        let mut redo_requested = false;
        let mut select_all_requested = false;

        // Use ctx.input() to check global keyboard events instead of ui.input()
        // This ensures shortcuts work even when focus is in nested UI elements
        ui.ctx().input(|i| {
            // Undo: Ctrl+Z (Windows/Linux) or Cmd+Z (macOS)
            if i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift {
                undo_requested = true;
            }

            // Redo: Ctrl+Y (Windows/Linux) or Cmd+Shift+Z (macOS) or Ctrl+Shift+Z
            if (i.modifiers.command && i.key_pressed(egui::Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
            {
                redo_requested = true;
            }

            // Select All: Ctrl+A (Windows/Linux) or Cmd+A (macOS)
            // Note: egui's TextEdit handles this automatically, but we detect it here for logging
            if i.modifiers.command && i.key_pressed(egui::Key::A) {
                select_all_requested = true;
            }
        });

        // Process undo/redo requests
        if undo_requested && self.can_undo() {
            self.undo();
            changed = true;
            self.log_to_console("Undo via keyboard shortcut");
        }
        if redo_requested && self.can_redo() {
            self.redo();
            changed = true;
            self.log_to_console("Redo via keyboard shortcut");
        }
        if select_all_requested {
            self.log_to_console("Select all via keyboard shortcut");
        }

        // Toolbar
        ui.horizontal(|ui| {
            // View mode toggle
            let view_text = match self.view_mode {
                ViewMode::Text => "📝 Text",
                ViewMode::Tree => "🌲 Tree",
            };
            if ui.button(view_text).clicked() {
                self.toggle_view_mode();
            }

            ui.separator();

            // Format buttons (only in text mode)
            if self.view_mode == ViewMode::Text {
                if ui.button("Pretty").clicked() && self.is_valid() {
                    self.push_undo();
                    self.apply_pretty_print();
                    changed = true;
                }

                if ui.button("Compact").clicked() && self.is_valid() {
                    self.push_undo();
                    self.apply_compact();
                    changed = true;
                }
            }

            ui.separator();

            // Edit buttons
            if ui
                .add_enabled(self.can_undo(), egui::Button::new("Undo"))
                .clicked()
            {
                self.undo();
                changed = true;
            }

            if ui
                .add_enabled(self.can_redo(), egui::Button::new("Redo"))
                .clicked()
            {
                self.redo();
                changed = true;
            }

            ui.separator();

            // Line numbers toggle (only in text mode)
            if self.view_mode == ViewMode::Text {
                if ui
                    .checkbox(&mut self.show_line_numbers, "Line Numbers")
                    .clicked()
                {
                    self.log_to_console(&format!(
                        "Line numbers: {}",
                        if self.show_line_numbers { "on" } else { "off" }
                    ));
                }

                ui.separator();
            }

            // Validation status
            if self.is_valid() {
                ui.colored_label(egui::Color32::GREEN, "✓ Valid JSON");
            } else {
                ui.colored_label(egui::Color32::RED, "✗ Invalid JSON");
            }
        });

        ui.separator();

        // Error message
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }

        // Render based on view mode
        match self.view_mode {
            ViewMode::Tree => {
                // Tree view with folding
                if let Some(value) = &self.parsed_value {
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            self.render_tree_view(ui, value, None, String::new());
                        });
                } else {
                    ui.colored_label(
                        egui::Color32::RED,
                        "Invalid JSON - cannot display tree view",
                    );
                }
            }
            ViewMode::Text => {
                // Original text editor view
                self.render_text_editor(ui, &mut changed, text_edit_id);
            }
        }

        changed
    }

    /// Render the text editor mode
    fn render_text_editor(
        &mut self,
        ui: &mut egui::Ui,
        changed: &mut bool,
        text_edit_id: egui::Id,
    ) {
        // Editor area with line numbers - use all available height
        let available_height = ui.available_height();

        // Calculate scroll offset if we need to scroll to a target line
        let scroll_offset = if let Some(target) = self.target_line {
            let line_height = 17.0;
            Some((target as f32 - 1.0) * line_height)
        } else {
            None
        };

        // Clear target_line after calculating offset
        if self.target_line.is_some() {
            self.target_line = None;
        }

        // Single ScrollArea containing both line numbers and editor
        let mut scroll_area = egui::ScrollArea::vertical()
            .id_salt("json_editor_scroll")
            .max_height(available_height);

        // Apply scroll offset if needed
        if let Some(offset) = scroll_offset {
            scroll_area = scroll_area.vertical_scroll_offset(offset);
        }

        scroll_area.show(ui, |ui| {
            ui.horizontal_top(|ui| {
                // Line numbers column
                if self.show_line_numbers {
                    let line_count = self.text.lines().count();
                    let line_number_width = 50.0;

                    ui.allocate_ui_with_layout(
                        egui::vec2(line_number_width, available_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.style_mut().spacing.item_spacing.y = 0.0;
                            // Use fixed line height matching monospace font
                            let line_height = 17.0;

                            for i in 1..=line_count {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(line_number_width, line_height),
                                    egui::Layout::top_down(egui::Align::Max),
                                    |ui| {
                                        // Make line number clickable
                                        let line_label = ui.selectable_label(
                                            false,
                                            egui::RichText::new(format!("{:>4}", i))
                                                .color(egui::Color32::from_gray(128)),
                                        );

                                        // Detect click
                                        if line_label.clicked() {
                                            self.clicked_line = Some(i);
                                            self.log_to_console(&format!("Line {} clicked", i));
                                        }
                                    },
                                );
                            }
                        },
                    );

                    ui.separator();
                }

                // Text editor - now using full available space
                let text_edit = egui::TextEdit::multiline(&mut self.text)
                    .id(text_edit_id)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .code_editor()
                    .char_limit(usize::MAX) // No character limit for JSON spec compliance
                    .lock_focus(true); // Maintain focus for IME input (Korean, etc.)

                let response = ui.add(text_edit);

                if response.changed() {
                    // Apply Unicode NFC normalization for Korean input
                    self.text = self.text.nfc().collect();

                    // Push previous text to undo stack for per-character undo
                    if self.text != self.previous_text {
                        self.undo_stack.push(self.previous_text.clone());
                        if self.undo_stack.len() > self.max_history {
                            self.undo_stack.remove(0);
                        }
                        self.redo_stack.clear();
                        self.previous_text = self.text.clone();
                    }

                    let was_valid = self.is_valid();
                    self.validate();
                    self.log_to_console("Text changed");
                    *changed = true;

                    // If validation failed, maintain focus on the text editor
                    if !self.is_valid() && was_valid {
                        ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                        self.log_to_console("JSON validation failed - focus maintained");
                    }
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_editor() {
        let editor = JsonEditor::new();
        assert!(editor.is_valid());
        assert!(!editor.text().is_empty());
    }

    #[test]
    fn test_valid_json() {
        let editor = JsonEditor::with_text(r#"{"key": "value"}"#.to_string());
        assert!(editor.is_valid());
        assert!(editor.error_message().is_none());
    }

    #[test]
    fn test_invalid_json() {
        let editor = JsonEditor::with_text(r#"{"key": invalid}"#.to_string());
        assert!(!editor.is_valid());
        assert!(editor.error_message().is_some());
    }

    #[test]
    fn test_pretty_print() {
        let mut editor = JsonEditor::with_text(r#"{"a":1,"b":2}"#.to_string());
        assert!(editor.is_valid());

        editor.apply_pretty_print();
        assert!(editor.text().contains('\n'));
        assert!(editor.text().contains("  "));
    }

    #[test]
    fn test_compact() {
        let mut editor = JsonEditor::with_text(
            r#"{
  "a": 1,
  "b": 2
}"#
            .to_string(),
        );
        assert!(editor.is_valid());

        editor.apply_compact();
        assert!(!editor.text().contains('\n'));
    }

    #[test]
    fn test_set_text() {
        let mut editor = JsonEditor::new();
        editor.set_text(r#"{"new": "value"}"#.to_string());
        assert!(editor.is_valid());
        assert_eq!(editor.text(), r#"{"new": "value"}"#);
    }
}
