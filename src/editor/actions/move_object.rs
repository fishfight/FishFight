use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use macroquad::prelude::Vec2;

use crate::map::Map;

use super::UndoableAction;

#[derive(Debug)]
pub struct MoveObject {
    layer_id: String,
    index: usize,
    position: Vec2,
    old_position: Option<Vec2>,
}

impl MoveObject {
    pub fn new(layer_id: String, index: usize, position: Vec2) -> Self {
        MoveObject {
            layer_id,
            index,
            position,
            old_position: None,
        }
    }
}

impl UndoableAction for MoveObject {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(object) = layer.objects.get_mut(self.index) {
                self.old_position = Some(object.position);

                object.position = self.position;
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"MoveObject: The specified object index does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"MoveObject: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(object) = self.old_position.take() {
                layer.objects[self.index].position = object;
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"MoveObject: No old position found on action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"MoveObject (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }
}
