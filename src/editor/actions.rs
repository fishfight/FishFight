use macroquad::prelude::*;

use core::error::Result;

use crate::map::{Map, MapLayerKind, MapTileset};
use crate::map::{MapBackgroundLayer, MapObjectKind};

use super::state::{EditorTool, SelectableEntity};

/// An enum describing a set of undoable actions done within the level editor.
/// If you need to perform multiple actions in one call, use the `Batch` variant.
#[derive(Debug, Clone)]
pub enum UiAction {
    Batch(Vec<UiAction>),
    Undo,
    Redo,
    SelectTool(EditorTool),
    UpdateBackground {
        color: Color,
        layers: Vec<MapBackgroundLayer>,
    },
    SelectTile {
        id: u32,
        tileset_id: String,
    },
    UpdateTileAttributes {
        index: usize,
        layer_id: String,
        attributes: Vec<String>,
    },
    SelectLayer(String),
    SetLayerDrawOrderIndex {
        id: String,
        index: usize,
    },
    CreateLayer {
        id: String,
        kind: MapLayerKind,
        has_collision: bool,
        index: Option<usize>,
    },
    DeleteLayer(String),
    UpdateLayer {
        id: String,
        is_visible: bool,
    },
    SelectTileset(String),
    Import {
        tilesets: Vec<MapTileset>,
        background_color: Option<Color>,
        background_layers: Vec<MapBackgroundLayer>,
    },
    CreateTileset {
        id: String,
        texture_id: String,
    },
    DeleteTileset(String),
    UpdateTileset {
        id: String,
        texture_id: String,
        autotile_mask: Vec<bool>,
    },
    CreateObject {
        id: String,
        kind: MapObjectKind,
        position: Vec2,
        layer_id: String,
    },
    DeleteObject {
        index: usize,
        layer_id: String,
    },
    UpdateObject {
        layer_id: String,
        index: usize,
        id: String,
        kind: MapObjectKind,
    },
    MoveObject {
        layer_id: String,
        index: usize,
        position: Vec2,
    },
    CreateSpawnPoint(Vec2),
    DeleteSpawnPoint(usize),
    MoveSpawnPoint {
        index: usize,
        position: Vec2,
    },
    PlaceTile {
        id: u32,
        layer_id: String,
        tileset_id: String,
        coords: UVec2,
    },
    RemoveTile {
        layer_id: String,
        coords: UVec2,
    },
    CreateMap {
        name: String,
        description: Option<String>,
        tile_size: Vec2,
        grid_size: UVec2,
    },
    OpenMap(usize),
    SaveMap {
        name: Option<String>,
    },
    DeleteMap(usize),
}

impl UiAction {
    pub fn batch(actions: &[UiAction]) -> Self {
        Self::Batch(actions.to_vec())
    }

    pub fn then(mut self, action: UiAction) -> Self {
        match &mut self {
            UiAction::Batch(batch) => {
                batch.push(action);
                self
            }
            _ => Self::batch(&[self, action]),
        }
    }

    pub fn then_do(&mut self, action: UiAction) {
        match self {
            UiAction::Batch(batch) => {
                batch.push(action);
            }
            _ => {
                let mut temp = Self::Batch(Vec::with_capacity(2));
                std::mem::swap(&mut temp, self);
                self.then_do(temp);
                self.then_do(action);
            }
        }
    }
}

pub trait UiActionExt {
    fn then(self, action: Option<UiAction>) -> Option<UiAction>;
    fn then_some(self, action: UiAction) -> Option<UiAction>;
    fn then_do(&mut self, action: Option<UiAction>);
    fn then_do_some(&mut self, action: UiAction);
}

impl UiActionExt for Option<UiAction> {
    fn then(self, action: Option<UiAction>) -> Option<UiAction> {
        match action {
            Some(action) => self.then_some(action),
            None => self,
        }
    }

    fn then_some(self, action: UiAction) -> Option<UiAction> {
        match self {
            Some(self_action) => Some(self_action.then(action)),
            None => Some(action),
        }
    }

    fn then_do(&mut self, action: Option<UiAction>) {
        if let Some(action) = action {
            self.then_do_some(action);
        }
    }

    fn then_do_some(&mut self, action: UiAction) {
        match self {
            Some(self_action) => self_action.then_do(action),
            None => *self = Some(action),
        }
    }
}

/// All actions that modify map data should implement this trait
pub trait UndoableAction {
    fn apply_to(&mut self, _map: &mut Map) -> Result<()>;

    fn undo(&mut self, _map: &mut Map) -> Result<()>;

    /// Implement this for actions that can be redundant (ie. no change will take place if it is applied).
    /// This is to avoid history being filled up with repeat actions if user is holding input down
    /// for a long time, for example.
    /// This should not be used to circumvent bugs and errors, however. It is meant to stop the same
    /// action from firing several times in a row, for example from holding mouse down on the map
    /// (placing multiple tiles, with the same id, to the same coords).
    /// Edge cases, like an action wanting to delete a layer that does not exist, should be handled
    /// with errors, in stead (basically, things like that shouldn't happen, as it should be prevented
    /// at a higher level)
    fn is_redundant(&self, _map: &Map) -> bool {
        false
    }
}

mod update_background;
pub use update_background::*;

mod set_layer_draw_order_index;
pub use set_layer_draw_order_index::*;

mod update_tile_attributes;
pub use update_tile_attributes::*;

mod create_layer;
pub use create_layer::*;

mod delete_layer;
pub use delete_layer::*;

mod update_layer;
pub use update_layer::*;

mod import;
pub use import::*;

mod create_tileset;
pub use create_tileset::*;

mod delete_tileset;
pub use delete_tileset::*;

mod update_tileset;
pub use update_tileset::*;

mod create_object;
pub use create_object::*;

mod delete_object;
pub use delete_object::*;

mod update_object;
pub use update_object::*;

mod move_object;
pub use move_object::*;

mod create_spawn_point;
pub use create_spawn_point::*;

mod delete_spawn_point;
pub use delete_spawn_point::*;

mod move_spawn_point;
pub use move_spawn_point::*;

mod place_tile;
pub use place_tile::*;

mod remove_tile;
pub use remove_tile::*;
