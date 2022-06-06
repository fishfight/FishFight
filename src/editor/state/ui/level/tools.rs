use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::UiAction,
        state::EditorTool,
        util::{EguiCompatibleVec, EguiTextureHandler},
        view::UiLevelView,
        Editor,
    },
    Resources,
};

impl Editor {
    pub(super) fn handle_tool(&mut self, view: &UiLevelView, cursor_tile_pos: egui::Pos2) {
        match self.selected_tool {
            EditorTool::TilePlacer => self.handle_tile_placement(view, cursor_tile_pos),
            // TODO: Spawnpoint placement overlay
            // TODO: Object placement overlay
            _ => (),
        };
    }

    fn handle_tile_placement(&mut self, view: &UiLevelView, cursor_tile_pos: egui::Pos2) {
        let tile_size = self.map_resource.map.tile_size.into_egui();

        if cursor_tile_pos.x >= 0. && cursor_tile_pos.y >= 0. {
            if let (Some(selected_tile), Some(selected_layer)) =
                (&self.tile_palette, &self.selected_layer)
            {
                let tileset = &self.map_resource.map.tilesets[&selected_tile.tileset];
                let texture_id = storage::get::<Resources>().textures[&tileset.texture_id]
                    .texture
                    .egui_id();
                let tileset_uv_tile_size = egui::Vec2::splat(1.)
                    / egui::vec2(tileset.grid_size.x as f32, tileset.grid_size.y as f32);
                let tileset_x =
                    (selected_tile.tile_id % tileset.grid_size.x) as f32 * tileset_uv_tile_size.x;
                let tileset_y =
                    (selected_tile.tile_id / tileset.grid_size.x) as f32 * tileset_uv_tile_size.y;
                let uv = egui::Rect::from_min_size(
                    egui::Pos2 {
                        x: tileset_x,
                        y: tileset_y,
                    },
                    tileset_uv_tile_size,
                );

                let mut tile_mesh = egui::Mesh::with_texture(texture_id);
                tile_mesh.add_rect_with_uv(
                    egui::Rect::from_min_size(
                        view.world_to_screen_pos((cursor_tile_pos.to_vec2() * tile_size).to_pos2()),
                        tile_size * view.view.scale,
                    ),
                    uv,
                    egui::Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 200),
                );

                view.painter().add(egui::Shape::mesh(tile_mesh));

                if view.response.clicked() || view.response.dragged() {
                    let input = view.ctx().input();
                    if input.pointer.button_down(egui::PointerButton::Primary) {
                        let id = selected_tile.tile_id;
                        let layer_id = selected_layer.clone();
                        let tileset_id = selected_tile.tileset.clone();
                        self.apply_action(UiAction::PlaceTile {
                            id,
                            layer_id,
                            tileset_id,
                            coords: macroquad::math::UVec2::new(
                                cursor_tile_pos.x as u32,
                                cursor_tile_pos.y as u32,
                            ),
                        });
                    } else if input.pointer.button_down(egui::PointerButton::Secondary) {
                        let layer_id = selected_layer.clone();
                        self.apply_action(UiAction::RemoveTile {
                            layer_id,
                            coords: macroquad::math::UVec2::new(
                                cursor_tile_pos.x as u32,
                                cursor_tile_pos.y as u32,
                            ),
                        });
                    }
                }
            }
        }
    }
}
