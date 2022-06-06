use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use macroquad::prelude::Vec2;

use crate::map::Map;

use super::UndoableAction;

#[derive(Debug)]
pub struct DeleteSpawnPoint {
    index: usize,
    spawn_point: Option<Vec2>,
}

impl DeleteSpawnPoint {
    pub fn new(index: usize) -> Self {
        DeleteSpawnPoint {
            index,
            spawn_point: None,
        }
    }
}

impl UndoableAction for DeleteSpawnPoint {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        let spawn_point = map.spawn_points.remove(self.index);
        self.spawn_point = Some(spawn_point);

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(spawn_point) = self.spawn_point.take() {
            if self.index >= map.spawn_points.len() {
                map.spawn_points.push(spawn_point);
            } else {
                map.spawn_points.insert(self.index, spawn_point);
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteSpawnPoint (Undo): No spawn point saved in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}
