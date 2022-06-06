use core::error::ErrorKind;

use core::error::Error;

use macroquad::prelude::Vec2;

use crate::map::MapObject;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapObjectKind;

#[derive(Debug)]
pub struct CreateObject {
    id: String,
    kind: MapObjectKind,
    position: Vec2,
    layer_id: String,
}

impl CreateObject {
    pub fn new(id: String, kind: MapObjectKind, position: Vec2, layer_id: String) -> Self {
        CreateObject {
            id,
            kind,
            position,
            layer_id,
        }
    }
}

impl UndoableAction for CreateObject {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            let object = MapObject::new(&self.id, self.kind, self.position);

            layer.objects.insert(0, object);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateObject: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            layer.objects.remove(0);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateObject (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }
}
