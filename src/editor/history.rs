use core::Result;

use crate::Map;

use super::actions::UndoableAction;

pub struct ActionHistory {
    undo_stack: Vec<Box<dyn UndoableAction>>,
    redo_stack: Vec<Box<dyn UndoableAction>>,
}

impl ActionHistory {
    pub fn new() -> Self {
        ActionHistory {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn apply(
        &mut self,
        mut action: impl UndoableAction + 'static,
        map: &mut Map,
    ) -> Result<()> {
        if !action.is_redundant(map) {
            action.apply_to(map)?;
            self.undo_stack.push(Box::new(action));
            self.redo_stack.clear();
        }

        Ok(())
    }

    pub fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(mut action) = self.undo_stack.pop() {
            action.undo(map)?;
            self.redo_stack.push(action);
        }

        Ok(())
    }

    pub fn redo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(mut action) = self.redo_stack.pop() {
            action.apply_to(map)?;
            self.undo_stack.push(action);
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for ActionHistory {
    fn default() -> Self {
        Self::new()
    }
}
