mod animated_sprite;
mod sprite;

pub use animated_sprite::*;
use hv_cell::AtomicRefCell;
use hv_lua::{FromLua, ToLua, UserData};
pub use sprite::*;
use std::{
    borrow::{Borrow, BorrowMut, Cow},
    sync::Arc,
};
use tealr::{
    mlu::{MaybeSend, TealData, UserDataWrapper},
    TypeBody, TypeName,
};

use macroquad::prelude::*;

use hecs::World;

use core::{lua::get_table, Transform};

/// This is a wrapper type for all the different types of drawable sprites, used so that we can
/// access them all in one query and draw them, ordered, in one pass, according to `draw_order`.
#[derive(Clone, TypeName)]
pub struct Drawable {
    /// This is used to specify draw order on a sprite
    /// This will be used, primarily, by `Player` to draw equipped items in the right order, relative
    /// to its own sprite. This is done by multiplying the player id by ten and adding whatever offset
    /// is required to this number, to order it relative to other sprites controlled by this specific
    /// `Player` component.
    pub draw_order: u32,
    pub kind: DrawableKind,
}

impl<'lua> FromLua<'lua> for Drawable {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            draw_order: table.get("draw_order")?,
            kind: table.get("kind")?,
        })
    }
}
impl<'lua> ToLua<'lua> for Drawable {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("draw_order", self.draw_order)?;
        table.set("kind", self.kind)?;
        lua.pack(table)
    }
}
impl TypeBody for Drawable {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("draw_order").into(), u32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("kind").into(), DrawableKind::get_type_parts()));
    }
}

impl Drawable {
    pub fn new_sprite(draw_order: u32, texture_id: &str, params: SpriteParams) -> Self {
        let sprite = Sprite::new(texture_id, params);

        Drawable {
            draw_order,
            kind: DrawableKind::Sprite(sprite),
        }
    }

    pub fn new_sprite_set(draw_order: u32, sprites: &[(&str, Sprite)]) -> Self {
        let sprite_set = SpriteSet::from(sprites);

        Drawable {
            draw_order,
            kind: DrawableKind::SpriteSet(sprite_set),
        }
    }

    pub fn new_animated_sprite(
        draw_order: u32,
        texture_id: &str,
        animations: &[Animation],
        params: AnimatedSpriteParams,
    ) -> Self {
        let sprite = AnimatedSprite::new(texture_id, animations, params);

        Drawable {
            draw_order,
            kind: DrawableKind::AnimatedSprite(sprite),
        }
    }

    pub fn new_animated_sprite_set(draw_order: u32, sprites: &[(&str, AnimatedSprite)]) -> Self {
        let sprite_set = AnimatedSpriteSet::from(sprites);

        Drawable {
            draw_order,
            kind: DrawableKind::AnimatedSpriteSet(sprite_set),
        }
    }

    pub fn get_sprite(&self) -> Option<&Sprite> {
        match self.kind.borrow() {
            DrawableKind::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_sprite_mut(&mut self) -> Option<&mut Sprite> {
        match self.kind.borrow_mut() {
            DrawableKind::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_sprite_set(&self) -> Option<&SpriteSet> {
        match self.kind.borrow() {
            DrawableKind::SpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }

    pub fn get_sprite_set_mut(&mut self) -> Option<&mut SpriteSet> {
        match self.kind.borrow_mut() {
            DrawableKind::SpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }

    pub fn get_animated_sprite(&self) -> Option<&AnimatedSprite> {
        match self.kind.borrow() {
            DrawableKind::AnimatedSprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_animated_sprite_mut(&mut self) -> Option<&mut AnimatedSprite> {
        match self.kind.borrow_mut() {
            DrawableKind::AnimatedSprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_animated_sprite_set(&self) -> Option<&AnimatedSpriteSet> {
        match self.kind.borrow() {
            DrawableKind::AnimatedSpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }

    pub fn get_animated_sprite_set_mut(&mut self) -> Option<&mut AnimatedSpriteSet> {
        match self.kind.borrow_mut() {
            DrawableKind::AnimatedSpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }
}

#[derive(Clone, TypeName)]
pub enum DrawableKind {
    Sprite(Sprite),
    SpriteSet(SpriteSet),
    AnimatedSprite(AnimatedSprite),
    AnimatedSpriteSet(AnimatedSpriteSet),
}
impl UserData for DrawableKind {
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
impl TealData for DrawableKind {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("try_get_sprite", |_, this, ()| {
            if let DrawableKind::Sprite(x) = this {
                Ok((true, Some(x.to_owned())))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_sprite_set", |_, this, ()| {
            if let DrawableKind::SpriteSet(x) = this {
                Ok((true, Some(x.to_owned())))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_animated_sprite", |_, this, ()| {
            if let DrawableKind::AnimatedSprite(x) = this {
                Ok((true, Some(x.to_owned())))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_animated_sprite_set", |_, this, ()| {
            if let DrawableKind::AnimatedSpriteSet(x) = this {
                Ok((true, Some(x.to_owned())))
            } else {
                Ok((false, None))
            }
        });
    }
    fn add_type_methods<'lua, M: tealr::mlu::TealDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) where
        Self: 'static + tealr::mlu::MaybeSend,
    {
        methods.add_function("new_sprite", |_, v| Ok(Self::Sprite(v)));
        methods.add_function("new_sprite", |_, v| Ok(Self::AnimatedSprite(v)));
        methods.add_function("new_sprite", |_, v| Ok(Self::AnimatedSpriteSet(v)));
        methods.add_function("new_sprite", |_, v| Ok(Self::SpriteSet(v)));
    }
}
impl TypeBody for DrawableKind {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_methods(gen);
    }
    fn get_type_body_marker(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_type_methods(gen);
    }
}

pub fn draw_drawables(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    let mut ordered = world
        .query_mut::<&Drawable>()
        .into_iter()
        .map(|(e, drawable)| (e, drawable.draw_order))
        .collect::<Vec<_>>();

    ordered.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

    for e in ordered.into_iter().map(|(e, _)| e) {
        let transform = world.get_mut::<Transform>(e).unwrap();
        let mut drawable = world.get_mut::<Drawable>(e).unwrap();

        match drawable.kind.borrow_mut() {
            DrawableKind::Sprite(sprite) => {
                draw_one_sprite(&transform, sprite);
            }
            DrawableKind::SpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    draw_one_sprite(&transform, sprite);
                }
            }
            DrawableKind::AnimatedSprite(sprite) => {
                draw_one_animated_sprite(&transform, sprite);
            }
            DrawableKind::AnimatedSpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    draw_one_animated_sprite(&transform, sprite);
                }
            }
        }
    }
}

pub fn debug_draw_drawables(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    let mut ordered = world
        .query_mut::<&Drawable>()
        .into_iter()
        .map(|(e, drawable)| (e, drawable.draw_order))
        .collect::<Vec<_>>();

    ordered.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

    for e in ordered.into_iter().map(|(e, _)| e) {
        let position = world.get_mut::<Transform>(e).map(|t| t.position).unwrap();

        let drawable = world.get_mut::<Drawable>(e).unwrap();

        match drawable.kind.borrow() {
            DrawableKind::Sprite(sprite) => {
                debug_draw_one_sprite(position, sprite);
            }
            DrawableKind::SpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    debug_draw_one_sprite(position, sprite);
                }
            }
            DrawableKind::AnimatedSprite(sprite) => {
                debug_draw_one_animated_sprite(position, sprite);
            }
            DrawableKind::AnimatedSpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    debug_draw_one_animated_sprite(position, sprite);
                }
            }
        }
    }
}
