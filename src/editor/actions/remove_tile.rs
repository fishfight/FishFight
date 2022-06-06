use core::error::ErrorKind;

use core::error::Error;

use macroquad::prelude::UVec2;

use crate::map::MapLayerKind;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapTile;

#[derive(Debug)]
pub struct RemoveTile {
    layer_id: String,
    coords: UVec2,
    tile: Option<MapTile>,
}

impl RemoveTile {
    pub fn new(layer_id: String, coords: UVec2) -> Self {
        RemoveTile {
            layer_id,
            coords,
            tile: None,
        }
    }
}

impl UndoableAction for RemoveTile {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if let Some(tile) = layer.tiles.remove(i as usize) {
                    self.tile = Some(tile);

                    layer.tiles.insert(i, None);
                } else {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"RemoveTile: No tile at the specified coords, in the specified layer. Undo was probably called on an action that was never applied"));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"RemoveTile: The specified layer is not a tile layer",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"RemoveTile: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if let Some(old_tile) = self.tile.take() {
                    if let Some(tile) = layer.tiles.get_mut(i) {
                        *tile = Some(old_tile);
                    } else {
                        return Err(Error::new_const(ErrorKind::EditorAction, &"RemoveTile (Undo): No tile found vec entry in map at the index stored in action (this should not be possible, as the entry should be a `None` if the tile was empty)"));
                    }
                } else {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"RemoveTile (Undo): No tile stored in action. Undo was probably called on an action that was never applied"));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"RemoveTile (Undo): The specified layer is not a tile layer",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"RemoveTile (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        let i = map.to_index(self.coords);
        if let Some(layer) = map.layers.get(&self.layer_id) {
            if let Some(tile) = layer.tiles.get(i) {
                return tile.is_none();
            }
        }

        false
    }
}
