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
    /// Enable sync from editor to graph
    sync_editor_to_graph: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            json_editor: JsonEditor::new(),
            json_graph: JsonGraph::new(),
            left_panel_width: 400.0,
            graph_initialized: false,
            sync_graph_to_editor: true,
            sync_editor_to_graph: true,
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

                // Sync checkboxes
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

                if ui
                    .checkbox(&mut self.sync_editor_to_graph, "Sync Editor → Graph")
                    .clicked()
                {
                    utils::log(
                        "App",
                        &format!(
                            "Editor to Graph sync: {}",
                            if self.sync_editor_to_graph {
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

                // Check if a line was clicked in the editor (for editor-to-graph sync)
                if let Some(clicked_line) = self.json_editor.take_clicked_line()
                    && self.sync_editor_to_graph
                    && let Some(path) = self.json_editor.find_path_for_line(clicked_line)
                {
                    self.json_graph.select_by_path(&path);
                    utils::log(
                        "App",
                        &format!(
                            "Synced to graph: clicked line {} -> path {:?}",
                            clicked_line, path
                        ),
                    );
                }

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

            let selection_changed = self.json_graph.ui(ui);

            // Sync graph selection to editor if enabled
            if selection_changed
                && self.sync_graph_to_editor
                && let Some(path) = self.json_graph.get_selected_path()
                && let Some(line) = self.json_editor.find_line_for_path(&path)
            {
                self.json_editor.scroll_to_line(line);
                utils::log(
                    "App",
                    &format!("Synced to editor: line {} (path: {:?})", line, path),
                );
            }
        });
    }
}
