use core::error::ErrorKind;

use core::error::Error;

use crate::map::MapLayer;

use core::error::Result;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapLayerKind;

#[derive(Debug)]
pub struct CreateLayer {
    id: String,
    kind: MapLayerKind,
    has_collision: bool,
    index: Option<usize>,
}

impl CreateLayer {
    pub fn new(id: String, kind: MapLayerKind, has_collision: bool, index: Option<usize>) -> Self {
        CreateLayer {
            id,
            kind,
            has_collision,
            index,
        }
    }
}

impl UndoableAction for CreateLayer {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        if map.layers.contains_key(&self.id) {
            let len = map.draw_order.len();
            for i in 0..len {
                let id = map.draw_order.get(i).unwrap();
                if id == &self.id {
                    map.draw_order.remove(i);
                    break;
                }
            }
        }

        let layer = MapLayer::new(&self.id, self.kind, self.has_collision, map.grid_size);
        map.layers.insert(self.id.clone(), layer);

        if let Some(i) = self.index {
            if i <= map.draw_order.len() {
                map.draw_order.insert(i, self.id.clone());

                return Ok(());
            }
        }

        map.draw_order.push(self.id.clone());

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if map.layers.remove(&self.id).is_none() {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateLayer (Undo): The specified layer does not exist",
            ));
        }

        if let Some(i) = self.index {
            let id = map.draw_order.remove(i);
            if id != self.id {
                return Err(Error::new_const(ErrorKind::EditorAction, &"CreateLayer (Undo): There is a mismatch between the actions layer id and the one found at the actions draw order index"));
            }
        } else {
            map.draw_order.retain(|id| id != &self.id);
        }

        Ok(())
    }
}
