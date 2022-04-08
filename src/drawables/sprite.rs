use core::lua::get_table;
use core::lua::wrapped_types::{ColorLua, RectLua, Texture2DLua, Vec2Lua};
use std::iter::FromIterator;
use std::ops::Div;
use std::{borrow::Cow, collections::HashMap};

use hv_lua::{FromLua, ToLua, UserData};
use macroquad::color;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};
use tealr::mlu::{MaybeSend, TealData, UserDataWrapper};
use tealr::{TypeBody, TypeName};

use core::Transform;

use crate::Resources;

/// Parameters for `Sprite` component.
#[derive(Debug, Clone, Serialize, Deserialize, tealr::TypeName)]
pub struct SpriteMetadata {
    /// The id of the texture that will be used
    #[serde(rename = "texture")]
    pub texture_id: String,
    /// The sprites index in the sprite sheet
    #[serde(default)]
    pub index: usize,
    /// This is a scale factor that the sprite size will be multiplied by before draw
    #[serde(default)]
    pub scale: Option<f32>,
    /// The offset of the drawn sprite, relative to the position provided as an argument to the
    /// `Sprite` draw method.
    /// Note that this offset will not be inverted if the sprite is flipped.
    #[serde(default, with = "core::json::vec2_def")]
    pub offset: Vec2,
    /// The pivot of the sprite, relative to the position provided as an argument to the `Sprite`
    /// draw method, plus any offset.
    /// Note that this offset will not be inverted if the sprite is flipped.
    #[serde(
        default,
        with = "core::json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot: Option<Vec2>,
    /// The size of the drawn sprite. If no size is specified, the texture entry's `sprite_size`
    /// will be used, if specified, or the raw texture size, if not.
    #[serde(
        default,
        with = "core::json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<Vec2>,
    /// An optional color to blend with the texture color
    #[serde(
        default,
        with = "core::json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    /// If this is true, the sprite will not be drawn.
    #[serde(default)]
    pub is_deactivated: bool,
}

impl TypeBody for SpriteMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("texture").into(), String::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("index").into(), usize::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("scale").into(),
            Option::<f32>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("pivot").into(),
            Option::<Vec2Lua>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("size").into(),
            Option::<Vec2Lua>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("tint").into(),
            Option::<ColorLua>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("is_deactivated").into(),
            bool::get_type_parts(),
        ));
    }
}

impl Default for SpriteMetadata {
    fn default() -> Self {
        SpriteMetadata {
            texture_id: "".to_string(),
            index: 0,
            scale: None,
            offset: Vec2::ZERO,
            pivot: None,
            size: None,
            tint: None,
            is_deactivated: false,
        }
    }
}

#[derive(Debug, Clone, TypeName)]
pub struct Sprite {
    pub texture: Texture2D,
    pub source_rect: Rect,
    pub tint: Color,
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub is_flipped_x: bool,
    pub is_flipped_y: bool,
    pub is_deactivated: bool,
}

impl UserData for Sprite {
    fn add_fields<'lua, F: hv_lua::UserDataFields<'lua, Self>>(fields: &mut F) {
        let mut wrapper = UserDataWrapper::from_user_data_fields(fields);
        <Self as TealData>::add_fields(&mut wrapper)
    }
    fn add_methods<'lua, M: hv_lua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
    fn add_type_methods<'lua, M: hv_lua::UserDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) where
        Self: 'static + MaybeSend,
    {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_type_methods(&mut wrapper)
    }
}
impl TealData for Sprite {
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("texture", |_, this| Ok(Texture2DLua::from(this.texture)));
        fields.add_field_method_get("source_rect", |_, this| Ok(RectLua::from(this.source_rect)));
        fields.add_field_method_get("tint", |_, this| Ok(ColorLua::from(this.tint)));
        fields.add_field_method_get("scale", |_, this| Ok(this.scale));
        fields.add_field_method_get("offset", |_, this| Ok(Vec2Lua::from(this.offset)));
        fields.add_field_method_get("pivot", |_, this| Ok(this.pivot.map(Vec2Lua::from)));
        fields.add_field_method_get("is_flipped_x", |_, this| Ok(this.is_flipped_x));
        fields.add_field_method_get("is_flipped_y", |_, this| Ok(this.is_flipped_y));
        fields.add_field_method_get("is_deactivated", |_, this| Ok(this.is_deactivated));
    }
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("size", |_, this, ()| Ok(Vec2Lua::from(this.size())));
        methods.add_method_mut("set_scale", |_, this, scale| {
            this.set_scale(scale);
            Ok(())
        })
    }

    fn add_type_methods<'lua, M: tealr::mlu::TealDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) where
        Self: 'static + MaybeSend,
    {
        methods.add_function("new", |_, (texture, params): (String, SpriteParams)| {
            Ok(Self::new(&texture, params))
        })
    }
}
impl TypeBody for Sprite {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_fields(gen);
        <Self as TealData>::add_methods(gen);
    }

    fn get_type_body_marker(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_type_methods(gen);
    }
}

