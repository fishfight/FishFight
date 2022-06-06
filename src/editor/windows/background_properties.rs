use std::ops::ControlFlow;

use macroquad::prelude::{collections::storage, Color};

use crate::{
    editor::actions::{UiAction, UiActionExt},
    map::{Map, MapBackgroundLayer, MapLayerKind},
    resources::TextureKind,
    Resources,
};

pub struct BackgroundPropertiesWindow {
    valid_layer_texture_ids: Vec<String>,
    layer_depth: f32,
    selected_layer: Option<usize>,
}

impl Default for BackgroundPropertiesWindow {
    fn default() -> Self {
        let resources = storage::get::<Resources>();
        Self {
            valid_layer_texture_ids: resources
                .textures
                .values()
                .filter_map(|texture_res| {
                    let mut res = None;

                    if let Some(kind) = texture_res.meta.kind {
                        if kind == TextureKind::Background {
                            res = Some(texture_res.meta.id.clone());
                        }
                    }

                    res
                })
                .collect(),
            layer_depth: Default::default(),
            selected_layer: Default::default(),
        }
    }
}

impl BackgroundPropertiesWindow {
    pub fn ui(&mut self, egui_ctx: &egui::Context, map: &Map) -> Option<UiAction> {
        let mut action = None;

        egui::Window::new("Background Properties").show(egui_ctx, |ui| {
            let color = map.background_color;

            let mut color = egui::Rgba::from_rgba_premultiplied(color.r, color.g, color.b, color.a);

            ui.horizontal(|ui| {
                ui.label("Color: ");
                egui::color_picker::color_edit_button_rgba(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::OnlyBlend,
                );
            });

            let color = Color::new(color.r(), color.g(), color.b(), color.a());

            if color != map.background_color {
                action = Some(UiAction::UpdateBackground {
                    color,
                    layers: map.background_layers.clone(),
                });
            }

            ui.separator();

            ui.horizontal(|ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .column(egui_extras::Size::exact(175.))
                    .body(|body| {
                        body.rows(16., map.background_layers.len(), |idx, mut row| {
                            row.col(|ui| {
                                let response = ui.selectable_label(
                                    self.selected_layer == Some(idx),
                                    format!(
                                        "Layer {} ({})",
                                        idx, map.background_layers[idx].texture_id
                                    ),
                                );

                                if response.clicked() {
                                    self.selected_layer = Some(idx);
                                }
                            });
                        })
                    });

                if let Some(selected_layer_idx) = self.selected_layer {
                    let selected_layer = &map.background_layers[selected_layer_idx];
                    ui.vertical(|ui| {
                        let mut idx_selected = self
                            .valid_layer_texture_ids
                            .iter()
                            .enumerate()
                            .find(|(_, x)| x == &&selected_layer.texture_id)
                            .map(|(i, _)| i)
                            .unwrap();

                        if egui::ComboBox::new("texture", "Texture")
                            .show_index(
                                ui,
                                &mut idx_selected,
                                self.valid_layer_texture_ids.len(),
                                |i| self.valid_layer_texture_ids[i].to_owned(),
                            )
                            .changed()
                        {
                            let mut layers = map.background_layers.clone();
                            layers[selected_layer_idx].texture_id =
                                self.valid_layer_texture_ids[idx_selected].clone();
                            action = Some(UiAction::UpdateBackground {
                                color: map.background_color,
                                layers,
                            });
                        }

                        let mut depth = selected_layer.depth;
                        if ui
                            .horizontal(|ui| {
                                ui.label("Depth: ");
                                ui.add(egui::DragValue::new(&mut depth))
                            })
                            .response
                            .changed()
                        {
                            // FIXME This does not set the depth (maybe doesn't get run)
                            let mut layers = map.background_layers.clone();
                            layers[selected_layer_idx].depth = depth;
                            action = Some(UiAction::UpdateBackground {
                                color: map.background_color,
                                layers,
                            });
                        }

                        // TODO: Delete/Up/Down
                    });
                } else {
                    ui.label("Select a layer on the left");
                }
            });
        });

        action
    }
}
