use egui::RichText;
use egui_extras::TableBuilder;
use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::{UiAction, UiActionExt},
        state::{EditorTool, SelectableEntity, SelectableEntityKind},
        util::EguiTextureHandler,
    },
    map::MapLayer,
    resources::TextureResource,
    Resources,
};

use super::{Editor, TABLE_ROW_HEIGHT};

impl Editor {
    pub(super) fn draw_object_info(&self, ui: &mut egui::Ui, layer: &MapLayer) {
        egui_extras::TableBuilder::new(ui)
            .column(egui_extras::Size::remainder())
            .striped(true)
            .header(20.0, |mut row| {
                row.col(|ui| {
                    ui.heading("Objects");
                });
            })
            .body(|body| {
                body.rows(
                    TABLE_ROW_HEIGHT,
                    layer.objects.len(),
                    |row_index, mut row| {
                        let object = &layer.objects[row_index];
                        let is_selected = matches!(
                            self.selection.as_ref(),
                            Some(SelectableEntity {
                                kind: SelectableEntityKind::Object {
                                    layer_id,
                                    index
                                },
                                ..
                            }) if layer_id == &layer.id && index == &row_index
                        );
                        
                        row.col(|ui| {
                            if is_selected {
                                ui.label(RichText::new(&object.id).strong());
                            } else {
                                ui.label(&object.id);
                            }
                        });
                    },
                )
            })
    }
}
