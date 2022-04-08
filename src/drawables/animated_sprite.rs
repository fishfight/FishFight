use core::lua::get_table;
use core::lua::wrapped_types::{ColorLua, RectLua, Texture2DLua};
use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::ops::Mul;
use std::{borrow::BorrowMut, sync::Arc};

use hv_cell::AtomicRefCell;
use hv_lua::{FromLua, LuaSerdeExt, ToLua, UserData, Value};
use macroquad::color;
use macroquad::experimental::animation::Animation as MQAnimation;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::World;

use serde::{Deserialize, Serialize};
use tealr::mlu::{MaybeSend, TealData, UserDataWrapper};
use tealr::{TypeBody, TypeName};

use core::{lua::wrapped_types::Vec2Lua, Transform};

use crate::{Drawable, DrawableKind, Resources};

#[derive(Debug, Clone, TypeName)]
pub struct Animation {
    pub id: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
    pub tweens: HashMap<String, Tween>,
    pub is_looping: bool,
}

impl<'lua> FromLua<'lua> for Animation {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            id: table.get("id")?,
            row: table.get("row")?,
            frames: table.get("frames")?,
            fps: table.get("fps")?,
            tweens: table.get("tweens")?,
            is_looping: table.get("is_looping")?,
        })
    }
}
impl<'lua> ToLua<'lua> for Animation {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("id", self.id)?;
        table.set("row", self.row)?;
        table.set("frames", self.frames)?;
        table.set("fps", self.fps)?;
        table.set("tweens", self.tweens)?;
        table.set("is_looping", self.is_looping)?;
        lua.pack(table)
    }
}
impl TypeBody for Animation {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("id").into(), String::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("row").into(), u32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("frames").into(), u32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("fps").into(), u32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("tweens").into(),
            HashMap::<String, Tween>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("is_looping").into(), bool::get_type_parts()));
    }
}

impl From<AnimationMetadata> for Animation {
    fn from(meta: AnimationMetadata) -> Self {
        let tweens = HashMap::from_iter(
            meta.tweens
                .into_iter()
                .map(|meta| (meta.id.clone(), meta.into())),
        );

        Animation {
            id: meta.id,
            row: meta.row,
            frames: meta.frames,
            fps: meta.fps,
            tweens,
            is_looping: meta.is_looping,
        }
    }
}

#[derive(Debug, Clone, TypeName)]
pub struct Tween {
    pub keyframes: Vec<Keyframe>,
    pub current_translation: Vec2,
}

impl<'lua> FromLua<'lua> for Tween {
    fn from_lua(lua_value: hv_lua::Value<'lua>, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            keyframes: lua.from_value(table.get::<_, Value>("keyframes")?)?,
            current_translation: table.get::<_, Vec2Lua>("current_translation")?.into(),
        })
    }
}

impl<'lua> ToLua<'lua> for Tween {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("keyframes", lua.to_value(&self.keyframes)?)?;
        table.set(
            "current_translation",
            Vec2Lua::from(self.current_translation),
        )?;
        lua.pack(table)
    }
}

impl TypeBody for Tween {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("keyframes").into(),
            Vec::<Keyframe>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("current_translation").into(),
            Vec2Lua::get_type_parts(),
        ));
    }
}

impl From<TweenMetadata> for Tween {
    fn from(meta: TweenMetadata) -> Self {
        let mut keyframes = meta.keyframes;

        keyframes.sort_by(|a, b| a.frame.cmp(&b.frame));

        let current_translation = keyframes
            .first()
            .map(|keyframe| keyframe.translation)
            .unwrap_or_else(|| Vec2::ZERO);

        Tween {
            keyframes,
            current_translation,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, tealr::TypeName)]
pub struct Keyframe {
    pub frame: u32,
    #[serde(with = "core::json::vec2_def")]
    pub translation: Vec2,
}
impl TypeBody for Keyframe {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("frame").into(), u32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("translation").into(),
            Vec2Lua::get_type_parts(),
        ));
    }
}

