use std::ops::ControlFlow;

use macroquad::prelude::collections::storage;

use crate::{map::Map, resources::TextureKind, Resources};

pub struct CreateTilesetWindow {
    tileset_name: String,
    texture_options: Vec<String>,
    texture_idx: usize,
}

impl Default for CreateTilesetWindow {
    fn default() -> Self {
        Self {
            tileset_name: "Unnamed Tileset".to_owned(),
            texture_options: storage::get::<Resources>()
                .textures
                .iter()
                .filter_map(|(id, resource)| {
                    matches!(resource.meta.kind, Some(TextureKind::Tileset)).then(|| id.clone())
                })
                .collect(),
            texture_idx: 0,
        }
    }
}
pub enum CreateTilesetResult {
    Create {
        tileset_name: String,
        texture: String,
    },
    Close,
}

impl CreateTilesetWindow {
    pub fn ui(&mut self, egui_ctx: &egui::Context, map: &Map) -> ControlFlow<CreateTilesetResult> {
        let mut action = ControlFlow::Continue(());

        egui::Window::new("Create Tileset").show(egui_ctx, |ui| {
            ui.text_edit_singleline(&mut self.tileset_name).changed();
            egui::ComboBox::new("tileset texture", "Texture")
                .selected_text(&self.texture_options[self.texture_idx])
                .show_ui(ui, |ui| {
                    for (texture_idx, texture) in self.texture_options.iter().enumerate() {
                        ui.selectable_value(&mut self.texture_idx, texture_idx, texture);
                    }
                });

            let can_create_tileset = !map.tilesets.contains_key(&self.tileset_name);
            if !can_create_tileset {
                ui.label(
                    egui::RichText::new("Tileset names must be unique").color(egui::Color32::RED),
                );
            }
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_create_tileset, egui::Button::new("Create"))
                    .clicked()
                {
                    action = ControlFlow::Break(CreateTilesetResult::Create {
                        tileset_name: self.tileset_name.clone(),
                        texture: self.texture_options[self.texture_idx].clone(),
                    });
                }
                if ui.button("Cancel").clicked() {
                    action = ControlFlow::Break(CreateTilesetResult::Close)
                }
            })
        });

        action
    }
}
