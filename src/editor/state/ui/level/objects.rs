use std::ops::Deref;

use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::UiAction,
        state::{
            DragData, Editor, EditorTool, ObjectSettings, SelectableEntity, SelectableEntityKind,
        },
        util::{EguiCompatibleVec, EguiTextureHandler, MqCompatibleVec},
        view::UiLevelView,
    },
    map::{MapLayer, MapLayerKind, MapObject, MapObjectKind},
    AnimatedSpriteMetadata, Resources,
};

impl Editor {
    pub(super) fn draw_level_object_placement_overlay(&mut self, view: &UiLevelView) {
        if let Some(settings) = &mut self.object_being_placed {
            let position = view.world_to_screen_pos(settings.position);

            enum PlaceObjectResult {
                Create,
                Close,
                Noop,
            }

            let response = egui::Window::new("Placing object")
                .current_pos(position)
                .collapsible(false)
                .resizable(false)
                .show(view.ctx(), |ui| {
                    let target_layer = self
                        .selected_layer
                        .as_ref()
                        .and_then(|id| self.map_resource.map.layers.get(id))
                        .filter(|layer| layer.kind == MapLayerKind::ObjectLayer);

                    if let Some(target) = target_layer {
                        ui.label(format!("Within layer selection: {}", target.id));
                    } else {
                        ui.label(format!("Select an object layer"));
                    }

                    egui::ComboBox::new("object_kind", "Kind")
                        .selected_text(format!("{}", settings.kind))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut settings.kind,
                                MapObjectKind::Decoration,
                                "Decoration",
                            )
                            .clicked()
                            .then(|| settings.id = None);
                            ui.selectable_value(
                                &mut settings.kind,
                                MapObjectKind::Environment,
                                "Environment",
                            )
                            .clicked()
                            .then(|| settings.id = None);
                            ui.selectable_value(&mut settings.kind, MapObjectKind::Item, "Item")
                                .clicked()
                                .then(|| settings.id = None);
                        });
                    egui::ComboBox::new("object_id", "Id")
                        .selected_text(settings.id.as_deref().unwrap_or("Pick one"))
                        .show_ui(ui, |ui| {
                            let resources = storage::get::<Resources>();

                            match settings.kind {
                                MapObjectKind::Item => resources.items.keys().for_each(|id| {
                                    ui.selectable_value(&mut settings.id, Some(id.clone()), id);
                                }),
                                MapObjectKind::Environment => {
                                    ["sproinger", "crab"].into_iter().for_each(|id| {
                                        ui.selectable_value(
                                            &mut settings.id,
                                            Some(id.to_owned()),
                                            id,
                                        );
                                    })
                                }
                                MapObjectKind::Decoration => {
                                    resources.decoration.keys().for_each(|id| {
                                        ui.selectable_value(&mut settings.id, Some(id.clone()), id);
                                    })
                                }
                            }
                        });

                    ui.horizontal(|ui| {
                        let create_button = ui.add_enabled(
                            settings.id.is_some() && target_layer.is_some(),
                            egui::Button::new("Create"),
                        );
                        if create_button.clicked() {
                            return PlaceObjectResult::Create;
                        }
                        if ui.button("Cancel").clicked() {
                            return PlaceObjectResult::Close;
                        }

                        PlaceObjectResult::Noop
                    })
                });

            let result = response
                .and_then(|r| {
                    settings.position = view.screen_to_world_pos(r.response.rect.min);

                    r.inner.map(|r| r.inner)
                })
                .unwrap_or(PlaceObjectResult::Noop);

            match result {
                PlaceObjectResult::Create => {
                    let id = settings.id.as_ref().unwrap().clone();
                    let kind = settings.kind;
                    let position = settings.position.into_macroquad();
                    let layer_id = self.selected_layer.as_ref().unwrap().clone();
                    self.apply_action(UiAction::CreateObject {
                        id,
                        kind,
                        position,
                        layer_id,
                    });
                    self.object_being_placed = None;
                }
                PlaceObjectResult::Close => {
                    self.object_being_placed = None;
                }
                PlaceObjectResult::Noop => (),
            }
        }
    }

    pub(super) fn handle_objects(
        &mut self,
        ui: &mut egui::Ui,
        view: &UiLevelView,
        to_select: &mut Option<SelectableEntity>,
    ) {
        for layer in self
            .map_resource
            .map
            .draw_order
            .iter()
            .filter_map(|layer_id| self.map_resource.map.layers.get(layer_id))
        {
            if layer.is_visible {
                self.handle_object_layer(layer, view, ui, to_select);
            }
        }
    }

    fn handle_object_layer(
        &self,
        layer: &MapLayer,
        view: &UiLevelView,
        ui: &mut egui::Ui,
        to_select: &mut Option<SelectableEntity>,
    ) {
        let resources = storage::get::<Resources>();

        let is_selection_being_dragged = matches!(
            &self.selection,
            Some(SelectableEntity {
                drag_data: Some(_),
                ..
            })
        );

        for (object_idx, object) in layer.objects.iter().enumerate().filter(|(idx, _)| {
            let is_selection = matches!(
                &self.selection,
                Some(SelectableEntity {
                    kind: SelectableEntityKind::Object {
                        index,
                        layer_id
                    },
                    ..
                }) if index == idx && layer_id == &layer.id
            );

            !(is_selection && is_selection_being_dragged)
        }) {
            let (dest, is_valid) = draw_object(
                object,
                object.position.into_egui().to_pos2(),
                view,
                &resources,
                1.,
            );

            let response = &view.response;
            let is_hovered = response.hovered()
                && ui
                    .input()
                    .pointer
                    .hover_pos()
                    .map(|hover_pos| dest.contains(hover_pos))
                    .unwrap_or(false)
                && self.selected_tool == EditorTool::Cursor;

            if is_hovered && !is_selection_being_dragged {
                self.show_object_info_tooltip(ui.ctx(), object, is_valid);

                if response.drag_started() || response.clicked() {
                    let click_pos = ui.input().pointer.interact_pos().unwrap();

                    *to_select = Some(SelectableEntity {
                        kind: SelectableEntityKind::Object {
                            index: object_idx,
                            layer_id: layer.id.clone(),
                        },
                        drag_data: response.drag_started().then(|| DragData {
                            cursor_offset: dest.min - click_pos,
                            new_pos: object.position.into_egui().to_pos2(),
                        }),
                    });
                    dbg!("Selected object");
                }
                view.painter().add(egui::Shape::rect_stroke(
                    dest,
                    egui::Rounding::none(),
                    egui::Stroke::new(1., egui::Color32::GRAY),
                ));
            }
        }
    }

    fn show_object_info_tooltip(
        &self,
        egui_ctx: &egui::Context,
        object: &MapObject,
        is_valid: bool,
    ) {
        egui::containers::show_tooltip_at_pointer(egui_ctx, egui::Id::new("object info"), |ui| {
            ui.set_max_width(200.);
            ui.vertical_centered(|ui| {
                ui.heading(&object.id);
                ui.label(egui::RichText::new(format!("{}", object.kind)).small());
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new("Position: ").weak());
                ui.label(
                    egui::RichText::new(format!("({}, {})", object.position.x, object.position.y))
                        .monospace(),
                );
            });
            if !is_valid {
                ui.label(
                    egui::RichText::new(
                        "Object is not valid (i.e. has no valid object or texture ID)",
                    )
                    .color(egui::Color32::RED),
                );
            }
        });
    }
}