#[derive(Clone, TypeName)]
pub struct AnimatedSpriteParams {
    pub frame_size: Option<Vec2>,
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub tint: Color,
    pub is_flipped_x: bool,
    pub is_flipped_y: bool,
    pub autoplay_id: Option<String>,
}

impl<'lua> FromLua<'lua> for AnimatedSpriteParams {
    fn from_lua(lua_value: Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            frame_size: table
                .get::<_, Option<Vec2Lua>>("frame_size")?
                .map(Vec2::from),
            scale: table.get("scale")?,
            offset: table.get::<_, Vec2Lua>("offset")?.into(),
            pivot: table.get::<_, Option<Vec2Lua>>("pivot")?.map(Vec2::from),
            tint: table.get::<_, ColorLua>("tint")?.into(),
            is_flipped_x: table.get("is_flipped_x")?,
            is_flipped_y: table.get("is_flipped_y")?,
            autoplay_id: table.get("autoplay_id")?,
        })
    }
}

impl<'lua> ToLua<'lua> for AnimatedSpriteParams {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Value<'lua>> {
        let table = lua.create_table()?;
        table.set("frame_size", self.frame_size.map(Vec2Lua::from))?;
        table.set("scale", self.scale)?;
        table.set("offset", Vec2Lua::from(self.offset))?;
        table.set("pivot", self.pivot.map(Vec2Lua::from))?;
        table.set("tint", ColorLua::from(self.tint))?;
        table.set("is_flipped_x", self.is_flipped_x)?;
        table.set("is_flipped_y", self.is_flipped_y)?;
        table.set("autoplay_id", self.autoplay_id)?;
        lua.pack(table)
    }
}

impl TypeBody for AnimatedSpriteParams {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("frame_size").into(),
            Option::<Vec2Lua>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("scale").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("pivot").into(),
            Option::<Vec2Lua>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("tint").into(), ColorLua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("is_flipped_x").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("is_flipped_y").into(), bool::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("autoplay_id").into(),
            Option::<String>::get_type_parts(),
        ));
    }
}

impl Default for AnimatedSpriteParams {
    fn default() -> Self {
        AnimatedSpriteParams {
            frame_size: None,
            scale: 1.0,
            offset: Vec2::ZERO,
            pivot: None,
            tint: color::WHITE,
            is_flipped_x: false,
            is_flipped_y: false,
            autoplay_id: None,
        }
    }
}

impl From<AnimatedSpriteMetadata> for AnimatedSpriteParams {
    fn from(meta: AnimatedSpriteMetadata) -> Self {
        AnimatedSpriteParams {
            scale: meta.scale.unwrap_or(1.0),
            offset: meta.offset,
            pivot: meta.pivot,
            tint: meta.tint.unwrap_or(color::WHITE),
            autoplay_id: meta.autoplay_id,
            ..Default::default()
        }
    }
}

#[derive(Clone, TypeName)]
pub enum QueuedAnimationAction {
    Play(String),
    PlayIndex(usize),
    WaitThen(f32, Box<QueuedAnimationAction>),
    Deactivate,
}
impl UserData for QueuedAnimationAction {}
impl TypeBody for QueuedAnimationAction {
    fn get_type_body(_: &mut tealr::TypeGenerator) {}
}

impl QueuedAnimationAction {
    pub fn wait_then(delay: f32, action: QueuedAnimationAction) -> Self {
        QueuedAnimationAction::WaitThen(delay, Box::new(action))
    }
}

#[derive(Clone, TypeName)]
pub struct AnimatedSprite {
    pub texture: Texture2D,
    pub frame_size: Vec2,
    pub scale: f32,
    pub offset: Vec2,
    pub pivot: Option<Vec2>,
    pub tint: Color,
    pub animations: Vec<Animation>,
    pub current_index: usize,
    pub queued_action: Option<QueuedAnimationAction>,
    pub current_frame: u32,
    pub frame_timer: f32,
    pub is_playing: bool,
    pub is_flipped_x: bool,
    pub is_flipped_y: bool,
    pub is_deactivated: bool,
    pub wait_timer: f32,
}