impl Sprite {
    pub fn new(texture_id: &str, params: SpriteParams) -> Self {
        let texture_res = {
            let resources = storage::get::<Resources>();
            resources
                .textures
                .get(texture_id)
                .cloned()
                .unwrap_or_else(|| panic!("Sprite: Invalid texture ID '{}'", texture_id))
        };

        let sprite_size = params
            .sprite_size
            .unwrap_or_else(|| texture_res.frame_size());

        let source_rect = {
            let grid_size = texture_res.meta.size.div(sprite_size).as_u32();

            {
                let frame_cnt = (grid_size.x * grid_size.y) as usize;
                assert!(
                    params.index < frame_cnt,
                    "Sprite: index '{}' exceeds total frame count '{}'",
                    params.index,
                    frame_cnt
                );
            }

            let position = vec2(
                (params.index as u32 % grid_size.x) as f32 * sprite_size.x,
                (params.index as u32 / grid_size.x) as f32 * sprite_size.y,
            );

            Rect::new(position.x, position.y, sprite_size.x, sprite_size.y)
        };

        let tint = params.tint.unwrap_or(color::WHITE);

        Sprite {
            texture: texture_res.texture,
            source_rect,
            tint,
            scale: params.scale,
            offset: params.offset,
            pivot: params.pivot,
            is_flipped_x: params.is_flipped_x,
            is_flipped_y: params.is_flipped_y,
            is_deactivated: params.is_deactivated,
        }
    }

    pub fn size(&self) -> Vec2 {
        self.source_rect.size() * self.scale
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

#[derive(Clone, TypeName)]
pub struct SpriteParams {
    pub sprite_size: Option<Vec2>,
    pub index: usize,
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub size: Option<Vec2>,
    pub tint: Option<Color>,
    pub is_flipped_x: bool,
    pub is_flipped_y: bool,
    pub is_deactivated: bool,
}

impl<'lua> FromLua<'lua> for SpriteParams {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            sprite_size: table
                .get::<_, Option<Vec2Lua>>("sprite_size")?
                .map(Vec2::from),
            index: table.get("index")?,
            scale: table.get("scale")?,
            offset: table.get::<_, Vec2Lua>("offset")?.into(),
            pivot: table.get::<_, Option<Vec2Lua>>("pivot")?.map(Vec2::from),
            size: table.get::<_, Option<Vec2Lua>>("size")?.map(Vec2::from),
            tint: table.get::<_, Option<ColorLua>>("tint")?.map(Color::from),
            is_flipped_x: table.get("is_flipped_x")?,
            is_flipped_y: table.get("is_flipped_y")?,
            is_deactivated: table.get("is_deactivated")?,
        })
    }
}
impl<'lua> ToLua<'lua> for SpriteParams {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("sprite_size", self.sprite_size.map(Vec2Lua::from))?;
        table.set("index", self.index)?;
        table.set("scale", self.scale)?;
        table.set("offset", Vec2Lua::from(self.offset))?;
        table.set("pivot", self.pivot.map(Vec2Lua::from))?;
        table.set("size", self.size.map(Vec2Lua::from))?;
        table.set("tint", self.tint.map(ColorLua::from))?;
        table.set("is_flipped_x", self.is_flipped_x)?;
        table.set("is_flipped_y", self.is_flipped_y)?;
        table.set("is_deactivated", self.is_deactivated)?;
        lua.pack(table)
    }
}

