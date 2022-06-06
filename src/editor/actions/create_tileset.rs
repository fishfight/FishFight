use core::error::ErrorKind;

use core::error::Error;

use crate::map::MapTileset;

use crate::Resources;

use macroquad::experimental::collections::storage;
use macroquad::prelude::uvec2;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

#[derive(Debug)]
pub struct CreateTileset {
    id: String,
    texture_id: String,
}

impl CreateTileset {
    pub fn new(id: String, texture_id: String) -> Self {
        CreateTileset { id, texture_id }
    }
}

impl UndoableAction for CreateTileset {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        let resources = storage::get::<Resources>();
        if let Some(texture_entry) = resources.textures.get(&self.texture_id).cloned() {
            let texture_size = uvec2(
                texture_entry.texture.width() as u32,
                texture_entry.texture.height() as u32,
            );

            let mut first_tile_id = 1;
            for tileset in map.tilesets.values() {
                let next_tile_id = tileset.first_tile_id + tileset.tile_cnt;
                if next_tile_id > first_tile_id {
                    first_tile_id = next_tile_id;
                }
            }

            let tileset = MapTileset::new(
                &self.id,
                &self.texture_id,
                texture_size,
                map.tile_size,
                first_tile_id,
            );

            map.tilesets.insert(self.id.clone(), tileset);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateTileset: The specified texture does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if map.tilesets.remove(&self.id).is_none() {
            return Err(Error::new_const(ErrorKind::EditorAction, &"CreateTileset (Undo): The specified tileset does not exist. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}