impl UserData for AnimatedSprite {
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
impl TealData for AnimatedSprite {
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("texture", |_, this| Ok(Texture2DLua::from(this.texture)));
        fields.add_field_method_get("frame_size", |_, this| Ok(Vec2Lua::from(this.frame_size)));
        fields.add_field_method_get("scale", |_, this| Ok(this.scale));
        fields.add_field_method_get("offset", |_, this| Ok(Vec2Lua::from(this.offset)));
        fields.add_field_method_get("pivot", |_, this| Ok(this.pivot.map(Vec2Lua::from)));
        fields.add_field_method_get("tint", |_, this| Ok(ColorLua::from(this.tint)));
        fields.add_field_method_get("animations", |_, this| Ok(this.animations.clone()));
        fields.add_field_method_get("current_index", |_, this| Ok(this.current_index));
        fields.add_field_method_get("queued_action", |_, this| Ok(this.queued_action.clone()));
        fields.add_field_method_get("current_frame", |_, this| Ok(this.current_frame));
        fields.add_field_method_get("frame_timer", |_, this| Ok(this.frame_timer));
        fields.add_field_method_get("is_playing", |_, this| Ok(this.is_playing));
        fields.add_field_method_get("is_flipped_x", |_, this| Ok(this.is_flipped_x));
        fields.add_field_method_get("is_flipped_y", |_, this| Ok(this.is_flipped_y));
        fields.add_field_method_get("is_deactivated", |_, this| Ok(this.is_deactivated));
        fields.add_field_method_get("wait_timer", |_, this| Ok(this.wait_timer));
        fields.add_field_method_set("texture", |_, this, value: Texture2DLua| {
            this.texture = value.into();
            Ok(())
        });
        fields.add_field_method_set("frame_size", |_, this, value: Vec2Lua| {
            this.frame_size = value.into();
            Ok(())
        });
        fields.add_field_method_set("scale", |_, this, value| {
            this.scale = value;
            Ok(())
        });
        fields.add_field_method_set("offset", |_, this, value: Vec2Lua| {
            this.offset = value.into();
            Ok(())
        });
        fields.add_field_method_set("pivot", |_, this, value: Option<Vec2Lua>| {
            this.pivot = value.map(Into::into);
            Ok(())
        });
        fields.add_field_method_set("tint", |_, this, value: ColorLua| {
            this.tint = value.into();
            Ok(())
        });
        fields.add_field_method_set("animations", |_, this, value| {
            this.animations = value;
            Ok(())
        });
        fields.add_field_method_set("current_index", |_, this, value| {
            this.current_index = value;
            Ok(())
        });
        fields.add_field_method_set("queued_action", |_, this, value| {
            this.queued_action = value;
            Ok(())
        });
        fields.add_field_method_set("current_frame", |_, this, value| {
            this.current_frame = value;
            Ok(())
        });
        fields.add_field_method_set("frame_timer", |_, this, value| {
            this.frame_timer = value;
            Ok(())
        });
        fields.add_field_method_set("is_playing", |_, this, value| {
            this.is_playing = value;
            Ok(())
        });
        fields.add_field_method_set("is_flipped_x", |_, this, value| {
            this.is_flipped_x = value;
            Ok(())
        });
        fields.add_field_method_set("is_flipped_y", |_, this, value| {
            this.is_flipped_y = value;
            Ok(())
        });
        fields.add_field_method_set("is_deactivated", |_, this, value| {
            this.is_deactivated = value;
            Ok(())
        });
        fields.add_field_method_set("wait_timer", |_, this, value| {
            this.wait_timer = value;
            Ok(())
        });
    }
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("get_animation", |_, this, value: String| {
            Ok(this.get_animation(&value).map(ToOwned::to_owned))
        });
        methods.add_method("current_animation", |_, this, ()| {
            Ok(this.current_animation().to_owned())
        });
        methods.add_method("size", |_, this, ()| Ok(Vec2Lua::from(this.size())));
        methods.add_method("source_rect", |_, this, ()| {
            Ok(RectLua::from(this.source_rect()))
        });
        methods.add_method("as_index", |_, this, value: String| {
            Ok(this.as_index(&value))
        });
        methods.add_method_mut("set_animation_index", |_, this, (index, should_restart)| {
            this.set_animation_index(index, should_restart);
            Ok(())
        });
        methods.add_method_mut(
            "set_animation",
            |_, this, (id, should_restart): (String, _)| {
                this.set_animation(&id, should_restart);
                Ok(())
            },
        );
        methods.add_method_mut("queue_action", |_, this, value| {
            this.queue_action(value);
            Ok(())
        });
        methods.add_method_mut("restart", |_, this, ()| {
            this.restart();
            Ok(())
        });
    }
    fn add_type_methods<'lua, M: tealr::mlu::TealDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) where
        Self: 'static + tealr::mlu::MaybeSend,
    {
        methods.add_function("new",|_,(texture_id, animations,params): (String, Vec<Animation>, AnimatedSpriteParams)|{
            Ok(AnimatedSprite::new(&texture_id, &animations, params))
        })
    }
}
impl TypeBody for AnimatedSprite {
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

impl AnimatedSprite {
    pub fn new(texture_id: &str, animations: &[Animation], params: AnimatedSpriteParams) -> Self {
        let animations = animations.to_vec();

        let texture_res = {
            let resources = storage::get::<Resources>();
            resources
                .textures
                .get(texture_id)
                .cloned()
                .unwrap_or_else(|| panic!("AnimatedSprite: Invalid texture ID '{}'", texture_id))
        };

        let mut is_playing = false;
        let mut current_index = 0;

        if let Some(autoplay_id) = &params.autoplay_id {
            is_playing = true;

            for (i, animation) in animations.iter().enumerate() {
                if animation.id == *autoplay_id {
                    current_index = i;
                    break;
                }
            }
        }

        let frame_size = params
            .frame_size
            .unwrap_or_else(|| texture_res.frame_size());

        AnimatedSprite {
            texture: texture_res.texture,
            frame_size,
            animations,
            scale: params.scale,
            offset: params.offset,
            pivot: params.pivot,
            tint: params.tint,
            frame_timer: 0.0,
            current_index,
            queued_action: None,
            current_frame: 0,
            is_playing,
            is_flipped_x: params.is_flipped_x,
            is_flipped_y: params.is_flipped_y,
            is_deactivated: false,
            wait_timer: 0.0,
        }
    }

