use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

#[derive(Debug)]
pub struct UpdateTileAttributes {
    index: usize,
    layer_id: String,
    attributes: Vec<String>,
    old_attributes: Option<Vec<String>>,
}

impl UpdateTileAttributes {
    pub fn new(index: usize, layer_id: String, attributes: Vec<String>) -> Self {
        UpdateTileAttributes {
            index,
            layer_id,
            attributes,
            old_attributes: None,
        }
    }
}

impl UndoableAction for UpdateTileAttributes {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(Some(tile)) = layer.tiles.get_mut(self.index) {
                self.old_attributes = Some(tile.attributes.clone());
                tile.attributes = self.attributes.clone();
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"UpdateTileAttributes: The specified tile does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTileAttributes: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(Some(tile)) = layer.tiles.get_mut(self.index) {
                if let Some(old_attributes) = self.old_attributes.take() {
                    tile.attributes = old_attributes;
                } else {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTileAttributes (Undo): No old attributes stored in action. Undo was probably called on an action that was never applied"));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"UpdateTileAttributes (Undo): The specified tile does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTileAttributes (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        if let Some(layer) = map.layers.get(&self.layer_id) {
            if let Some(Some(tile)) = layer.tiles.get(self.index) {
                let matching = tile
                    .attributes
                    .iter()
                    .zip(&self.attributes)
                    .filter(|&(a, b)| a == b)
                    .count();
                return matching == self.attributes.len();
            }
        }

        false
    }
}
