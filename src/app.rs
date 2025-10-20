/// Application UI and state
///
/// This module contains the main application UI logic using egui
use crate::json_editor::{JsonEditor, JsonGraph};
use egui;

/// Main application structure
pub struct App {
    /// JSON editor instance
    json_editor: JsonEditor,
    /// JSON graph visualizer
    json_graph: JsonGraph,
    /// Width of the left panel (JSON editor)
    left_panel_width: f32,
    /// Whether the graph has been initialized
    graph_initialized: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            json_editor: JsonEditor::new(),
            json_graph: JsonGraph::new(),
            left_panel_width: 400.0,
            graph_initialized: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the UI
    pub fn update(&mut self, ctx: &egui::Context) {
        // Top panel for title and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("WGPU Canvas Editor - JSON Visualizer");
                ui.separator();

                if ui.button("Reset Layout").clicked() {
                    self.left_panel_width = 400.0;
                    self.log_to_console("Layout reset");
                }
            });
        });

        // Left panel for JSON editor
        egui::SidePanel::left("json_editor_panel")
            .resizable(true)
            .default_width(self.left_panel_width)
            .width_range(200.0..=800.0)
            .show(ctx, |ui| {
                ui.heading("JSON Editor");
                ui.separator();

                let changed = self.json_editor.ui(ui);

                // Update graph if JSON changed and is valid
                if changed
                    && self.json_editor.is_valid()
                    && let Some(value) = self.json_editor.parsed_value()
                {
                    self.json_graph.build_from_json(value);
                    self.log_to_console("Graph updated from JSON");
                }
            });

        // Central panel for graph visualization
        egui::CentralPanel::default().show(ctx, |ui| {
            // Initialize graph on first frame if JSON is valid
            if !self.graph_initialized
                && self.json_editor.is_valid()
                && let Some(value) = self.json_editor.parsed_value()
            {
                self.json_graph.build_from_json(value);
                self.graph_initialized = true;
            }

            self.json_graph.ui(ui);
        });
    }

    /// Log message to browser console (WASM) or stdout (desktop)
    fn log_to_console(&self, message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("[App] {}", message).into());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("[App] {}", message);
        }
    }
}