    pub fn get_animation(&self, id: &str) -> Option<&Animation> {
        self.animations.iter().find(|&a| a.id == *id)
    }

    pub fn current_animation(&self) -> &Animation {
        self.animations.get(self.current_index).unwrap()
    }

    pub fn size(&self) -> Vec2 {
        self.frame_size * self.scale
    }

    pub fn source_rect(&self) -> Rect {
        let animation = self.animations.get(self.current_index).unwrap();

        Rect::new(
            self.current_frame as f32 * self.frame_size.x,
            animation.row as f32 * self.frame_size.y,
            self.frame_size.x,
            self.frame_size.y,
        )
    }

    pub fn as_index(&self, id: &str) -> Option<usize> {
        self.animations
            .iter()
            .enumerate()
            .find(|&(_, a)| a.id == *id)
            .map(|(i, _)| i)
    }

    pub fn set_animation_index(&mut self, index: usize, should_restart: bool) {
        if should_restart || self.current_index != index {
            self.wait_timer = 0.0;
            self.current_index = index;
            self.current_frame = 0;
            self.frame_timer = 0.0;
            self.is_playing = true;
        }
    }

    pub fn set_animation(&mut self, id: &str, should_restart: bool) {
        if let Some(index) = self.as_index(id) {
            self.set_animation_index(index, should_restart);
        }
    }

    pub fn queue_action(&mut self, action: QueuedAnimationAction) {
        self.queued_action = Some(action);
    }

