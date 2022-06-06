use core::error::ErrorKind;

use core::error::Error;

use macroquad::prelude::UVec2;

use crate::map::MapLayerKind;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapTile;

#[derive(Debug)]
pub struct PlaceTile {
    id: u32,
    layer_id: String,
    tileset_id: String,
    coords: UVec2,
    replaced_tile: Option<MapTile>,
}

impl PlaceTile {
    pub fn new(id: u32, layer_id: String, tileset_id: String, coords: UVec2) -> Self {
        PlaceTile {
            id,
            layer_id,
            tileset_id,
            coords,
            replaced_tile: None,
        }
    }
}

impl UndoableAction for PlaceTile {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = map.tilesets.get(&self.tileset_id) {
            let texture_id = tileset.texture_id.clone();
            let texture_coords = tileset.get_texture_coords(self.id);

            let i = map.to_index(self.coords);

            if let Some(layer) = map.layers.get_mut(&self.layer_id) {
                if let MapLayerKind::TileLayer = layer.kind {
                    self.replaced_tile = layer.tiles.remove(i as usize);

                    let tile = MapTile {
                        tile_id: self.id,
                        tileset_id: self.tileset_id.clone(),
                        texture_id,
                        texture_coords,
                        attributes: vec![],
                    };

                    layer.tiles.insert(i as usize, Some(tile));
                } else {
                    return Err(Error::new_const(
                        ErrorKind::EditorAction,
                        &"PlaceTile: The specified layer is not a tile layer",
                    ));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"PlaceTile: The specified layer does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"PlaceTile: The specified tileset does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if layer.tiles.get(i as usize).is_none() {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"PlaceTile (Undo): No tile at the specified coords, in the specified layer. Undo was probably called on an action that was never applied"));
                }

                if let Some(tile) = self.replaced_tile.take() {
                    layer.tiles[i as usize] = Some(tile);
                } else {
                    layer.tiles[i as usize] = None;
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"PlaceTile (Undo): The specified layer is not a tile layer",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"PlaceTile (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        if let Some(layer) = map.layers.get(&self.layer_id) {
            let i = map.to_index(self.coords);
            if let Some(Some(tile)) = layer.tiles.get(i) {
                return tile.tileset_id == self.tileset_id && tile.tile_id == self.id;
            }
        }

        false
    }
}
