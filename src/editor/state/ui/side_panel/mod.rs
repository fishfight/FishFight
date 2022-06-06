mod layers;
mod objects;
mod tileset;

use crate::map::MapLayerKind;

use super::super::Editor;

pub const TABLE_ROW_HEIGHT: f32 = 18.0;

impl Editor {
    pub(super) fn draw_side_panel(&mut self, egui_ctx: &egui::Context) {
        egui::SidePanel::new(egui::containers::panel::Side::Right, "Side panel").show(
            egui_ctx,
            |ui| {
                self.draw_layer_info(ui);

                ui.separator();

                if let Some(selected_layer_id) = self.selected_layer.as_ref() {
                    if let Some(layer) = self.map_resource.map.layers.get(selected_layer_id) {
                        match layer.kind {
                            MapLayerKind::TileLayer => {
                                self.draw_tileset_info(ui);
                            }
                            MapLayerKind::ObjectLayer => {
                                self.draw_object_info(ui, layer);
                            }
                        }
                    }
                }
            },
        );
    }
}