    pub fn restart(&mut self) {
        self.current_frame = 0;
        self.frame_timer = 0.0;
        self.is_playing = true;
    }
}

pub fn update_animated_sprites(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    for (_, drawable) in world.query_mut::<&mut Drawable>() {
        match drawable.kind.borrow_mut() {
            DrawableKind::AnimatedSprite(sprite) => {
                update_one_animated_sprite(sprite);
            }
            DrawableKind::AnimatedSpriteSet(sprite_set) => {
                for key in &sprite_set.draw_order {
                    let sprite = sprite_set.map.get_mut(key).unwrap();
                    update_one_animated_sprite(sprite);
                }
            }
            _ => {}
        }
    }
}

pub fn update_one_animated_sprite(sprite: &mut AnimatedSprite) {
    let dt = get_frame_time();

    if !sprite.is_deactivated && sprite.is_playing {
        let (is_last_frame, is_looping) = {
            let animation = sprite.animations.get(sprite.current_index).unwrap();
            (
                sprite.current_frame == animation.frames - 1,
                animation.is_looping,
            )
        };

        if is_last_frame {
            let queued_action = sprite.queued_action.clone();

            if let Some(action) = queued_action {
                match action {
                    QueuedAnimationAction::Play(id) => {
                        sprite.set_animation(&id, false);
                        sprite.queued_action = None;
                    }
                    QueuedAnimationAction::PlayIndex(index) => {
                        sprite.set_animation_index(index, false);
                        sprite.queued_action = None;
                    }
                    QueuedAnimationAction::WaitThen(delay, action) => {
                        sprite.wait_timer += dt;
                        if sprite.wait_timer >= delay {
                            sprite.queued_action = Some(*action);
                            sprite.wait_timer = 0.0;
                        }
                    }
                    QueuedAnimationAction::Deactivate => {
                        sprite.is_deactivated = true;
                        sprite.queued_action = None;
                    }
                }
            } else {
                sprite.is_playing = is_looping;
            }
        }

        let (fps, frame_cnt, tweens) = {
            let animation = sprite.animations.get_mut(sprite.current_index).unwrap();
            (animation.fps, animation.frames, &mut animation.tweens)
        };

        if sprite.is_playing {
            sprite.frame_timer += dt;

            if sprite.frame_timer > 1.0 / fps as f32 {
                sprite.current_frame += 1;
                sprite.frame_timer = 0.0;
            }
        }

        sprite.current_frame %= frame_cnt;

        for tween in tweens.values_mut() {
            let mut current = tween.keyframes.first();
            let mut next = current;

            let len = tween.keyframes.len();

            if len > 0 {
                'tweens: for i in 1..len {
                    let keyframe = tween.keyframes.get(i).unwrap();

                    if sprite.current_frame < keyframe.frame {
                        next = Some(keyframe);

                        break 'tweens;
                    } else {
                        current = Some(keyframe);
                    }
                }
            }

            if let Some(current) = current {
                if let Some(next) = next {
                    let (frames, progress) = if current.frame <= next.frame {
                        let frames = next.frame - current.frame + 1;
                        let progress = sprite.current_frame - current.frame + 1;

                        (frames, progress)
                    } else {
                        let frames = frame_cnt + next.frame - current.frame;
                        let progress = if sprite.current_frame < current.frame {
                            frame_cnt + sprite.current_frame - current.frame
                        } else {
                            sprite.current_frame - current.frame + 1
                        };

                        (frames, progress)
                    };

                    let factor = progress as f32 / frames as f32;

                    tween.current_translation =
                        current.translation + (next.translation - current.translation).mul(factor);
                }
            }
        }
    }
}

pub fn draw_one_animated_sprite(transform: &Transform, sprite: &AnimatedSprite) {
    if !sprite.is_deactivated {
        let position = transform.position + sprite.offset;

        draw_texture_ex(
            sprite.texture,
            position.x,
            position.y,
            sprite.tint,
            DrawTextureParams {
                flip_x: sprite.is_flipped_x,
                flip_y: sprite.is_flipped_y,
                rotation: transform.rotation,
                source: Some(sprite.source_rect()),
                dest_size: Some(sprite.size()),
                pivot: sprite.pivot,
            },
        )
    }
}

