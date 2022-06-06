use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

#[derive(Debug)]
pub struct SetLayerDrawOrderIndex {
    pub(crate) id: String,
    pub(crate) draw_order_index: usize,
    pub(crate) old_draw_order_index: Option<usize>,
}

impl SetLayerDrawOrderIndex {
    pub fn new(id: String, draw_order_index: usize) -> Self {
        SetLayerDrawOrderIndex {
            id,
            draw_order_index,
            old_draw_order_index: None,
        }
    }
}

impl UndoableAction for SetLayerDrawOrderIndex {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        for i in 0..map.draw_order.len() {
            let id = map.draw_order.get(i).unwrap();
            if id == &self.id {
                self.old_draw_order_index = Some(i);
                map.draw_order.remove(i);
                break;
            }
        }

        if self.old_draw_order_index.is_none() {
            return Err(Error::new_const(ErrorKind::EditorAction, &"SetLayerDrawOrderIndex: Could not find the specified layer in the map draw order array"));
        }

        if self.draw_order_index > map.draw_order.len() {
            map.draw_order.push(self.id.clone());
        } else {
            map.draw_order
                .insert(self.draw_order_index, self.id.clone());
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if map.draw_order.remove(self.draw_order_index) != self.id {
            return Err(Error::new_const(ErrorKind::EditorAction, &"SetLayerDrawOrderIndex (Undo): There was a mismatch between the layer id found in at the specified draw order index and the id stored in the action"));
        }

        if let Some(i) = self.old_draw_order_index {
            if i > map.draw_order.len() {
                map.draw_order.push(self.id.clone());
            } else {
                map.draw_order.insert(i, self.id.clone());
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"SetLayerDrawOrderIndex (Undo): No old draw order index was found. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        for i in 0..map.draw_order.len() {
            let id = map.draw_order.get(i).unwrap();
            if id == &self.id && i == self.draw_order_index {
                return true;
            }
        }

        false
    }
}
