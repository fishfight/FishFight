use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use macroquad::prelude::Color;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapBackgroundLayer;

#[derive(Debug)]
pub struct UpdateBackground {
    pub(crate) color: Color,
    pub(crate) old_color: Option<Color>,
    pub(crate) layers: Vec<MapBackgroundLayer>,
    pub(crate) old_layers: Option<Vec<MapBackgroundLayer>>,
}

impl UpdateBackground {
    pub fn new(color: Color, layers: Vec<MapBackgroundLayer>) -> Self {
        UpdateBackground {
            color,
            old_color: None,
            layers,
            old_layers: None,
        }
    }
}

impl UndoableAction for UpdateBackground {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        self.old_color = Some(map.background_color);

        map.background_color = self.color;

        self.old_layers = Some(map.background_layers.clone());

        map.background_layers = self.layers.clone();

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(color) = self.old_color.take() {
            map.background_color = color;
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateBackgroundProperties (Undo): No old background color was found. Undo was probably called on an action that was never applied"));
        }

        if let Some(layers) = self.old_layers.take() {
            map.background_layers = layers;
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateBackgroundProperties (Undo): No old background layers was found. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}