pub fn debug_draw_one_animated_sprite(position: Vec2, sprite: &AnimatedSprite) {
    if !sprite.is_deactivated {
        let position = position + sprite.offset;
        let size = sprite.size();

        draw_rectangle_lines(position.x, position.y, size.x, size.y, 2.0, color::BLUE)
    }
}

#[derive(Default, Clone, TypeName)]
pub struct AnimatedSpriteSet {
    pub draw_order: Vec<String>,
    pub map: HashMap<String, AnimatedSprite>,
}

impl UserData for AnimatedSpriteSet {
    fn add_fields<'lua, F: hv_lua::UserDataFields<'lua, Self>>(fields: &mut F) {
        let mut wrapper = UserDataWrapper::from_user_data_fields(fields);
        <Self as TealData>::add_fields(&mut wrapper)
    }

    fn add_methods<'lua, M: hv_lua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
}
impl TealData for AnimatedSpriteSet {
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("draw_order", |_, this| Ok(this.draw_order.clone()));
        fields.add_field_method_get("map", |_, this| Ok(this.map.clone()));
    }
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("is_empty", |_, this, ()| Ok(this.is_empty()));
        methods.add_method("size", |_, this, ()| Ok(Vec2Lua::from(this.size())));
        methods.add_method_mut(
            "set_animation",
            |_, this, (sprite_id, id, should_restart): (String, String, _)| {
                this.set_animation(&sprite_id, &id, should_restart);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_animation_index",
            |_, this, (sprite_id, index, should_restart): (String, _, _)| {
                this.set_animation_index(&sprite_id, index, should_restart);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_queued_action",
            |_, this, (sprite_id, action): (String, _)| {
                this.set_queued_action(&sprite_id, action);
                Ok(())
            },
        );
        methods.add_method_mut("set_all", |_, this, (id, should_restart): (String, _)| {
            this.set_all(&id, should_restart);
            Ok(())
        });
        methods.add_method_mut("set_all_to_index", |_, this, (index, should_restart)| {
            this.set_all_to_index(index, should_restart);
            Ok(())
        });
        methods.add_method_mut("queue_action_on_all", |_, this, value| {
            this.queue_action_on_all(value);
            Ok(())
        });
        methods.add_method_mut("restart_all", |_, this, ()| {
            this.restart_all();
            Ok(())
        });
        methods.add_method_mut("flip_all_x", |_, this, value| {
            this.flip_all_x(value);
            Ok(())
        });
        methods.add_method_mut("flip_all_y", |_, this, value| {
            this.flip_all_y(value);
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
        methods.add_method_mut("play_all", |_, this, ()| {
            this.play_all();
            Ok(())
        });
        methods.add_method_mut("stop_all", |_, this, ()| {
            this.stop_all();
            Ok(())
        });
    }
    fn add_type_methods<'lua, M: tealr::mlu::TealDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) where
        Self: 'static + MaybeSend,
    {
        methods.add_function("new_default", |_, ()| Ok(Self::default()))
    }
}
impl TypeBody for AnimatedSpriteSet {
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
impl AnimatedSpriteSet {
    pub fn is_empty(&self) -> bool {
        self.draw_order.is_empty()
    }

    pub fn size(&self) -> Vec2 {
        let mut size = Vec2::ZERO;

        for sprite in self.map.values() {
            let sprite_size = sprite.size();

            if sprite_size.x > size.x {
                size.x = sprite_size.x;
            }

            if sprite_size.y > size.y {
                size.y = sprite_size.y;
            }
        }

        size
    }

    pub fn set_animation(&mut self, sprite_id: &str, id: &str, should_restart: bool) {
        if let Some(sprite) = self.map.get_mut(sprite_id) {
            sprite.set_animation(id, should_restart);
        }
    }

    pub fn set_animation_index(&mut self, sprite_id: &str, index: usize, should_restart: bool) {
        if let Some(sprite) = self.map.get_mut(sprite_id) {
            sprite.set_animation_index(index, should_restart);
        }
    }

    pub fn set_queued_action(&mut self, sprite_id: &str, action: QueuedAnimationAction) {
        if let Some(sprite) = self.map.get_mut(sprite_id) {
            sprite.queue_action(action);
        }
    }

    pub fn set_all(&mut self, id: &str, should_restart: bool) {
        for sprite in self.map.values_mut() {
            sprite.set_animation(id, should_restart);
        }
    }

    pub fn set_all_to_index(&mut self, index: usize, should_restart: bool) {
        for sprite in self.map.values_mut() {
            sprite.set_animation_index(index, should_restart);
        }
    }

    pub fn queue_action_on_all(&mut self, action: QueuedAnimationAction) {
        for sprite in self.map.values_mut() {
            sprite.queue_action(action.clone());
        }
    }

    pub fn restart_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.restart();
        }
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

    pub fn play_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_playing = true;
        }
    }

    pub fn stop_all(&mut self) {
        for sprite in self.map.values_mut() {
            sprite.is_playing = false;
        }
    }
}

