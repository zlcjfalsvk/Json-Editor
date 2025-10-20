use serde_json::Value;

/// JSON Editor state and functionality
pub struct JsonEditor {
    /// The raw JSON text being edited
    text: String,
    /// Parsed JSON value (None if invalid)
    parsed_value: Option<Value>,
    /// Last validation error message
    error_message: Option<String>,
    /// Whether to show pretty-printed JSON
    pretty_print: bool,
    /// Current indentation level for pretty printing
    indent_size: usize,
}

impl Default for JsonEditor {
    fn default() -> Self {
        let default_json = r#"{
  "name": "example",
  "version": "1.0.0",
  "items": [
    {"id": 1, "value": "first"},
    {"id": 2, "value": "second"}
  ]
}"#;

        Self {
            text: default_json.to_string(),
            parsed_value: serde_json::from_str(default_json).ok(),
            error_message: None,
            pretty_print: true,
            indent_size: 2,
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
            parsed_value: None,
            error_message: None,
            pretty_print: true,
            indent_size: 2,
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
        self.text = text;
        self.validate();
        self.log_to_console("JSON content updated");
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
            self.text = pretty;
            self.log_to_console("Applied pretty print");
        }
    }

    /// Compact the JSON (remove unnecessary whitespace)
    pub fn apply_compact(&mut self) {
        if let Some(ref value) = self.parsed_value
            && let Ok(compact) = serde_json::to_string(value)
        {
            self.text = compact;
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

    /// Log message to browser console (WASM) or stdout (desktop)
    fn log_to_console(&self, message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("[JSON Editor] {}", message).into());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("[JSON Editor] {}", message);
        }
    }

    /// Render the editor UI using egui
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let text_edit_id = ui.id().with("json_text_edit");

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("Pretty").clicked() && self.is_valid() {
                self.apply_pretty_print();
                changed = true;
            }

            if ui.button("Compact").clicked() && self.is_valid() {
                self.apply_compact();
                changed = true;
            }

            ui.separator();

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

        // Text editor with persistent ID
        let response = ui.add(
            egui::TextEdit::multiline(&mut self.text)
                .id(text_edit_id)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(30)
                .code_editor(),
        );

        if response.changed() {
            let was_valid = self.is_valid();
            self.validate();
            self.log_to_console("Text changed");
            changed = true;

            // If validation failed, maintain focus on the text editor
            if !self.is_valid() && was_valid {
                ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                self.log_to_console("JSON validation failed - focus maintained");
            }
        }

        changed
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
