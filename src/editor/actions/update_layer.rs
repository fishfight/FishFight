use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

#[derive(Debug)]
pub struct UpdateLayer {
    id: String,
    is_visible: bool,
    old_is_visible: Option<bool>,
}

impl UpdateLayer {
    pub fn new(id: String, is_visible: bool) -> Self {
        UpdateLayer {
            id,
            is_visible,
            old_is_visible: None,
        }
    }
}

impl UndoableAction for UpdateLayer {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.id) {
            self.old_is_visible = Some(layer.is_visible);
            layer.is_visible = self.is_visible;
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateLayer: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.id) {
            if let Some(old_is_visible) = self.old_is_visible.take() {
                layer.is_visible = old_is_visible;
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateLayer (Undo): No `old_is_visible` on action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateLayer (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }
}