impl TypeBody for SpriteParams {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("sprite_size").into(),
            Option::<Vec2Lua>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("index").into(), usize::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("scale").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("pivot").into(),
            Option::<Vec2Lua>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("size").into(),
            Option::<Vec2Lua>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("tint").into(),
            Option::<ColorLua>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("is_flipped_x").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("is_flipped_y").into(), bool::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("is_deactivated").into(),
            bool::get_type_parts(),
        ));
    }
}

impl Default for SpriteParams {
    fn default() -> Self {
        SpriteParams {
            sprite_size: None,
            index: 0,
            scale: 1.0,
            offset: Vec2::ZERO,
            pivot: None,
            size: None,
            tint: None,
            is_flipped_x: false,
            is_flipped_y: false,
            is_deactivated: false,
        }
    }
}

impl From<SpriteMetadata> for SpriteParams {
    fn from(meta: SpriteMetadata) -> Self {
        SpriteParams {
            index: meta.index,
            scale: meta.scale.unwrap_or(1.0),
            offset: meta.offset,
            pivot: meta.pivot,
            size: meta.size,
            tint: meta.tint,
            is_deactivated: meta.is_deactivated,
            ..Default::default()
        }
    }
}

pub fn draw_one_sprite(transform: &Transform, sprite: &Sprite) {
    if !sprite.is_deactivated {
        let size = sprite.size();

        draw_texture_ex(
            sprite.texture,
            transform.position.x + sprite.offset.x,
            transform.position.y + sprite.offset.y,
            sprite.tint,
            DrawTextureParams {
                flip_x: sprite.is_flipped_x,
                flip_y: sprite.is_flipped_y,
                rotation: transform.rotation,
                source: Some(sprite.source_rect),
                dest_size: Some(size),
                pivot: sprite.pivot,
            },
        );
    }
}

pub fn debug_draw_one_sprite(position: Vec2, sprite: &Sprite) {
    if !sprite.is_deactivated {
        let size = sprite.size();

        draw_rectangle_lines(
            position.x + sprite.offset.x,
            position.y + sprite.offset.y,
            size.x,
            size.y,
            2.0,
            color::BLUE,
        )
    }
}

#[derive(Debug, Clone, TypeName)]
pub struct SpriteSet {
    pub draw_order: Vec<String>,
    pub map: HashMap<String, Sprite>,
}

impl From<&[(&str, Sprite)]> for SpriteSet {
    fn from(sprites: &[(&str, Sprite)]) -> Self {
        let draw_order = sprites.iter().map(|(id, _)| id.to_string()).collect();

        let map = HashMap::from_iter(
            sprites
                .iter()
                .map(|(id, sprite)| (id.to_string(), sprite.clone())),
        );

        SpriteSet { draw_order, map }
    }
}

impl UserData for SpriteSet {
    fn add_methods<'lua, M: hv_lua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
    fn add_type_methods<'lua, M: hv_lua::UserDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) where
        Self: 'static + MaybeSend,
    {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_type_methods(&mut wrapper)
    }
}
impl TealData for SpriteSet {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("is_empty", |_, this, ()| Ok(this.is_empty()));
        methods.add_method_mut("flip_all_x", |_, this, state| {
            this.flip_all_x(state);
            Ok(())
        });
        methods.add_method_mut("flip_all_x", |_, this, state| {
            this.flip_all_y(state);
            Ok(())
        });
        methods.add_method_mut("activate_all", |_, this, ()| {
            this.activate_all();
            Ok(())
        });
        methods.add_method_mut("deactivate_all", |_, this, ()| {
            this.deactivate_all();
            Ok(())
        });
    }
    fn add_type_methods<'lua, M: tealr::mlu::TealDataMethods<'lua, hv_alchemy::Type<Self>>>(
        _methods: &mut M,
    ) where
        Self: 'static + MaybeSend,
    {
    }
}

impl TypeBody for SpriteSet {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_methods(gen);
    }
}

impl SpriteSet {
    pub fn is_empty(&self) -> bool {
        self.draw_order.is_empty()
    }

    pub fn flip_all_x(&mut self, state: bool) {
        for sprite in self.map.values_mut() {
            sprite.is_flipped_x = state;
        }
    }

    pub fn flip_all_y(&mut self, state: bool) {
        for sprite in self.map.values_mut() {
            sprite.is_flipped_y = state;
        }
    }

    pub fn activate_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_deactivated = false;
        }
    }

    pub fn deactivate_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_deactivated = true;
        }
    }
}
