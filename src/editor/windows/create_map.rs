use std::{ops::ControlFlow, path::Path};

use macroquad::prelude::{collections::storage, ivec2, uvec2, IVec2, UVec2};

use crate::{
    resources::{map_name_to_filename, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION},
    Resources,
};

pub struct CreateMapWindow {
    name: String,
    description: String,
    tile_size: UVec2,
    grid_size: UVec2,
}
pub enum CreateMapResult {
    Create {
        name: String,
        description: String,
        tile_size: UVec2,
        grid_size: UVec2,
    },
    Close,
}

impl Default for CreateMapWindow {
    fn default() -> Self {
        Self {
            name: "Unnamed Map".to_owned(),
            description: String::new(),
            tile_size: uvec2(32, 32),
            grid_size: uvec2(100, 75),
        }
    }
}

impl CreateMapWindow {
    pub fn ui(&mut self, egui_ctx: &egui::Context) -> ControlFlow<CreateMapResult> {
        let mut action = ControlFlow::Continue(());

        egui::Window::new("Create Map")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(egui_ctx, |ui| {
                ui.text_edit_singleline(&mut self.name);

                {
                    let map_export_path = {
                        let resources = storage::get::<Resources>();
                        Path::new(&resources.assets_dir).join(MAP_EXPORTS_DEFAULT_DIR)
                    };

                    let path = map_export_path
                        .join(map_name_to_filename(&self.name))
                        .with_extension(MAP_EXPORTS_EXTENSION);

                    ui.label(path.to_string_lossy().as_ref());
                }

                ui.add(egui::TextEdit::multiline(&mut self.description).hint_text("Description"));
                ui.horizontal(|ui| {
                    ui.label("Tile size: ");
                    ui.add(egui::DragValue::new(&mut self.tile_size.x).clamp_range(1..=512u32));
                    ui.label("*");
                    ui.add(egui::DragValue::new(&mut self.tile_size.y).clamp_range(1..=512u32));
                });
                ui.horizontal(|ui| {
                    ui.label("Grid size: ");
                    ui.add(egui::DragValue::new(&mut self.grid_size.x).clamp_range(1..=512u32));
                    ui.label("*");
                    ui.add(egui::DragValue::new(&mut self.grid_size.y).clamp_range(1..=512u32));
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        action = ControlFlow::Break(CreateMapResult::Create {
                            name: self.name.clone(),
                            description: self.description.clone(),
                            grid_size: self.grid_size,
                            tile_size: self.tile_size,
                        });
                    }
                    if ui.button("Cancel").clicked() {
                        action = ControlFlow::Break(CreateMapResult::Close);
                    }
                })
            });

        action
    }
}
