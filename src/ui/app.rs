/// Application UI and state
///
/// This module contains the main application UI logic using egui
use crate::json_editor::{JsonEditor, JsonGraph};
use crate::utils;
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
    /// Enable sync from graph to editor
    sync_graph_to_editor: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            json_editor: JsonEditor::new(),
            json_graph: JsonGraph::new(),
            left_panel_width: 400.0,
            graph_initialized: false,
            sync_graph_to_editor: true,
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
                    utils::log("App", "Layout reset");
                }

                ui.separator();

                // Sync checkbox
                if ui
                    .checkbox(&mut self.sync_graph_to_editor, "Sync Graph → Editor")
                    .clicked()
                {
                    utils::log(
                        "App",
                        &format!(
                            "Graph to Editor sync: {}",
                            if self.sync_graph_to_editor {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        ),
                    );
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
                // OR if graph hasn't been initialized yet but JSON is valid
                if changed && self.json_editor.is_valid() {
                    if let Some(value) = self.json_editor.parsed_value() {
                        self.json_graph.build_from_json(value);
                        self.graph_initialized = true;
                        utils::log("App", "Graph updated from JSON");
                    }
                } else if changed && !self.json_editor.is_valid() {
                    // Clear graph if JSON becomes invalid
                    self.json_graph.build_from_json(&serde_json::Value::Null);
                    utils::log("App", "Graph cleared - invalid JSON");
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
}
