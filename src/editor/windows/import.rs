use std::{collections::HashSet, ops::ControlFlow, path::Path};

use macroquad::prelude::{collections::storage, ivec2, uvec2, Color, IVec2, UVec2};

use crate::{
    editor::util::EguiTextureHandler,
    map::{Map, MapBackgroundLayer, MapTileset},
    resources::{map_name_to_filename, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION},
    Resources,
};

pub struct ImportWindow {
    map_importing_from: Map,
    selected_tilesets: HashSet<String>,
    should_import_background: bool,
}
pub enum ImportResult {
    Import {
        tilesets: Vec<MapTileset>,
        background_color: Option<Color>,
        background_layers: Vec<MapBackgroundLayer>,
    },
    Close,
}

impl ImportWindow {
    pub fn new(map: Map) -> Self {
        Self {
            map_importing_from: map,
            selected_tilesets: HashSet::new(),
            should_import_background: false,
        }
    }

    pub fn ui(&mut self, egui_ctx: &egui::Context) -> ControlFlow<ImportResult> {
        let mut action = ControlFlow::Continue(());

        egui::Window::new("Import To Map")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(egui_ctx, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .column(egui_extras::Size::remainder())
                    .header(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Tilesets to import:");
                        });
                    })
                    .body(|body| {
                        body.rows(
                            18.0,
                            self.map_importing_from.tilesets.len(),
                            |idx, mut row| {
                                let key = self.map_importing_from.tilesets.keys().nth(idx).unwrap();
                                let is_selected = self.selected_tilesets.contains(key);
                                let mut clicked = false;

                                row.col(|ui| {
                                    clicked =
                                        ui.toggle_value(&mut (is_selected.clone()), key).clicked();
                                });

                                if clicked {
                                    if is_selected {
                                        self.selected_tilesets.remove(key);
                                    } else {
                                        self.selected_tilesets.insert(key.clone());
                                    }
                                }
                            },
                        );
                    });
                ui.separator();
                ui.checkbox(&mut self.should_import_background, "Import Background");

                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("Import").heading()).clicked() {
                        action = ControlFlow::Break(ImportResult::Import {
                            tilesets: self
                                .map_importing_from
                                .tilesets
                                .iter()
                                .filter(|(k, _)| self.selected_tilesets.contains(*k))
                                .map(|(_, x)| x)
                                .cloned()
                                .collect(),
                            background_color: Some(self.map_importing_from.background_color),
                            background_layers: if self.should_import_background {
                                self.map_importing_from.background_layers.clone()
                            } else {
                                Vec::new()
                            },
                        })
                    }

                    if ui.button(egui::RichText::new("Cancel").heading()).clicked() {
                        action = ControlFlow::Break(ImportResult::Close);
                    }
                });
            });

        action
    }
}
