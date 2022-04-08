use std::borrow::Cow;

use hv_lua::{FromLua, ToLua, UserData, Value};
use macroquad::{
    audio::Sound,
    prelude::{Color, Rect, Texture2D, Vec2},
};
use tealr::{
    mlu::{MaybeSend, TealData, UserDataWrapper},
    new_type, TypeBody, TypeName,
};

#[derive(Clone)]
pub struct Vec2Lua {
    pub x: f32,
    pub y: f32,
}
impl From<Vec2> for Vec2Lua {
    fn from(x: Vec2) -> Self {
        Self { x: x.x, y: x.y }
    }
}
impl From<Vec2Lua> for Vec2 {
    fn from(x: Vec2Lua) -> Self {
        Self::new(x.x, x.y)
    }
}
impl<'lua> ToLua<'lua> for Vec2Lua {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let position = lua.create_table()?;
        position.set("x", self.x)?;
        position.set("y", self.y)?;
        lua.pack(position)
    }
}

impl<'lua> FromLua<'lua> for Vec2Lua {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let value = match lua_value {
            hv_lua::Value::Table(x) => x,
            x => {
                return Err(hv_lua::Error::FromLuaConversionError {
                    from: x.type_name(),
                    to: "Table",
                    message: None,
                })
            }
        };
        let x = value.get("x")?;
        let y = value.get("y")?;
        Ok(Self { x, y })
    }
}

impl TypeName for Vec2Lua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        tealr::new_type!(Vec2, External)
    }
}
impl TypeBody for Vec2Lua {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("x").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("y").into(), f32::get_type_parts()));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorLua {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl TypeBody for ColorLua {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push(("r".as_bytes().to_vec().into(), f32::get_type_parts()));
        gen.fields
            .push(("g".as_bytes().to_vec().into(), f32::get_type_parts()));
        gen.fields
            .push(("b".as_bytes().to_vec().into(), f32::get_type_parts()));
        gen.fields
            .push(("a".as_bytes().to_vec().into(), f32::get_type_parts()));
    }
}

impl TypeName for ColorLua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        tealr::new_type!(Color, External)
    }
}
impl From<Color> for ColorLua {
    fn from(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

impl From<ColorLua> for Color {
    fn from(color: ColorLua) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}
impl<'lua> FromLua<'lua> for ColorLua {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let v = match lua_value {
            Value::Table(x) => x,
            x => {
                return Err(hv_lua::Error::FromLuaConversionError {
                    from: x.type_name(),
                    to: "table",
                    message: None,
                })
            }
        };
        Ok(Self {
            r: v.get("r")?,
            g: v.get("g")?,
            b: v.get("b")?,
            a: v.get("a")?,
        })
    }
}
impl<'lua> ToLua<'lua> for ColorLua {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Value<'lua>> {
        let table = lua.create_table()?;
        table.set("r", self.r)?;
        table.set("g", self.g)?;
        table.set("b", self.b)?;
        table.set("a", self.a)?;
        lua.pack(table)
    }
}

#[derive(Clone)]
pub struct RectLua(Rect);
impl From<RectLua> for Rect {
    fn from(r: RectLua) -> Self {
        r.0
    }
}
impl From<Rect> for RectLua {
    fn from(x: Rect) -> Self {
        Self(x)
    }
}
impl TypeName for RectLua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        tealr::new_type!(Rect, External)
    }
}
impl UserData for RectLua {
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
impl TealData for RectLua {
    fn add_type_methods<'lua, M: tealr::mlu::TealDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) where
        Self: 'static + MaybeSend,
    {
        methods.add_function("new", |_, (x, y, w, h)| {
            Ok(RectLua::from(Rect::new(x, y, w, h)))
        })
    }
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("point", |lua, this, ()| Ok(Vec2Lua::from(this.0.point())));
        methods.add_method("size", |lua, this, ()| Ok(Vec2Lua::from(this.0.size())));
        methods.add_method("left", |lua, this, ()| Ok(this.0.left()));
        methods.add_method("right", |lua, this, ()| Ok(this.0.right()));
        methods.add_method("top", |lua, this, ()| Ok(this.0.top()));
        methods.add_method("bottom", |lua, this, ()| Ok(this.0.bottom()));
        methods.add_method_mut("move_to", |_, this, vec: Vec2Lua| {
            this.0.move_to(vec.into());
            Ok(())
        });
        methods.add_method_mut("scale", |_, this, (sx, sy)| {
            this.0.scale(sx, sy);
            Ok(())
        });
        methods.add_method("contains", |lua, this, point: Vec2Lua| {
            Ok(this.0.contains(point.into()))
        });
        methods.add_method("overlaps", |lua, this, other: RectLua| {
            Ok(this.0.overlaps(&other.into()))
        });
        methods.add_method("combine_with", |lua, this, other: RectLua| {
            Ok(RectLua::from(this.0.combine_with(other.into())))
        });
        methods.add_method("intersect", |lua, this, other: RectLua| {
            Ok(this.0.intersect(other.into()).map(RectLua::from))
        });
        methods.add_method("offset", |lua, this, offset: Vec2Lua| {
            Ok(RectLua::from(this.0.offset(offset.into())))
        });
    }
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |lua, this| Ok(this.0.x));
        fields.add_field_method_get("y", |lua, this| Ok(this.0.y));
        fields.add_field_method_get("w", |lua, this| Ok(this.0.w));
        fields.add_field_method_get("h", |lua, this| Ok(this.0.h));
        fields.add_field_method_set("x", |_, this, value| {
            this.0.x = value;
            Ok(())
        });
        fields.add_field_method_set("y", |_, this, value| {
            this.0.y = value;
            Ok(())
        });
        fields.add_field_method_set("w", |_, this, value| {
            this.0.w = value;
            Ok(())
        });
        fields.add_field_method_set("h", |_, this, value| {
            this.0.h = value;
            Ok(())
        });
    }
}
impl TypeBody for RectLua {
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

#[derive(Clone)]
pub struct Texture2DLua(Texture2D);
impl TypeName for Texture2DLua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        new_type!(Texture2D, External)
    }
}
impl UserData for Texture2DLua {}
impl TypeBody for Texture2DLua {
    fn get_type_body(_: &mut tealr::TypeGenerator) {}
}

impl From<Texture2DLua> for Texture2D {
    fn from(x: Texture2DLua) -> Self {
        x.0
    }
}
impl From<Texture2D> for Texture2DLua {
    fn from(x: Texture2D) -> Self {
        Self(x)
    }
}

#[derive(Clone)]
pub struct SoundLua(Sound);
impl TypeName for SoundLua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        new_type!(Sound, External)
    }
}
impl UserData for SoundLua {}
impl TealData for SoundLua {}
impl TypeBody for SoundLua {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
    }
}

impl From<Sound> for SoundLua {
    fn from(s: Sound) -> Self {
        Self(s)
    }
}
impl From<SoundLua> for Sound {
    fn from(x: SoundLua) -> Self {
        x.0
    }
}
