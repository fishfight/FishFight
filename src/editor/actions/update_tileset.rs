use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

#[derive(Debug)]
pub struct UpdateTileset {
    id: String,
    texture_id: String,
    old_texture_id: Option<String>,
    autotile_mask: Vec<bool>,
    old_autotile_mask: Option<Vec<bool>>,
}

impl UpdateTileset {
    pub fn new(id: String, texture_id: String, autotile_mask: Vec<bool>) -> Self {
        UpdateTileset {
            id,
            texture_id,
            old_texture_id: None,
            autotile_mask,
            old_autotile_mask: None,
        }
    }
}

impl UndoableAction for UpdateTileset {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = map.tilesets.get_mut(&self.id) {
            self.old_texture_id = Some(tileset.texture_id.clone());
            tileset.texture_id = self.texture_id.clone();

            if self.autotile_mask.len() != tileset.autotile_mask.len() {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTileset: There is a size mismatch between the actions autotile mask vector and the one on the tileset"));
            }

            self.old_autotile_mask = Some(tileset.autotile_mask.clone());
            tileset.autotile_mask = self.autotile_mask.clone();

            tileset.bitmasks = tileset.get_bitmasks();
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTileset: The specified tileset does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = map.tilesets.get_mut(&self.id) {
            if let Some(old_texture_id) = &self.old_texture_id {
                tileset.texture_id = old_texture_id.clone();
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTileset (Undo): No old texture id stored in action. Undo was probably called on an action that was never applied"));
            }

            if let Some(old_autotile_mask) = &self.old_autotile_mask {
                if old_autotile_mask.len() != tileset.autotile_mask.len() {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTileset (Undo): There is a size mismatch between the actions autotile mask vector and the one on the tileset"));
                }

                tileset.autotile_mask = old_autotile_mask.clone();
                self.old_autotile_mask = None;
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTileset (Undo): No old autotile mask stored in action. Undo was probably called on an action that was never applied"));
            }

            tileset.bitmasks = tileset.get_bitmasks();
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTileset (Undo): The specified tileset does not exist",
            ));
        }

        Ok(())
    }
}
