use egui_extras::Size;

use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::Editor,
    },
    map::MapLayerKind,
};

use super::TABLE_ROW_HEIGHT;

impl Editor {
    pub(super) fn draw_layer_info(&mut self, ui: &mut egui::Ui) {
        ui.heading("Layers");
        self.draw_layer_list(ui); // Draw layer list
        self.draw_layer_utils(ui); // Draw layer util buttons (+ - Up Down)
    }

    fn draw_layer_list(&mut self, ui: &mut egui::Ui) {
        let mut action = None;

        ui.group(|ui| {
            egui_extras::TableBuilder::new(ui)
                .column(Size::exact(40.0))
                .column(Size::remainder().at_least(100.0))
                .column(Size::exact(70.0))
                .striped(true)
                .sense(egui::Sense::click())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Type");
                    });
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Visibility");
                    });
                })
                .body(|body| {
                    body.rows(
                        TABLE_ROW_HEIGHT,
                        self.map_resource.map.draw_order.len(),
                        |row_index, mut row| {
                            let layer_name = &self.map_resource.map.draw_order[row_index];
                            let layer = &self.map_resource.map.layers[layer_name];
                            let mut clicked = false;
                            let is_selected = self.selected_layer.as_ref() == Some(layer_name);

                            clicked |= row
                                .col(|ui| {
                                    let mut label = egui::RichText::new(match layer.kind {
                                        MapLayerKind::TileLayer => "T",
                                        MapLayerKind::ObjectLayer => "O",
                                    });

                                    if is_selected {
                                        label = label.strong();
                                    }

                                    ui.label(label);
                                })
                                .clicked();
                            clicked |= row
                                .col(|ui| {
                                    let mut label = egui::RichText::new(layer_name);

                                    if is_selected {
                                        label = label.strong();
                                    }

                                    ui.label(label);
                                })
                                .clicked();
                            let mut is_visible = layer.is_visible;
                            clicked |= row
                                .col(|ui| {
                                    let mut label = egui::RichText::new("Visible");

                                    if is_selected {
                                        label = label.strong();
                                    }

                                    if ui.checkbox(&mut is_visible, label).clicked() {
                                        action.then_do_some(UiAction::UpdateLayer {
                                            id: layer_name.clone(),
                                            is_visible,
                                        });
                                    }
                                })
                                .clicked();

                            if clicked {
                                let layer_name = layer_name.clone();
                                self.apply_action(UiAction::SelectLayer(layer_name));
                            }
                        },
                    );
                });
        });

        if let Some(action) = action {
            self.apply_action(action);
        }
    }

    fn draw_layer_utils(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("+").clicked() {
                self.create_layer_window = Some(Default::default());
            }

            ui.add_enabled_ui(self.selected_layer.is_some(), |ui| {
                if ui.button("-").clicked() {
                    let layer_id = self.selected_layer.as_ref().unwrap().clone();
                    self.apply_action(UiAction::DeleteLayer(layer_id));
                }
                let selected_layer_idx = self
                    .selected_layer
                    .as_ref()
                    .and_then(|layer| {
                        self.map_resource
                            .map
                            .draw_order
                            .iter()
                            .enumerate()
                            .find(|(_idx, id)| &layer == id)
                            .map(|(idx, _)| idx)
                    })
                    .unwrap_or(usize::MAX - 1);

                if ui
                    .add_enabled(selected_layer_idx > 0, egui::Button::new("Up"))
                    .clicked()
                {
                    let id = self.selected_layer.as_ref().unwrap().clone();
                    self.apply_action(UiAction::SetLayerDrawOrderIndex {
                        id,
                        index: selected_layer_idx - 1,
                    });
                }

                if ui
                    .add_enabled(
                        selected_layer_idx + 1 < self.map_resource.map.draw_order.len(),
                        egui::Button::new("Down"),
                    )
                    .clicked()
                {
                    let id = self.selected_layer.as_ref().unwrap().clone();
                    self.apply_action(UiAction::SetLayerDrawOrderIndex {
                        id,
                        index: selected_layer_idx + 1,
                    });
                }
            });
        });
    }
}
