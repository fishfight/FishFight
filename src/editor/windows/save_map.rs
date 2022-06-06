use std::{ops::ControlFlow, path::Path};

use macroquad::prelude::collections::storage;

use crate::{
    resources::{
        is_valid_map_export_path, map_name_to_filename, MAP_EXPORTS_DEFAULT_DIR,
        MAP_EXPORTS_EXTENSION,
    },
    Resources,
};

#[derive(Default)]
pub struct SaveMapWindow {
    map_name: String,
    overwrite_existing: bool,
}
pub enum SaveMapResult {
    Save { name: String },
    Close,
}

impl SaveMapWindow {
    pub fn ui(&mut self, egui_ctx: &egui::Context) -> ControlFlow<SaveMapResult> {
        let mut action = ControlFlow::Continue(());
        let resources = storage::get::<Resources>();

        egui::Window::new("Save Map")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(egui_ctx, |ui| {
                ui.text_edit_singleline(&mut self.map_name).changed();
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.;
                    ui.label(
                        egui::RichText::new(format!(
                            "{}/{}/",
                            &resources.assets_dir.trim_end_matches('/'),
                            MAP_EXPORTS_DEFAULT_DIR.trim_end_matches('/')
                        ))
                        .weak(),
                    );
                    ui.label(format!(
                        "{}.{}",
                        map_name_to_filename(&self.map_name),
                        MAP_EXPORTS_EXTENSION
                    ));
                });
                ui.checkbox(&mut self.overwrite_existing, "Overwrite Existing");
                ui.horizontal(|ui| {
                    let path = Path::new(MAP_EXPORTS_DEFAULT_DIR)
                        .join(map_name_to_filename(&self.map_name))
                        .with_extension(MAP_EXPORTS_EXTENSION);

                    let save_button = ui.add_enabled(
                        is_valid_map_export_path(path, self.overwrite_existing),
                        egui::Button::new("Save"),
                    );
                    if save_button.clicked() {
                        action = ControlFlow::Break(SaveMapResult::Save {
                            name: self.map_name.clone(),
                        });
                    }
                    if ui.button("Cancel").clicked() {
                        action = ControlFlow::Break(SaveMapResult::Close);
                    }
                });
            });

        action
    }
}
