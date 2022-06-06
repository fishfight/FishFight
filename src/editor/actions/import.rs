use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use macroquad::prelude::Color;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapBackgroundLayer;

use crate::map::MapTileset;

#[derive(Debug)]
pub struct Import {
    tilesets: Vec<MapTileset>,
    background_color: Option<Color>,
    old_background_color: Option<Color>,
    background_layers: Vec<MapBackgroundLayer>,
    old_background_layers: Vec<MapBackgroundLayer>,
}

impl Import {
    pub fn new(
        tilesets: Vec<MapTileset>,
        background_color: Option<Color>,
        background_layers: Vec<MapBackgroundLayer>,
    ) -> Self {
        Import {
            tilesets,
            background_color,
            old_background_color: None,
            background_layers,
            old_background_layers: Vec::new(),
        }
    }
}

impl UndoableAction for Import {
    fn apply_to(&mut self, map: &mut Map) -> Result<()> {
        for tileset in &self.tilesets {
            let mut first_tile_id = 1;
            for tileset in map.tilesets.values() {
                let next_tile_id = tileset.first_tile_id + tileset.tile_cnt;
                if next_tile_id > first_tile_id {
                    first_tile_id = next_tile_id;
                }
            }

            let tileset = MapTileset {
                id: tileset.id.clone(),
                texture_id: tileset.texture_id.clone(),
                texture_size: tileset.texture_size,
                tile_size: tileset.tile_size,
                grid_size: tileset.grid_size,
                first_tile_id,
                tile_cnt: tileset.tile_cnt,
                tile_subdivisions: tileset.tile_subdivisions,
                autotile_mask: tileset.autotile_mask.clone(),
                tile_attributes: tileset.tile_attributes.clone(),
                properties: tileset.properties.clone(),
                bitmasks: None,
            };

            map.tilesets.insert(tileset.id.clone(), tileset);
        }

        if let Some(background_color) = self.background_color {
            self.old_background_color = Some(map.background_color);
            map.background_color = background_color;
        }

        self.old_background_layers = map.background_layers.clone();

        map.background_layers
            .append(&mut self.background_layers.clone());

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        for tileset in &self.tilesets {
            if map.tilesets.remove(&tileset.id).is_none() {
                return Err(Error::new_const(ErrorKind::EditorAction, &"ImportTilesets (Undo): One of the imported tilesets could not be found in the map"));
            }
        }

        if let Some(background_color) = self.old_background_color.take() {
            map.background_color = background_color;
        }

        map.background_layers = self.old_background_layers.drain(..).collect();

        Ok(())
    }
}
