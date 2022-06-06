use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapLayer;

#[derive(Debug)]
pub struct DeleteLayer {
    id: String,
    layer: Option<MapLayer>,
    draw_order_index: Option<usize>,
}

impl DeleteLayer {
    pub fn new(id: String) -> Self {
        DeleteLayer {
            id,
            layer: None,
            draw_order_index: None,
        }
    }
}

impl UndoableAction for DeleteLayer {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.remove(&self.id) {
            self.layer = Some(layer);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteLayer: The specified layer does not exist",
            ));
        }

        let len = map.draw_order.len();
        for i in 0..len {
            let layer_id = map.draw_order.get(i).unwrap();
            if layer_id == &self.id {
                map.draw_order.remove(i);
                self.draw_order_index = Some(i);
                break;
            }
        }

        if self.draw_order_index.is_none() {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteLayer: The specified layer was not found in the draw order array",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = self.layer.take() {
            map.layers.insert(self.id.clone(), layer);
            if let Some(i) = self.draw_order_index {
                if i >= map.draw_order.len() {
                    map.draw_order.push(self.id.clone());
                } else {
                    map.draw_order.insert(i, self.id.clone());
                }
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteLayer (Undo): No draw order index stored in action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteLayer (Undo): No layer stored in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}
