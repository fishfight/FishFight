use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapTileset;

#[derive(Debug)]
pub struct DeleteTileset {
    id: String,
    tileset: Option<MapTileset>,
}

impl DeleteTileset {
    pub fn new(id: String) -> Self {
        DeleteTileset { id, tileset: None }
    }
}

impl UndoableAction for DeleteTileset {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = map.tilesets.remove(&self.id) {
            self.tileset = Some(tileset);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteTileset: The specified tileset does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = self.tileset.take() {
            map.tilesets.insert(self.id.clone(), tileset);
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteTileset (Undo): No tileset stored in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}
