use std::ops::ControlFlow;

use crate::map::{Map, MapLayerKind};

pub struct CreateLayerWindow {
    layer_name: String,
    layer_kind: MapLayerKind,
    has_collision: bool,
}
pub enum CreateLayerResult {
    Create {
        layer_name: String,
        layer_kind: MapLayerKind,
        has_collision: bool,
    },
    Close,
}

impl Default for CreateLayerWindow {
    fn default() -> Self {
        Self {
            layer_name: "Unnamed Layer".to_owned(),
            layer_kind: MapLayerKind::TileLayer,
            has_collision: false,
        }
    }
}

impl CreateLayerWindow {
    pub fn ui(&mut self, egui_ctx: &egui::Context, map: &Map) -> ControlFlow<CreateLayerResult> {
        let mut action = ControlFlow::Continue(());

        egui::Window::new("Create Layer").show(egui_ctx, |ui| {
            ui.text_edit_singleline(&mut self.layer_name).changed();
            egui::ComboBox::new("layer type", "Type")
                .selected_text(match self.layer_kind {
                    MapLayerKind::TileLayer => "Tiles",
                    MapLayerKind::ObjectLayer => "Objects",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.layer_kind, MapLayerKind::TileLayer, "Tiles");
                    ui.selectable_value(&mut self.layer_kind, MapLayerKind::ObjectLayer, "Objects");
                });
            if self.layer_kind == MapLayerKind::TileLayer {
                ui.checkbox(&mut self.has_collision, "Collision");
            }
            let can_create_map = !map.layers.contains_key(&self.layer_name);
            if !can_create_map {
                ui.label(
                    egui::RichText::new("Layer names must be unique").color(egui::Color32::RED),
                );
            }
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_create_map, egui::Button::new("Create"))
                    .clicked()
                {
                    action = ControlFlow::Break(CreateLayerResult::Create {
                        layer_name: self.layer_name.clone(),
                        has_collision: self.has_collision,
                        layer_kind: self.layer_kind,
                    });
                }
                if ui.button("Cancel").clicked() {
                    action = ControlFlow::Break(CreateLayerResult::Close)
                }
            })
        });

        action
    }
}
