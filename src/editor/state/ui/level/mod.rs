mod objects;
mod tools;

use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::UiAction,
        state::{DragData, EditorTool, ObjectSettings, SelectableEntity, SelectableEntityKind},
        util::{EguiCompatibleVec, EguiTextureHandler, MqCompatibleVec, Resizable},
        view::UiLevelView, windows::BackgroundPropertiesWindow,
    },
    map::{MapLayerKind, MapObjectKind},
    Resources,
};

use self::objects::draw_object;

use super::super::Editor;

impl Editor {
    pub(super) fn handle_level_view(&mut self, egui_ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_ctx, |ui| {
                let mut view = self.draw_level_tiles(ui);
                let mut clicked_add_object = false;

                view.response = view.response.context_menu(|ui| {
                    if let Some(MapLayerKind::ObjectLayer) = self.selected_layer_type() {
                        if ui.button("Add object").clicked() {
                            clicked_add_object = true;
                            ui.close_menu()
                        }
                    }
                    if ui.button("Edit background").clicked() {
                        self.background_properties_window = Some(Default::default());
                        ui.close_menu();
                    }
                });

                if view.response.dragged_by(egui::PointerButton::Middle) {
                    let drag_delta = egui_ctx.input().pointer.delta();

                    self.level_view.position -= drag_delta.into_macroquad() / view.view.scale;
                }

                if clicked_add_object
                    || (view.response.clicked() && self.selected_tool == EditorTool::ObjectPlacer)
                {
                    let position = view
                        .screen_to_world_pos(view.ctx().input().pointer.interact_pos().unwrap());

                    self.object_being_placed =
                        if let Some(settings) = self.object_being_placed.take() {
                            Some(ObjectSettings {
                                position,
                                ..settings
                            })
                        } else {
                            Some(ObjectSettings {
                                position,
                                kind: MapObjectKind::Item,
                                id: None,
                            })
                        };
                }

                let mut to_select = None;
                self.handle_objects(ui, &view, &mut to_select);
                self.handle_spawnpoints(&view, &mut to_select);
                self.draw_level_overlays(ui, &view);
                // Draw selection last (Special case)
                self.handle_selection(&view, to_select);

                let (width, height) = (
                    view.response.rect.width() as u32,
                    view.response.rect.height() as u32,
                );
                self.level_render_target.resize_if_appropiate(width, height);
            });
    }

    fn handle_selection(&mut self, view: &UiLevelView, to_select: Option<SelectableEntity>) {
        let mut selection_dest = None;

        let selected_anything = to_select.is_some();

        if let Some(to_select) = to_select {
            if let SelectableEntity {
                kind: SelectableEntityKind::Object { layer_id, .. },
                ..
            } = &to_select
            {
                self.selected_layer = Some(layer_id.clone());
                self.selected_tool = EditorTool::Cursor;
            }

            self.selection = Some(to_select);
            dbg!("Selection set: ", &self.selection);
        }

        match self.selection.take() {
            Some(SelectableEntity {
                kind: SelectableEntityKind::Object { index, layer_id },
                mut drag_data,
            }) => {
                let resources = storage::get::<Resources>();
                let object = &self.map_resource.map.layers[&layer_id].objects[index];
                let is_being_dragged;
                let mut action_to_apply = None;

                if let Some(DragData {
                    new_pos,
                    cursor_offset,
                }) = drag_data.take()
                {
                    draw_object(object, new_pos, &view, &resources, 1.);
                    let response = &view.response;

                    if response.dragged_by(egui::PointerButton::Primary) {
                        let cursor_level_pos = view.screen_to_world_pos(
                            view.ctx().input().pointer.interact_pos().unwrap() + cursor_offset,
                        );

                        drag_data = Some(DragData {
                            new_pos: cursor_level_pos,
                            cursor_offset,
                        });
                    }
                    if response.drag_released() {
                        action_to_apply = Some(UiAction::MoveObject {
                            index,
                            layer_id: layer_id.clone(),
                            position: macroquad::math::vec2(new_pos.x, new_pos.y),
                        });
                    }

                    is_being_dragged = true;
                } else {
                    is_being_dragged = false;
                }

                self.selection = Some(SelectableEntity {
                    drag_data,
                    kind: SelectableEntityKind::Object { index, layer_id },
                });

                let (dest, _is_valid) = draw_object(
                    object,
                    object.position.into_egui().to_pos2(),
                    &view,
                    &resources,
                    if is_being_dragged { 0.5 } else { 1.0 },
                );
                selection_dest = Some(dest);

                view.painter().add(egui::Shape::rect_stroke(
                    dest,
                    egui::Rounding::none(),
                    egui::Stroke::new(1., egui::Color32::WHITE),
                ));

                if let Some(action) = action_to_apply {
                    self.apply_action(action);
                }
            }

            Some(SelectableEntity {
                kind: SelectableEntityKind::SpawnPoint { index },
                mut drag_data,
            }) => {
                let resources = storage::get::<Resources>();
                let texture = &storage::get::<Resources>().textures["spawn_point_icon"];
                let texture_id = texture.texture.egui_id();
                let texture_size = texture.meta.size.into_egui();
                let position = self.map_resource.map.spawn_points[index];
                let is_being_dragged;
                let mut action_to_apply = None;

                if let Some(DragData {
                    new_pos,
                    cursor_offset,
                }) = drag_data.take()
                {
                    let dest = egui::Rect::from_min_size(
                        view.world_to_screen_pos(
                            new_pos - egui::vec2(texture_size.x / 2., texture_size.y),
                        ),
                        texture_size,
                    );
                    let mut mesh = egui::Mesh::with_texture(texture_id);
                    mesh.add_rect_with_uv(
                        dest,
                        egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                        egui::Color32::WHITE,
                    );
                    view.painter().add(egui::Shape::mesh(mesh));
                    let response = &view.response;

                    if response.dragged_by(egui::PointerButton::Primary) {
                        let cursor_level_pos = view.screen_to_world_pos(
                            view.ctx().input().pointer.interact_pos().unwrap() + cursor_offset,
                        );

                        drag_data = Some(DragData {
                            new_pos: cursor_level_pos,
                            cursor_offset,
                        });
                    }
                    if response.drag_released() {
                        action_to_apply = Some(UiAction::MoveSpawnPoint {
                            index,
                            position: macroquad::math::vec2(new_pos.x, new_pos.y),
                        });
                    }

                    is_being_dragged = true;
                } else {
                    is_being_dragged = false;
                }

                self.selection = Some(SelectableEntity {
                    drag_data,
                    kind: SelectableEntityKind::SpawnPoint { index },
                });

                let dest = egui::Rect::from_min_size(
                    view.world_to_screen_pos(
                        position.into_egui().to_pos2()
                            - egui::vec2(texture_size.x / 2., texture_size.y),
                    ),
                    texture_size,
                );
                let mut mesh = egui::Mesh::with_texture(texture_id);
                mesh.add_rect_with_uv(
                    dest,
                    egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                    egui::Color32::WHITE.linear_multiply(if is_being_dragged { 0.5 } else { 1.0 }),
                );
                view.painter().add(egui::Shape::mesh(mesh));
                selection_dest = Some(dest);

                view.painter().add(egui::Shape::rect_stroke(
                    dest,
                    egui::Rounding::none(),
                    egui::Stroke::new(1., egui::Color32::WHITE),
                ));

                if let Some(action) = action_to_apply {
                    self.apply_action(action);
                }
            }

            _ => (),
        }

        if !selected_anything
            && view.response.clicked()
            && !selection_dest.map_or(false, |d| {
                d.contains(view.ctx().input().pointer.interact_pos().unwrap())
            })
        {
            self.selection = None;
            dbg!("Selection unset");
        }
    }

    fn handle_spawnpoints(&mut self, view: &UiLevelView, to_select: &mut Option<SelectableEntity>) {
        let texture = &storage::get::<Resources>().textures["spawn_point_icon"];
        let texture_id = texture.texture.egui_id();
        let texture_size = texture.meta.size.into_egui();
        let is_selection_being_dragged = matches!(
            &self.selection,
            Some(SelectableEntity {
                drag_data: Some(_),
                ..
            })
        );

        for (idx, spawnpoint) in self
            .map_resource
            .map
            .spawn_points
            .iter()
            .enumerate()
            .filter(|(idx, _)| {
                let is_selection = matches!(
                    &self.selection,
                    Some(SelectableEntity {
                        kind: SelectableEntityKind::SpawnPoint {
                            index
                        },
                        ..
                    }) if index == idx
                );

                !(is_selection && is_selection_being_dragged)
            })
        {
            // This position is the bottom midpoint of the destination rect
            let pos = view.world_to_screen_pos(spawnpoint.into_egui().to_pos2());

            let dest = egui::Rect::from_min_size(
                pos - egui::vec2(texture_size.x / 2., texture_size.y),
                texture_size,
            );

            let mut mesh = egui::Mesh::with_texture(texture_id);
            mesh.add_rect_with_uv(
                dest,
                egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                egui::Color32::WHITE,
            );
            view.painter().add(egui::Shape::mesh(mesh));

            let is_hovered = view.response.hovered()
                && view
                    .ctx()
                    .input()
                    .pointer
                    .hover_pos()
                    .map_or(false, |hover_pos| dest.contains(hover_pos))
                && self.selected_tool == EditorTool::Cursor;

            if is_hovered && !is_selection_being_dragged {
                view.painter().add(egui::Shape::rect_stroke(
                    dest,
                    egui::Rounding::none(),
                    egui::Stroke::new(1., egui::Color32::GRAY),
                ));

                if view.response.drag_started() || view.response.clicked() {
                    let click_pos = view.ctx().input().pointer.interact_pos().unwrap();

                    *to_select = Some(SelectableEntity {
                        kind: SelectableEntityKind::SpawnPoint { index: idx },
                        drag_data: view.response.drag_started().then(|| DragData {
                            cursor_offset: pos - click_pos,
                            new_pos: spawnpoint.into_egui().to_pos2(),
                        }),
                    });
                }
            }
        }

        if self.selected_tool == EditorTool::SpawnPointPlacer && view.response.clicked() {
            let pos = view
                .screen_to_world_pos(view.ctx().input().pointer.interact_pos().unwrap())
                .to_vec2()
                .into_macroquad();
            self.apply_action(UiAction::CreateSpawnPoint(pos));
        }
    }

    fn draw_level_tiles(&self, ui: &mut egui::Ui) -> UiLevelView {
        let texture_id = self.level_render_target.texture.egui_id();

        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let mut level_mesh = egui::Mesh::with_texture(texture_id);
        level_mesh.add_rect_with_uv(
            response.rect,
            egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
            egui::Color32::WHITE,
        );
        painter.add(egui::Shape::mesh(level_mesh));

        UiLevelView::new(self.level_view, response, painter)
    }

    fn draw_level_overlays(&mut self, ui: &mut egui::Ui, view: &UiLevelView) {
        let level_contains_cursor = ui
            .input()
            .pointer
            .hover_pos()
            .map(|pos| view.response.rect.contains(pos))
            .unwrap_or(false);

        if level_contains_cursor {
            let tile_size = self.map_resource.map.tile_size.into_egui();

            let cursor_screen_pos = ui.input().pointer.interact_pos().unwrap();
            let cursor_px_pos = view.screen_to_world_pos(cursor_screen_pos);
            let cursor_tile_pos = (cursor_px_pos.to_vec2() / tile_size).floor().to_pos2();

            // TODO: Move outside
            self.handle_tool(view, cursor_tile_pos);

            self.draw_level_pointer_pos_overlay(ui, view, cursor_px_pos, cursor_tile_pos);

            self.draw_level_object_placement_overlay(view);
        }
    }

    fn draw_level_pointer_pos_overlay(
        &self,
        ui: &mut egui::Ui,
        view: &UiLevelView,
        cursor_px_pos: egui::Pos2,
        cursor_tile_pos: egui::Pos2,
    ) {
        egui::containers::Area::new("pointer pos overlay")
            .order(egui::Order::Tooltip)
            .fixed_pos(
                view.level_top_left()
                    + egui::vec2(
                        ui.spacing().window_margin.left,
                        ui.spacing().window_margin.top,
                    ),
            )
            .interactable(false)
            .drag_bounds(egui::Rect::EVERYTHING) // disable clip rect
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(&ui.ctx().style())
                    .show(ui, |ui| {
                        ui.label(format!(
                            "Cursor position: ({}, {}) in pixels: ({:.2}, {:.2})",
                            cursor_tile_pos.x, cursor_tile_pos.y, cursor_px_pos.x, cursor_px_pos.y,
                        ))
                    })
                    .inner
            });
    }
}
