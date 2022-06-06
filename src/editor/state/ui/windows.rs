use std::ops::ControlFlow;

use macroquad::prelude::collections::storage;

use crate::{
    editor::{
        actions::UiAction,
        state::Editor,
        windows::{
            CreateLayerResult, CreateMapResult, CreateTilesetResult, ImportResult, ImportWindow,
            MenuResult, OpenMapResult, SaveMapResult,
        },
    },
    Resources,
};

impl Editor {
    pub(super) fn draw_windows(&mut self, egui_ctx: &egui::Context) {
        if let Some(window) = &mut self.background_properties_window {
            if let Some(action) = window.ui(egui_ctx, &self.map_resource.map) {
                self.apply_action(action);
            }
        }

        if let Some(window) = &mut self.create_layer_window {
            match window.ui(egui_ctx, &self.map_resource.map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateLayerResult::Create {
                    has_collision,
                    layer_kind,
                    layer_name,
                }) => {
                    self.apply_action(UiAction::CreateLayer {
                        has_collision,
                        kind: layer_kind,
                        index: None,
                        id: layer_name,
                    });
                    self.create_layer_window = None;
                }
                ControlFlow::Break(CreateLayerResult::Close) => {
                    self.create_layer_window = None;
                }
            }
        }

        if let Some(window) = &mut self.create_tileset_window {
            match window.ui(egui_ctx, &self.map_resource.map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateTilesetResult::Create {
                    tileset_name,
                    texture,
                }) => {
                    self.apply_action(UiAction::CreateTileset {
                        id: tileset_name,
                        texture_id: texture,
                    });
                    self.create_tileset_window = None;
                }
                ControlFlow::Break(CreateTilesetResult::Close) => {
                    self.create_tileset_window = None;
                }
            }
        }

        if let Some(window) = &mut self.save_map_window {
            match window.ui(egui_ctx) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(SaveMapResult::Save { name }) => {
                    self.apply_action(UiAction::SaveMap { name: Some(name) });
                    self.save_map_window = None;
                }
                ControlFlow::Break(SaveMapResult::Close) => {
                    self.save_map_window = None;
                }
            }
        } else if let Some(window) = &mut self.create_map_window {
            match window.ui(egui_ctx) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateMapResult::Create {
                    name,
                    description,
                    tile_size,
                    grid_size,
                }) => {
                    self.apply_action(UiAction::CreateMap {
                        name,
                        description: if description.is_empty() {
                            None
                        } else {
                            Some(description)
                        },
                        tile_size: tile_size.as_f32(),
                        grid_size,
                    });
                    self.create_map_window = None;
                }
                ControlFlow::Break(CreateMapResult::Close) => {
                    self.create_map_window = None;
                }
            }
        } else if let Some(window) = &mut self.open_map_window {
            match window.ui(egui_ctx) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(OpenMapResult::Open { map_index }) => {
                    self.apply_action(UiAction::OpenMap(map_index));
                    self.open_map_window = None;
                }
                ControlFlow::Break(OpenMapResult::Import { map_index }) => {
                    self.import_window = Some(ImportWindow::new(
                        storage::get::<Resources>().maps[map_index].map.clone(),
                    ));
                    self.open_map_window = None;
                }
                ControlFlow::Break(OpenMapResult::Close) => {
                    self.open_map_window = None;
                }
            }
        } else if let Some(window) = &mut self.import_window {
            match window.ui(egui_ctx) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(ImportResult::Import {
                    background_color,
                    background_layers,
                    tilesets,
                }) => {
                    self.apply_action(UiAction::Import {
                        background_color,
                        background_layers,
                        tilesets,
                    });
                    self.import_window = None;
                }
                ControlFlow::Break(ImportResult::Close) => {
                    self.import_window = None;
                }
            }
        } else if let Some(window) = &mut self.menu_window {
            match window.ui(egui_ctx, self.map_resource.meta.is_user_map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(MenuResult::OpenCreateMapWindow) => {
                    self.create_map_window = Some(Default::default());
                    self.menu_window = None;
                }
                ControlFlow::Break(MenuResult::OpenLoadMapWindow) => {
                    self.open_map_window = Some(Default::default());
                    self.menu_window = None;
                }
                ControlFlow::Break(MenuResult::OpenSaveMapWindow) => {
                    self.save_map_window = Some(Default::default());
                    self.menu_window = None;
                }
                ControlFlow::Break(MenuResult::SaveMap) => {
                    self.apply_action(UiAction::SaveMap { name: None });
                }
                ControlFlow::Break(MenuResult::ExitToMainMenu) => {
                    crate::exit_to_main_menu();
                }
                ControlFlow::Break(MenuResult::QuitToDesktop) => {
                    crate::quit_to_desktop();
                }
            }
        }
    }
}
