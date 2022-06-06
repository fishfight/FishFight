use std::{ops::ControlFlow, path::Path};

use macroquad::prelude::{collections::storage, ivec2, uvec2, IVec2, UVec2};

use crate::{
    editor::util::EguiTextureHandler,
    resources::{map_name_to_filename, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION},
    Resources,
};

#[derive(Default)]
pub struct OpenMapWindow {
    selected_idx: Option<usize>,
}
pub enum OpenMapResult {
    Open { map_index: usize },
    Import { map_index: usize },
    Close,
}

impl OpenMapWindow {
    pub fn ui(&mut self, egui_ctx: &egui::Context) -> ControlFlow<OpenMapResult> {
        let mut action = ControlFlow::Continue(());

        egui::Window::new("Open/Import Map")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(egui_ctx, |ui| {
                let resources = storage::get::<Resources>();
                if let Some(idx) = self.selected_idx {
                    if ui.button("< Back").clicked() {
                        self.selected_idx = None;
                    }
                    let map = &resources.maps[idx];
                    let texture = &map.preview;
                    let texture_id = texture.egui_id();
                    let preview_width = 300.;
                    ui.vertical_centered(|ui| {
                        ui.image(
                            texture_id,
                            egui::vec2(
                                preview_width,
                                preview_width / texture.width() * texture.height(),
                            ),
                        );
                    });

                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("Open").heading()).clicked() {
                            action = ControlFlow::Break(OpenMapResult::Open { map_index: idx });
                        }
                        if ui.button(egui::RichText::new("Import").heading()).clicked() {
                            action = ControlFlow::Break(OpenMapResult::Import { map_index: idx });
                        }
                    });
                } else {
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .sense(egui::Sense::click())
                        .column(egui_extras::Size::remainder())
                        .body(|body| {
                            body.rows(18., resources.maps.len(), |idx, mut row| {
                                let col_response = row.col(|ui| {
                                    let map = &resources.maps[idx];
                                    let map_name = &map.meta.name;
                                    let map_path = &map.meta.path;

                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(map_name).strong());
                                        ui.label(
                                            egui::RichText::new(format!("({map_path})")).weak(),
                                        );
                                    });
                                });

                                if col_response.clicked() {
                                    self.selected_idx = Some(idx);
                                }
                            });
                        });

                    if ui.button(egui::RichText::new("Cancel").heading()).clicked() {
                        action = ControlFlow::Break(OpenMapResult::Close);
                    }
                }
            });

        action
    }
}