impl From<&[(&str, AnimatedSprite)]> for AnimatedSpriteSet {
    fn from(sprites: &[(&str, AnimatedSprite)]) -> Self {
        let draw_order = sprites.iter().map(|&(k, _)| k.to_string()).collect();

        let map = HashMap::from_iter(
            sprites
                .iter()
                .map(|(id, sprite)| (id.to_string(), sprite.clone())),
        );

        AnimatedSpriteSet { draw_order, map }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, tealr::TypeName)]
pub struct AnimationMetadata {
    pub id: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tweens: Vec<TweenMetadata>,
    #[serde(default)]
    pub is_looping: bool,
}

impl TypeBody for AnimationMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("id").into(), String::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("row").into(), u32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("frames").into(), u32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("fps").into(), u32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("tweens").into(),
            Vec::<TweenMetadata>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("is_looping").into(), bool::get_type_parts()));
    }
}

impl From<AnimationMetadata> for MQAnimation {
    fn from(a: AnimationMetadata) -> Self {
        MQAnimation {
            name: a.id,
            row: a.row,
            frames: a.frames,
            fps: a.fps,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, tealr::TypeName)]
pub struct TweenMetadata {
    pub id: String,
    pub keyframes: Vec<Keyframe>,
}

impl<'lua> FromLua<'lua> for TweenMetadata {
    fn from_lua(lua_value: Value<'lua>, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        LuaSerdeExt::from_value(lua, lua_value)
    }
}
impl<'lua> ToLua<'lua> for TweenMetadata {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Value<'lua>> {
        LuaSerdeExt::to_value(lua, &self)
    }
}

impl TypeBody for TweenMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("id").into(), String::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("keyframes").into(),
            Vec::<Keyframe>::get_type_parts(),
        ));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, tealr::TypeName)]
pub struct AnimatedSpriteMetadata {
    #[serde(rename = "texture")]
    pub texture_id: String,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default, with = "core::json::vec2_def")]
    pub offset: Vec2,
    #[serde(
        default,
        with = "core::json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub pivot: Option<Vec2>,
    #[serde(
        default,
        with = "core::json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    pub animations: Vec<AnimationMetadata>,
    #[serde(default)]
    pub autoplay_id: Option<String>,
    #[serde(default)]
    pub is_deactivated: bool,
}
impl TypeBody for AnimatedSpriteMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("texture_id").into(), String::get_type_parts()));
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
            Cow::Borrowed("tint").into(),
            Option::<ColorLua>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("animations").into(),
            Vec::<AnimationMetadata>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("autoplay_id").into(),
            Option::<String>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("is_deactivated").into(),
            bool::get_type_parts(),
        ));
    }
}
