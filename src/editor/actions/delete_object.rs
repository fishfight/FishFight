use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapObject;

#[derive(Debug)]
pub struct DeleteObject {
    index: usize,
    layer_id: String,
    object: Option<MapObject>,
}

impl DeleteObject {
    pub fn new(index: usize, layer_id: String) -> Self {
        DeleteObject {
            index,
            layer_id,
            object: None,
        }
    }
}

impl UndoableAction for DeleteObject {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            let object = layer.objects.remove(self.index);
            self.object = Some(object);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteObject: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(object) = self.object.take() {
            if let Some(layer) = map.layers.get_mut(&self.layer_id) {
                layer.objects.insert(self.index, object);
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"DeleteObject (Undo): The specified layer does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteObject (Undo): No object stored in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}