pub fn draw_object(
    object: &crate::map::MapObject,
    position: egui::Pos2,
    view: &UiLevelView,
    resources: &impl Deref<Target = Resources>,
    opacity: f32,
) -> (egui::Rect, bool) {
    const FULL_UV: egui::Rect = egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.));

    let draw_object = |texture_id: egui::TextureId,
                       offset: macroquad::math::Vec2,
                       dest_size: egui::Vec2,
                       uv: egui::Rect,
                       tint: egui::Color32|
     -> egui::Rect {
        let position_in_lvl = position + offset.into_egui();

        let dest = egui::Rect::from_min_size(
            view.world_to_screen_pos(position_in_lvl),
            dest_size * view.view.scale,
        );

        let mut mesh = egui::Mesh::with_texture(texture_id);
        mesh.add_rect_with_uv(dest, uv, tint);
        view.painter().add(egui::Shape::mesh(mesh));
        dest
    };

    let draw_invalid_object = || -> egui::Rect {
        let texture_id = resources
            .textures
            .get("object_error_icon")
            .unwrap()
            .texture
            .egui_id();
        let dest_size = egui::vec2(32., 32.);
        let uv = egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.));

        draw_object(
            texture_id,
            (0., 0.).into(),
            dest_size,
            uv,
            egui::Color32::WHITE.linear_multiply(opacity),
        )
    };

    let draw_animated_sprite = |sprite: &AnimatedSpriteMetadata, row: Option<u32>| -> egui::Rect {
        if let Some(texture_res) = resources.textures.get(&sprite.texture_id) {
            let tint = sprite
                .tint
                .map(|color| {
                    let [r, g, b, a]: [u8; 4] = color.into();
                    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
                })
                .unwrap_or(egui::Color32::WHITE);

            let texture_id = texture_res.texture.egui_id();
            let texture_size =
                egui::vec2(texture_res.texture.width(), texture_res.texture.height());
            let frame_size = texture_res.frame_size().into_egui();

            let dest_size = sprite.scale.map(|s| s * frame_size).unwrap_or(frame_size);

            let uv = row
                .map(|row| {
                    egui::Rect::from_min_size(
                        (egui::vec2(0.0, row as f32 * frame_size.y) / texture_size).to_pos2(),
                        frame_size / texture_size,
                    )
                })
                .unwrap_or(FULL_UV);

            draw_object(
                texture_id,
                sprite.offset,
                dest_size,
                uv,
                tint.linear_multiply(opacity),
            )
        } else {
            // Invalid texture ID
            draw_invalid_object()
        }
    };

    let dest;
    let is_valid;
    match object.kind {
        MapObjectKind::Decoration => {
            if let Some(meta) = resources.decoration.get(&object.id) {
                dest = draw_animated_sprite(
                    &meta.sprite,
                    meta.sprite.animations.first().map(|a| a.row),
                );
                is_valid = true;
            } else {
                // Invalid object ID
                dest = draw_invalid_object();
                is_valid = false;
            }
        }

        MapObjectKind::Item => {
            if let Some(meta) = resources.items.get(&object.id) {
                dest = draw_animated_sprite(
                    &meta.sprite,
                    Some(
                        meta.sprite
                            .animations
                            .iter()
                            .find(|&a| a.id == *crate::player::IDLE_ANIMATION_ID)
                            .map(|a| a.row)
                            .unwrap_or_default(),
                    ),
                );
                is_valid = true;
            } else {
                // Invalid object ID
                dest = draw_invalid_object();
                is_valid = false;
            }
        }
        MapObjectKind::Environment => {
            if &object.id == "sproinger" {
                let texture_res = resources.textures.get("sproinger").unwrap();

                let texture_id = texture_res.texture.egui_id();
                let texture_size =
                    egui::vec2(texture_res.texture.width(), texture_res.texture.height());
                let dest_size = texture_res
                    .meta
                    .frame_size
                    .map(macroquad::math::Vec2::into_egui)
                    .unwrap_or_else(|| texture_size);
                let uv = egui::Rect::from_min_size(
                    egui::Vec2::ZERO.to_pos2(),
                    texture_res.frame_size().into_egui() / texture_size,
                );

                dest = draw_object(
                    texture_id,
                    (0., 0.).into(),
                    dest_size,
                    uv,
                    egui::Color32::WHITE.linear_multiply(opacity),
                );
                is_valid = true;
            } else {
                // Invalid object ID
                dest = draw_invalid_object();
                is_valid = false;
            }
        }
    };

    (dest, is_valid)
}
